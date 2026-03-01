use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use rand::Rng;
use std::sync::Arc;
use crate::db::Database;
use crate::models::ReportResponse;

const PAT_LENGTH: usize = 48;
const PAT_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub fn generate_pat() -> String {
    let mut rng = rand::rng();
    (0..PAT_LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..PAT_CHARS.len());
            PAT_CHARS[idx] as char
        })
        .collect()
}

fn extract_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

/// Middleware: require valid PAT for API endpoints
pub async fn require_auth(
    State(db): State<Arc<Database>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ReportResponse>)> {
    let token = extract_token(request.headers());
    match token {
        Some(t) if db.validate_token(t).is_ok_and(|v| v) => Ok(next.run(request).await),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(ReportResponse {
                status: "unauthorized".to_string(),
                message: Some("Invalid or missing Bearer token".to_string()),
            }),
        )),
    }
}

/// Middleware: require admin PAT
pub async fn require_admin(
    State(db): State<Arc<Database>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ReportResponse>)> {
    let token = extract_token(request.headers());
    match token {
        Some(t) if db.is_admin_token(t).is_ok_and(|v| v) => Ok(next.run(request).await),
        Some(_) => Err((
            StatusCode::FORBIDDEN,
            Json(ReportResponse {
                status: "forbidden".to_string(),
                message: Some("Admin access required".to_string()),
            }),
        )),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(ReportResponse {
                status: "unauthorized".to_string(),
                message: Some("Invalid or missing Bearer token".to_string()),
            }),
        )),
    }
}
