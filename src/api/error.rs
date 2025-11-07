use crate::{
    core::{auth::AuthError, core::CoreMessage},
    repo::error::RepoError,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tackle_box::connection::{MatchPlayerResponse, ProcessGameRequest};
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // 认证/权限错误
    #[error("Unauthorized:")]
    Unauthorized(#[from] AuthError),

    // 业务逻辑错误
    #[error("Input validation failed: {0}")]
    Validation(String),

    // 数据库错误 (通常是底层错误)
    #[error("Database error")]
    Database(#[from] RepoError),

    // JWT 错误 (从 jsonwebtoken 库传递上来)
    #[error("Token processing error")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    // 内部服务器错误 (作为捕获所有未处理错误的默认值)
    #[error("Internal server error")]
    Internal(String),

    #[error("Serde error")]
    Serde(#[from] serde_json::Error),

    #[error("gRPC error")]
    Communication(#[from] tonic::Status),

    #[error("grpc connect error")]
    Grpc(#[from] tonic::transport::Error),

    #[error("uuid parse error")]
    UuidParse(#[from] uuid::Error),

    #[error("Time out or send error")]
    SendError(#[from] SendError<CoreMessage>),

    #[error("Time out or send error")]
    SendErrorTonic(#[from] SendError<ProcessGameRequest>),

    #[error("Time out or send error")]
    SendErrorClient(#[from] SendError<MatchPlayerResponse>),

    #[error("Match Abort")]
    MatchAborted(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 🌟 2. 根据错误类型，自动决定 HTTP 状态码和前端消息
        let (status, client_message) = match &self {
            // -- 客户端可见的错误 --
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.to_string()),
            AppError::Validation(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),

            // -- 内部错误（对用户隐藏细节）--
            AppError::Database(_)
            | AppError::Jwt(_)
            | AppError::Internal(_)
            | AppError::Serde(_)
            | AppError::Communication(_)
            | AppError::Grpc(_)
            | AppError::UuidParse(_)
            | AppError::SendErrorTonic(_)
            | AppError::SendError(_)
            | AppError::SendErrorClient(_)
            | AppError::MatchAborted(_) => {
                // 打印到服务器日志，以便后端排查
                eprintln!("Internal Error: {:?}", self);
                // 返回一个通用的 500 错误给前端
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Server experienced an unexpected error.".to_string(),
                )
            }
        };

        // 🌟 3. 格式化为 JSON 响应
        let body = Json(json!({
            "success": false,
            "message": client_message,
        }));

        (status, body).into_response()
    }
}
