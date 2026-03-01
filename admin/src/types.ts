/** Matches Rust: AdminStats */
export interface AdminStats {
  total_users: number;
  total_reports: number;
  total_tokens_processed: number;
  active_users_5h: number;
  active_users_7d: number;
  db_size_bytes: number;
}

/** Matches Rust: UserRecord */
export interface UserRecord {
  id: number;
  username: string;
  is_admin: boolean;
  created_at: string;
  token?: string;
}

/** Matches Rust: UserSummary */
export interface UserSummary {
  username: string;
  total_input_tokens: number;
  total_output_tokens: number;
  total_cache_read_tokens: number;
  total_cache_creation_tokens: number;
  total_messages: number;
  total_tool_uses: number;
  report_count: number;
  last_active: string;
  latest_percent_5h: number | null;
  latest_percent_7d: number | null;
}

/** Matches Rust: SummaryResponse */
export interface SummaryResponse {
  window_5h: UserSummary[];
  window_7d: UserSummary[];
}

/** Matches Rust: UserInfo */
export interface UserInfo {
  username: string;
  last_active: string;
  total_reports: number;
}

/** Matches Rust: HourlyUsage */
export interface HourlyUsage {
  hour: string;
  username: string;
  input_tokens: number;
  output_tokens: number;
}
