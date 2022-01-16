use std::fmt::Display;

use actix_web::{
    error::{InternalError, JsonPayloadError},
    http::StatusCode,
    web::JsonConfig,
    ResponseError,
};

#[derive(Debug)]
pub enum ApiError {
    JsonError { cause: String },
}
impl Display for ApiError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::JsonError { cause } => fmt.write_str(cause),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::JsonError { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl ApiError {
    pub fn json_error(cfg: JsonConfig) -> JsonConfig {
        cfg.limit(4096).error_handler(|err: JsonPayloadError, _| {
            InternalError::from_response(
                format!("JSON error: {}", err),
                ApiError::JsonError {
                    cause: format!("{}", err),
                }
                .error_response(),
            )
            .into()
        })
    }
}
