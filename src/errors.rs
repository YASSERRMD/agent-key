//! Centralized error handling for AgentKey backend.
//!
//! Provides a unified error type that integrates with Actix-web's error handling.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

/// API error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Error body details
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Unified application error type
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Authentication required: {0}")]
    Unauthorized(String),

    #[error("Access denied: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl ApiError {
    /// Get the error code string for the response
    fn error_code(&self) -> &str {
        match self {
            ApiError::Unauthorized(_) => "UNAUTHORIZED",
            ApiError::Forbidden(_) => "FORBIDDEN",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::Conflict(_) => "CONFLICT",
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::InternalError(_) => "INTERNAL_ERROR",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::RedisError(_) => "REDIS_ERROR",
            ApiError::EncryptionError(_) => "ENCRYPTION_ERROR",
            ApiError::JwtError(_) => "JWT_ERROR",
            ApiError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
        }
    }

    /// Extract the message from the error
    fn message(&self) -> String {
        match self {
            ApiError::Unauthorized(msg)
            | ApiError::Forbidden(msg)
            | ApiError::NotFound(msg)
            | ApiError::Conflict(msg)
            | ApiError::BadRequest(msg)
            | ApiError::ValidationError(msg)
            | ApiError::InternalError(msg)
            | ApiError::DatabaseError(msg)
            | ApiError::RedisError(msg)
            | ApiError::EncryptionError(msg)
            | ApiError::JwtError(msg)
            | ApiError::ServiceUnavailable(msg) => msg.clone(),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::BadRequest(_) | ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalError(_)
            | ApiError::DatabaseError(_)
            | ApiError::RedisError(_)
            | ApiError::EncryptionError(_)
            | ApiError::JwtError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let response = ErrorResponse {
            error: ErrorBody {
                code: self.error_code().to_string(),
                message: self.message(),
                details: None,
            },
            timestamp: chrono::Utc::now(),
        };

        HttpResponse::build(self.status_code()).json(response)
    }
}

// Conversion implementations for common error types

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "Database error occurred");
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(err: redis::RedisError) -> Self {
        tracing::error!(error = %err, "Redis error occurred");
        ApiError::RedisError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        tracing::warn!(error = %err, "JWT error occurred");
        ApiError::JwtError(err.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!(error = %err, "Internal error occurred");
        ApiError::InternalError(err.to_string())
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::ValidationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ApiError::Unauthorized("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::BadRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::InternalError("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            ApiError::Unauthorized("test".to_string()).error_code(),
            "UNAUTHORIZED"
        );
        assert_eq!(
            ApiError::DatabaseError("test".to_string()).error_code(),
            "DATABASE_ERROR"
        );
    }
}
