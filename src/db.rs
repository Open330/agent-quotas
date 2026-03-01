use std::sync::Mutex;
use rusqlite::{params, Connection};
use crate::models::{UsageReport, UserSummary, UserInfo, UserRecord, AdminStats};
use crate::auth::generate_pat;

pub struct Database {
    conn: Mutex<Connection>,
    path: String,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
        let db = Database {
            conn: Mutex::new(conn),
            path: path.to_string(),
        };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS usage_reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                report_id TEXT NOT NULL UNIQUE,
                username TEXT NOT NULL,
                session_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                model TEXT NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                cache_read_input_tokens INTEGER NOT NULL DEFAULT 0,
                cache_creation_input_tokens INTEGER NOT NULL DEFAULT 0,
                message_count INTEGER NOT NULL DEFAULT 0,
                tool_use_count INTEGER NOT NULL DEFAULT 0,
                usage_percent_5h REAL,
                usage_percent_7d REAL,
                received_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_usage_username ON usage_reports(username);
            CREATE INDEX IF NOT EXISTS idx_usage_timestamp ON usage_reports(timestamp);
            CREATE INDEX IF NOT EXISTS idx_usage_session ON usage_reports(session_id);

            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                token TEXT NOT NULL UNIQUE,
                is_admin INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );"
        )?;
        Ok(())
    }

    pub fn ensure_admin(&self) {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .unwrap_or(0);
        if count == 0 {
            let token = generate_pat();
            conn.execute(
                "INSERT INTO users (username, token, is_admin) VALUES (?1, ?2, 1)",
                params!["admin", &token],
            )
            .expect("Failed to create default admin user");
            println!("=== Default admin user created ===");
            println!("Username: admin");
            println!("Token:    {}", token);
            println!("==================================");
        }
    }

    pub fn validate_token(&self, token: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE token = ?1",
            params![token],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn is_admin_token(&self, token: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE token = ?1 AND is_admin = 1",
            params![token],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn get_all_users(&self) -> Result<Vec<UserRecord>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, is_admin, created_at FROM users ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(UserRecord {
                id: row.get(0)?,
                username: row.get(1)?,
                is_admin: row.get::<_, i32>(2)? != 0,
                created_at: row.get(3)?,
                token: None,
            })
        })?;
        rows.collect()
    }

    pub fn create_user(&self, username: &str, is_admin: bool) -> Result<UserRecord, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let token = generate_pat();
        conn.execute(
            "INSERT INTO users (username, token, is_admin) VALUES (?1, ?2, ?3)",
            params![username, &token, is_admin as i32],
        )?;
        let id = conn.last_insert_rowid();
        let created_at: String = conn.query_row(
            "SELECT created_at FROM users WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(UserRecord {
            id,
            username: username.to_string(),
            is_admin,
            created_at,
            token: Some(token),
        })
    }

    pub fn delete_user(&self, id: i64) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM users WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn regenerate_token(&self, id: i64) -> Result<Option<String>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let token = generate_pat();
        let affected = conn.execute(
            "UPDATE users SET token = ?1 WHERE id = ?2",
            params![&token, id],
        )?;
        if affected > 0 {
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    pub fn get_username_by_token(&self, token: &str) -> Result<Option<String>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT username FROM users WHERE token = ?1")?;
        let mut rows = stmt.query(params![token])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn get_admin_stats(&self) -> Result<AdminStats, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let total_users: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users", [], |row| row.get(0),
        )?;
        let total_reports: i64 = conn.query_row(
            "SELECT COUNT(*) FROM usage_reports", [], |row| row.get(0),
        )?;
        let total_tokens_processed: i64 = conn.query_row(
            "SELECT COALESCE(SUM(input_tokens + output_tokens), 0) FROM usage_reports",
            [], |row| row.get(0),
        )?;
        let active_users_5h: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT username) FROM usage_reports WHERE timestamp > datetime('now', '-5 hours')",
            [], |row| row.get(0),
        )?;
        let active_users_7d: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT username) FROM usage_reports WHERE timestamp > datetime('now', '-7 days')",
            [], |row| row.get(0),
        )?;
        let db_size_bytes: i64 = std::fs::metadata(&self.path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);
        Ok(AdminStats {
            total_users,
            total_reports,
            total_tokens_processed,
            active_users_5h,
            active_users_7d,
            db_size_bytes,
        })
    }

    pub fn insert_report(&self, report: &UsageReport) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let result = conn.execute(
            "INSERT OR IGNORE INTO usage_reports (report_id, username, session_id, timestamp, model, input_tokens, output_tokens, cache_read_input_tokens, cache_creation_input_tokens, message_count, tool_use_count, usage_percent_5h, usage_percent_7d)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                report.report_id,
                report.username,
                report.session_id,
                report.timestamp,
                report.model,
                report.input_tokens,
                report.output_tokens,
                report.cache_read_input_tokens,
                report.cache_creation_input_tokens,
                report.message_count,
                report.tool_use_count,
                report.usage_percent_5h,
                report.usage_percent_7d,
            ],
        )?;
        Ok(result > 0)
    }

    pub fn get_user_summaries(&self, window: &str) -> Result<Vec<UserSummary>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let time_filter = match window {
            "5h" => "datetime('now', '-5 hours')",
            "24h" => "datetime('now', '-24 hours')",
            "7d" => "datetime('now', '-7 days')",
            _ => "datetime('1970-01-01')",
        };
        let query = format!(
            "SELECT r.username,
                    COALESCE(SUM(r.input_tokens), 0),
                    COALESCE(SUM(r.output_tokens), 0),
                    COALESCE(SUM(r.cache_read_input_tokens), 0),
                    COALESCE(SUM(r.cache_creation_input_tokens), 0),
                    COALESCE(SUM(r.message_count), 0),
                    COALESCE(SUM(r.tool_use_count), 0),
                    COUNT(*),
                    MAX(r.timestamp),
                    latest.pct_5h,
                    latest.pct_7d
             FROM usage_reports r
             LEFT JOIN (
                 SELECT username,
                        usage_percent_5h as pct_5h,
                        usage_percent_7d as pct_7d
                 FROM usage_reports
                 WHERE usage_percent_5h IS NOT NULL OR usage_percent_7d IS NOT NULL
                 GROUP BY username
                 HAVING timestamp = MAX(timestamp)
             ) latest ON r.username = latest.username
             WHERE r.timestamp > {}
             GROUP BY r.username
             ORDER BY COALESCE(latest.pct_5h, 0) DESC, (SUM(r.input_tokens) + SUM(r.output_tokens)) DESC",
            time_filter
        );
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            Ok(UserSummary {
                username: row.get(0)?,
                total_input_tokens: row.get(1)?,
                total_output_tokens: row.get(2)?,
                total_cache_read_tokens: row.get(3)?,
                total_cache_creation_tokens: row.get(4)?,
                total_messages: row.get(5)?,
                total_tool_uses: row.get(6)?,
                report_count: row.get(7)?,
                last_active: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                latest_percent_5h: row.get(9)?,
                latest_percent_7d: row.get(10)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_users(&self) -> Result<Vec<UserInfo>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT username, MAX(timestamp) as last_active, COUNT(*) as total_reports
             FROM usage_reports
             GROUP BY username
             ORDER BY last_active DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(UserInfo {
                username: row.get(0)?,
                last_active: row.get(1)?,
                total_reports: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_hourly_usage(&self, user: Option<&str>) -> Result<Vec<(String, String, i64, i64)>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        if let Some(username) = user {
            let mut stmt = conn.prepare(
                "SELECT strftime('%Y-%m-%dT%H:00:00', timestamp) as hour,
                        username,
                        COALESCE(SUM(input_tokens), 0),
                        COALESCE(SUM(output_tokens), 0)
                 FROM usage_reports
                 WHERE timestamp > datetime('now', '-7 days') AND username = ?1
                 GROUP BY hour, username
                 ORDER BY hour"
            )?;
            let rows = stmt.query_map(params![username], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?;
            rows.collect()
        } else {
            let mut stmt = conn.prepare(
                "SELECT strftime('%Y-%m-%dT%H:00:00', timestamp) as hour,
                        username,
                        COALESCE(SUM(input_tokens), 0),
                        COALESCE(SUM(output_tokens), 0)
                 FROM usage_reports
                 WHERE timestamp > datetime('now', '-7 days')
                 GROUP BY hour, username
                 ORDER BY hour"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?;
            rows.collect()
        }
    }
}
