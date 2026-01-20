use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Database: {0}")]
    DbConnectionError(#[from] sqlx::Error),
}
impl IntoResponse for RequestError {
    fn into_response(self) -> axum::response::Response {
        match &self {
            RequestError::DbConnectionError(e) => {
                tracing::error!("Error during request handling: {e}",);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}
