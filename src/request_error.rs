use axum::{http::StatusCode, response::IntoResponse};

pub enum RequestError {
    DbConnectionError,
}
impl IntoResponse for RequestError {
    fn into_response(self) -> axum::response::Response {
        match &self {
            RequestError::DbConnectionError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}
impl From<mysql::Error> for RequestError {
    fn from(_: mysql::Error) -> Self {
        Self::DbConnectionError
    }
}
