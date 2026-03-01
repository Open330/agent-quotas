use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Serialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub total_reports: i64,
    pub total_tokens_processed: i64,
    pub active_users_5h: i64,
    pub active_users_7d: i64,
    pub db_size_bytes: i64,
}

#[derive(Debug, Deserialize)]
pub struct UsageReport {
    pub username: String,
    pub session_id: String,
    pub report_id: String,
    pub timestamp: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    #[serde(default)]
    pub cache_read_input_tokens: i64,
    #[serde(default)]
    pub cache_creation_input_tokens: i64,
    #[serde(default)]
    pub message_count: i64,
    #[serde(default)]
    pub tool_use_count: i64,
    #[serde(default)]
    pub usage_percent_5h: Option<f64>,
    #[serde(default)]
    pub usage_percent_7d: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserSummary {
    pub username: String,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cache_creation_tokens: i64,
    pub total_messages: i64,
    pub total_tool_uses: i64,
    pub report_count: i64,
    pub last_active: String,
    pub latest_percent_5h: Option<f64>,
    pub latest_percent_7d: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct SummaryResponse {
    pub window_5h: Vec<UserSummary>,
    pub window_7d: Vec<UserSummary>,
}

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    pub window: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub last_active: String,
    pub total_reports: i64,
}
