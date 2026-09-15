use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Validation(String),
    Conflict(String),
    VersionConflict(String),
    Unauthorized(String),
    Internal(String),
    Database(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    success: bool,
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation Error: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::VersionConflict(msg) => write!(f, "Version Conflict: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Error: {}", msg),
            AppError::Database(msg) => write!(f, "Database Error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status_code, code, message) = match self {
            AppError::NotFound(msg) => (
                actix_web::http::StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg.clone(),
            ),
            AppError::Validation(msg) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg.clone(),
            ),
            AppError::Conflict(msg) => (
                actix_web::http::StatusCode::CONFLICT,
                "CONFLICT",
                msg.clone(),
            ),
            AppError::VersionConflict(msg) => (
                actix_web::http::StatusCode::CONFLICT,
                "VERSION_CONFLICT",
                msg.clone(),
            ),
            AppError::Unauthorized(msg) => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                msg.clone(),
            ),
            AppError::Internal(msg) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
            ),
            AppError::Database(msg) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                msg.clone(),
            ),
        };

        let response = ErrorResponse {
            success: false,
            error: ErrorDetail {
                code: code.to_string(),
                message,
            },
        };

        HttpResponse::build(status_code).json(response)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.message().contains("Duplicate entry") {
                    AppError::Conflict("Resource already exists".to_string())
                } else {
                    AppError::Database(db_err.message().to_string())
                }
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}
