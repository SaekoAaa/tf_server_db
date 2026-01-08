use std::{error::Error, net::Ipv4Addr, sync::Arc, thread::sleep, time::Duration};

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use mysql::{
    Conn, Opts, Pool,
    prelude::{FromRow, Queryable},
};
use serde::Serialize;

use crate::request_error::RequestError;
mod request_error;
struct AppState {
    db_pool: Pool,
}
type ArcState = Arc<AppState>;
#[derive(thiserror::Error, Debug)]
enum ConnectionError {
    #[error("Failed to parse opts: {0}")]
    ParseOptsError(#[from] mysql::UrlError),
    #[error("Failed to connect to database")]
    ConnectFailed,
}

fn connect_to_database(mysql_connection_string: &str) -> Result<Pool, ConnectionError> {
    let retries = 3;
    for retry in 0..retries {
        let opts = Opts::try_from(mysql_connection_string)?;
        match Pool::new(opts) {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                eprintln!("Error connecting to database: {e} (retries {retry}/{retries})");
                sleep(Duration::from_secs(5));
            }
        }
    }
    Err(ConnectionError::ConnectFailed)
}

#[derive(FromRow, Serialize)]
struct Todo {
    id: i32,
    name: String,
}
async fn get_todos(State(state): State<ArcState>) -> Result<impl IntoResponse, RequestError> {
    let mut connection = state.db_pool.get_conn()?;
    let res: Vec<Todo> = connection.query(
        r#"
        select id, name from todos
"#,
    )?;
    Ok(Json::from(res))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().inspect_err(|e| eprintln!("{e}"));

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
    let pool = connect_to_database(&connection_string).unwrap();

    let app_state = AppState { db_pool: pool };
    let router = Router::new()
        .route("/", get(get_todos))
        .with_state(Arc::new(app_state));
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), app_port))
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap()
}
