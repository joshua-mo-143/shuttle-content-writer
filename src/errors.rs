use async_openai::error::OpenAIError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("OpenAI error: {0}")]
    OpenAI(#[from] OpenAIError),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("De/serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::OpenAI(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            Self::Reqwest(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            Self::SerdeJson(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        println!("An error happened: {body}");

        (status, body).into_response()
    }
}
