use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use crate::db::Database;
use crate::models::*;

pub async fn list_users(
    State(db): State<Arc<Database>>,
) -> Result<Json<Vec<UserRecord>>, (StatusCode, String)> {
    db.get_all_users()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn create_user(
    State(db): State<Arc<Database>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserRecord>), (StatusCode, String)> {
    db.create_user(&req.username, req.is_admin)
        .map(|u| (StatusCode::CREATED, Json(u)))
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

pub async fn delete_user(
    State(db): State<Arc<Database>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    match db.delete_user(id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn regenerate_token(
    State(db): State<Arc<Database>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match db.regenerate_token(id) {
        Ok(Some(token)) => Ok(Json(serde_json::json!({"token": token}))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn stats(
    State(db): State<Arc<Database>>,
) -> Result<Json<AdminStats>, (StatusCode, String)> {
    db.get_admin_stats()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
