use crate::{api::error::AppError, core::auth::AuthError};
use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone)]
pub struct Claims {
    pub user_id: Uuid,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

const JWT_SECRET: &[u8] = b"hello world";

#[derive(Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

impl<S: Sync> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. 尝试从 Authorization 头部获取 Token 字符串
        let header = parts
            .headers
            .get("Authorization")
            .ok_or(AppError::Validation("No found token".to_string()))?;

        let token_value = header
            .to_str()
            .map_err(|_| AppError::Validation("Invaid token".to_string()))?;

        // 2. 检查格式是否为 "Bearer <Token>"
        let token = token_value
            .strip_prefix("Bearer ")
            .ok_or(AppError::Validation("Invaid token".to_string()))?;

        let claims = check_jwt(token)?;

        // 4. 返回成功解析的用户 ID
        Ok(AuthenticatedUser {
            user_id: claims.user_id,
        })
    }
}

pub async fn generate_jwt(username: &String, user_id: Uuid) -> Result<String, AuthError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize;
    let exp = now + 3600 * 24 * 30;
    let claims = Claims {
        user_id,
        username: username.clone(),
        exp,
        iat: now,
    };
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(JWT_SECRET);
    encode(&header, &claims, &encoding_key).map_err(|_| AuthError::JWTError)
}

pub fn check_jwt(token: &str) -> Result<Claims, AuthError> {
    let decoding_key = DecodingKey::from_secret(JWT_SECRET);
    let validation = Validation::new(Algorithm::HS256);

    let claims = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| {
            // 细化错误类型
            if e.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                AuthError::JWTError
            } else {
                AuthError::JWTError
            }
        })?
        .claims;
    Ok(claims)
}

// /*
//  *  Ws Connection Athentication
//  */
// #[derive(Deserialize, Serialize)]
// pub struct WsClaims {
//     pub user_id: Uuid,
//     pub agent_id: Uuid,
//     pub exp: usize,
//     pub iat: usize,
// }

// #[derive(Deserialize)]
// pub struct AuthenticatedAgent {
//     pub user_id: Uuid,
//     pub agent_id: Uuid,
// }

// impl<S: Sync> FromRequestParts<S> for AuthenticatedAgent {
//     type Rejection = AppError;

//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         // 1. 尝试从 Authorization 头部获取 Token 字符串
//         let header = parts
//             .headers
//             .get("Authorization")
//             .ok_or(AppError::Validation("No found token".to_string()))?;

//         let token_value = header
//             .to_str()
//             .map_err(|_| AppError::Validation("Invaid token".to_string()))?;

//         // 2. 检查格式是否为 "Bearer <Token>"
//         let token = token_value
//             .strip_prefix("Bearer ")
//             .ok_or(AppError::Validation("Invaid token".to_string()))?;

//         let claims = check_ws_jwt(token).await?;

//         // 4. 返回成功解析的用户 ID
//         Ok(AuthenticatedAgent {
//             user_id: claims.user_id,
//             agent_id: claims.agent_id,
//         })
//     }
// }

// pub async fn check_ws_jwt(token: &str) -> Result<WsClaims, AuthError> {
//     let decoding_key = DecodingKey::from_secret(JWT_SECRET);
//     let validation = Validation::new(Algorithm::HS256);

//     let claims = decode::<WsClaims>(token, &decoding_key, &validation)
//         .map_err(|e| {
//             // 细化错误类型
//             if e.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
//                 AuthError::JWTError
//             } else {
//                 AuthError::JWTError
//             }
//         })?
//         .claims;
//     Ok(claims)
// }

// pub async fn gen_agent_key(user_id: Uuid, agent_id: Uuid) -> Result<String, AuthError> {
//     let now = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .expect("Time went backwards")
//         .as_secs() as usize;
//     let exp = now + 3600 * 24;
//     let claims = WsClaims {
//         user_id,
//         agent_id,
//         exp,
//         iat: now,
//     };
//     let header = Header::default();
//     let encoding_key = EncodingKey::from_secret(JWT_SECRET);
//     encode(&header, &claims, &encoding_key).map_err(|_| AuthError::JWTError)
// }
