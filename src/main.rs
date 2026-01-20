use std::{error::Error, net::Ipv4Addr, sync::Arc, thread::sleep, time::Duration};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use tracing::level_filters::LevelFilter;

use crate::request_error::RequestError;
mod request_error;
struct AppState {
    db_pool: sqlx::MySqlPool,
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
    let res: Vec<Todo> = sqlx::query_as(
        r#"
        select id, name from todos
"#,
    )
    .fetch_all(&state.db_pool)
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
        .with_max_level(LevelFilter::DEBUG)
        .init();
    dotenvy::dotenv().inspect_err(|e| tracing::error!("Dotenvy init error: {e}"));

    // let user = String::from("root");
    let user = std::env::var("DB_USER").expect("DB_USER env not found");
    let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD env not found");
    let address = std::env::var("DB_ADDRESS").expect("DB_ADDRESS env not found");
    let port = std::env::var("DB_PORT").expect("DB_PORT env not found");
    let database = std::env::var("DB_DATABASE").expect("DB_DATABASE env not found");
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

    let app_state = AppState { db_pool: pool };
    let router = Router::new()
        .route("/", get(get_todos).post(add_todo).delete(delete_todo))
        .route("/create", post(create_todo))
        .with_state(Arc::new(app_state));
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), app_port))
        .await
        .unwrap();
    tracing::info!("Listening on port: {}", app_port);

    axum::serve(listener, router).await.unwrap()
}
