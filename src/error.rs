use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;

pub type HttpResult<T = ()> = Result<T, HttpError>;

// Make our own error that wraps `anyhow::Error`.
pub struct HttpError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for HttpError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
