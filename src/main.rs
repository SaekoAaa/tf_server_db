use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use dotenvy::var;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use tokio::signal::unix::{SignalKind, signal};
use tracing::{info_span, level_filters::LevelFilter};

use crate::{
    metrics::{http_metrics_middleware, init_metrics},
    otel::OtelGuard,
    profiler::{Profiler, maybe_profile, profiling_middleware},
    request_error::RequestError,
};
mod metrics;
mod otel;
mod profiler;
mod request_error;
struct AppState {
    db_pool: sqlx::MySqlPool,
    profiler: Option<Arc<Profiler>>,
}
type ArcState = Arc<AppState>;
#[derive(thiserror::Error, Debug)]
enum ConnectionError {
    #[error("Failed to parse opts: {0}")]
    ParseOptsError(#[from] sqlx::Error),
    #[error("Failed to connect to database")]
    ConnectFailed,
}

async fn connect_to_database(
    mysql_connection_string: &str,
) -> Result<sqlx::MySqlPool, ConnectionError> {
    let retries = 3;
    for retry in 0..retries {
        let opts = MySqlPoolOptions::new();
        match opts.connect(mysql_connection_string).await {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                tracing::error!("Error connecting to database: {e} (retries {retry}/{retries})");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
    Err(ConnectionError::ConnectFailed)
}

#[derive(sqlx::FromRow, Serialize)]
struct Todo {
    id: i32,
    name: String,
}
async fn get_todos(State(state): State<ArcState>) -> Result<impl IntoResponse, RequestError> {
    let res = maybe_profile(
        state.profiler.clone(),
        "sql_query",
        "get_todos",
        async || {
            sqlx::query_as::<_, Todo>(
                r#"
        select id, name from todos
"#,
            )
            .fetch_all(&state.db_pool)
            .await
        },
    )
    .await?;
    Ok((StatusCode::OK, Json::from(res)))
}

#[derive(Deserialize)]
struct CreateTodo {
    name: String,
}
async fn add_todo(
    State(state): State<ArcState>,
    Json(CreateTodo { name }): Json<CreateTodo>,
) -> Result<impl IntoResponse, RequestError> {
    let id = sqlx::query("insert into todos (name) values (?)")
        .bind(name)
        .execute(&state.db_pool)
        .await?
        .last_insert_id();
    Ok((StatusCode::OK, Json::from(id)))
}

#[derive(Deserialize)]
struct DeleteTodo {
    id: u32,
}
async fn delete_todo(
    State(state): State<ArcState>,
    Json(DeleteTodo { id }): Json<DeleteTodo>,
) -> Result<impl IntoResponse, RequestError> {
    sqlx::query("delete from todos where id = ?")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    Ok(StatusCode::OK)
}

async fn create_todo(State(state): State<ArcState>) -> Result<impl IntoResponse, RequestError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS 
        todos
        (id INT PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255) NOT NULL);
    "#,
    )
    .execute(&state.db_pool)
    .await?;
    Ok(StatusCode::OK)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();
    let _ = dotenvy::dotenv().inspect_err(|e| tracing::warn!("Dotenvy init error: {e}"));

    let user = std::env::var("DB_USER").expect("DB_USER env not found");
    let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD env not found");
    let address = std::env::var("DB_ADDRESS").expect("DB_ADDRESS env not found");
    let port = std::env::var("DB_PORT").expect("DB_PORT env not found");
    let database = std::env::var("DB_DATABASE").expect("DB_DATABASE env not found");
    let collector_url = std::env::var("COLLECTOR_URL").ok();
    let app_port = std::env::var("PORT")
        .expect("PORT env not found")
        .parse::<u16>()
        .unwrap();

    let connection_string = format!(
        "mysql://{}:{}@{}:{}/{}",
        user, password, address, port, database
    );
    tracing::debug!(connection_string, "Connection string");
    let pool = connect_to_database(&connection_string).await.unwrap();

    let mut profiler: Option<Arc<Profiler>> = None;
    if let Ok(res) = std::env::var("USE_PROFILING")
        && res == "true"
    {
        let address = std::env::var("PYROSCOPE_URL").inspect_err(|_| {
            tracing::warn!("Profiling is enabled but PYROSCOPE_URL env is not set")
        });
        if let Ok(address) = address {
            profiler = Profiler::new(&address, "rust_server")
                .inspect_err(|e| tracing::error!("Failed to start profiler with error: {e}"))
                .ok()
                .inspect(|_| tracing::info!("Started profiling"))
                .map(Arc::new);
        }
    }
    let app_state = AppState {
        db_pool: pool,
        profiler: profiler.clone(),
    };
    let mut router = Router::new()
        .route("/", get(get_todos).post(add_todo).delete(delete_todo))
        .route("/create", post(create_todo))
        .layer(middleware::from_fn(http_metrics_middleware))
        .with_state(Arc::new(app_state));
    if let Some(profiler) = profiler.clone() {
        router = router.layer(middleware::from_fn_with_state(
            profiler,
            profiling_middleware,
        ));
    }
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), app_port))
        .await
        .unwrap();
    tracing::info!("Listening on port: {}", app_port);
    // Инициализация метрик
    let metrics_provider = {
        let use_metrics = var("USE_METRICS").is_ok_and(|t| t == "true");
        if use_metrics {
            if let Some(collector_url) = collector_url {
                let metrics = init_metrics(&format!("{}/metrics", collector_url)).unwrap();
                Some(metrics)
            } else {
                tracing::warn!("USE_METRICS env found, but COLLECTOR_URL env wasnt given");
                None
            }
        } else {
            None
        }
    };
    let _otel_guard = OtelGuard {
        meter_provider: metrics_provider,
    };
    let main_span = info_span!("app");
    let _g = main_span.enter();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    // Задача для ожидания сигналов
    tokio::spawn(async move {
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        let mut sigterm = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = sigint.recv() => {
                tracing::info!("Received SIGINT");
            }
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM");
            }
        }

        tx.send(()).await.ok();
    });

    // Основной сервер
    let _ = axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            rx.recv().await;
            tracing::info!("Shutting down gracefully...");
        })
        .await
        .inspect_err(|e| tracing::error!(?e));

    if let Some(profiler) = profiler
        && let Some(inner) = Arc::into_inner(profiler)
    {
        tracing::info!("Sending profile");
        inner.stop().unwrap()
    }
    drop(_g);
    drop(_otel_guard);
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn check_pipeline() {
        print!("Works");
        assert!(1_i32.is_negative());
    }
}
