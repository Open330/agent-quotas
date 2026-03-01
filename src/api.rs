use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use crate::db::Database;
use crate::models::*;

#[derive(Debug, Serialize)]
pub struct HourlyUsage {
    pub hour: String,
    pub username: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

// POST /api/report - receive usage report from hook
pub async fn report_handler(
    State(db): State<Arc<Database>>,
    Json(report): Json<UsageReport>,
) -> Result<(StatusCode, Json<ReportResponse>), (StatusCode, Json<ReportResponse>)> {
    match db.insert_report(&report) {
        Ok(true) => Ok((
            StatusCode::CREATED,
            Json(ReportResponse {
                status: "created".to_string(),
                message: None,
            }),
        )),
        Ok(false) => Err((
            StatusCode::CONFLICT,
            Json(ReportResponse {
                status: "duplicate".to_string(),
                message: Some("Report already exists".to_string()),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ReportResponse {
                status: "error".to_string(),
                message: Some(e.to_string()),
            }),
        )),
    }
}

// GET /api/users - list all users
pub async fn users_handler(
    State(db): State<Arc<Database>>,
) -> Result<Json<Vec<UserInfo>>, (StatusCode, String)> {
    db.get_users()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// GET /api/usage - query usage data with window parameter
pub async fn usage_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<UsageQuery>,
) -> Result<Json<Vec<UserSummary>>, (StatusCode, String)> {
    let window = params.window.as_deref().unwrap_or("5h");
    // If specific user requested, filter results
    let summaries = db.get_user_summaries(window)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let filtered = if let Some(ref user) = params.user {
        summaries.into_iter().filter(|s| &s.username == user).collect()
    } else {
        summaries
    };
    Ok(Json(filtered))
}

// GET /api/summary - per-user aggregated summary for dashboard (both windows)
pub async fn summary_handler(
    State(db): State<Arc<Database>>,
) -> Result<Json<SummaryResponse>, (StatusCode, String)> {
    let window_5h = db.get_user_summaries("5h")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let window_7d = db.get_user_summaries("7d")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(SummaryResponse { window_5h, window_7d }))
}

// GET /api/hourly - hourly usage for timeline chart
pub async fn hourly_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<UsageQuery>,
) -> Result<Json<Vec<HourlyUsage>>, (StatusCode, String)> {
    let data = db.get_hourly_usage(params.user.as_deref())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let result: Vec<HourlyUsage> = data.into_iter().map(|(hour, username, input, output)| {
        HourlyUsage { hour, username, input_tokens: input, output_tokens: output }
    }).collect();
    Ok(Json(result))
}
