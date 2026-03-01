import { api } from '../api.ts';
import { usePolling } from '../hooks.ts';
import type { SummaryResponse, UserSummary, AdminStats } from '../types.ts';

const CIRC = 2 * Math.PI * 38;

function percentColor(pct: number | null): string {
  if (pct === null || pct === undefined) return 'null';
  if (pct < 50) return 'green';
  if (pct < 75) return 'yellow';
  if (pct < 90) return 'orange';
  return 'red';
}

function badgeText(pct: number | null): string {
  if (pct === null || pct === undefined) return 'N/A';
  if (pct < 50) return 'OK';
  if (pct < 75) return 'MODERATE';
  if (pct < 90) return 'HIGH';
  return 'CRITICAL';
}

function relativeTime(isoStr: string): string {
  if (!isoStr) return 'never';
  const diff = Date.now() - new Date(isoStr).getTime();
  const m = Math.floor(diff / 60000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
  if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
  return n.toLocaleString();
}

function formatBytes(bytes: number): string {
  if (bytes >= 1_073_741_824) return (bytes / 1_073_741_824).toFixed(1) + ' GB';
  if (bytes >= 1_048_576) return (bytes / 1_048_576).toFixed(1) + ' MB';
  if (bytes >= 1_024) return (bytes / 1_024).toFixed(1) + ' KB';
  return bytes + ' B';
}

function CircularGauge({ pct, label }: { pct: number | null; label: string }) {
  const color = percentColor(pct);
  const displayPct = pct === null || pct === undefined ? '--' : pct.toFixed(1) + '%';
  const offset = pct === null || pct === undefined
    ? CIRC
    : CIRC * (1 - Math.min(pct, 100) / 100);

  return (
    <div className="gauge-wrap">
      <div className="gauge-label">{label}</div>
      <div className="circular-progress">
        <svg viewBox="0 0 90 90">
          <circle className="track" cx="45" cy="45" r="38" />
          <circle
            className={`fill stroke-${color}`}
            cx="45"
            cy="45"
            r="38"
            strokeDasharray={CIRC.toFixed(2)}
            strokeDashoffset={offset.toFixed(2)}
          />
        </svg>
        <div className="center-text">
          <span className={`gauge-percent color-${color}`}>{displayPct}</span>
          <span className="gauge-unit">of quota</span>
        </div>
      </div>
    </div>
  );
}

function UserCard({ user }: { user: UserSummary & { _7d?: UserSummary } }) {
  const pct5h = user.latest_percent_5h;
  const pct7d = user.latest_percent_7d ?? user._7d?.latest_percent_7d ?? null;
  const worstPct = Math.max(pct5h ?? -1, pct7d ?? -1);
  const worstColor = worstPct < 0 ? 'null' : percentColor(worstPct);
  const initial = (user.username || '?')[0].toUpperCase();

  return (
    <div className="user-card">
      <div className="user-card-header">
        <div className="user-avatar">{initial}</div>
        <div className="user-info">
          <div className="user-name">{user.username}</div>
          <div className="user-meta">Active {relativeTime(user.last_active)}</div>
        </div>
        <span className={`badge badge-${worstColor === 'null' ? 'muted' : worstColor}`}>
          {badgeText(worstPct < 0 ? null : worstPct)}
        </span>
      </div>

      <div className="gauges-row">
        <CircularGauge pct={pct5h} label="5h window" />
        <CircularGauge pct={pct7d} label="7d window" />
      </div>

      <div className="token-stats">
        <div className="token-stat">
          <span className="token-stat-label">Input tokens</span>
          <span className="token-stat-value">{user.total_input_tokens.toLocaleString()}</span>
        </div>
        <div className="token-stat">
          <span className="token-stat-label">Output tokens</span>
          <span className="token-stat-value">{user.total_output_tokens.toLocaleString()}</span>
        </div>
        <div className="token-stat">
          <span className="token-stat-label">Cache read</span>
          <span className="token-stat-value">{user.total_cache_read_tokens.toLocaleString()}</span>
        </div>
        <div className="token-stat">
          <span className="token-stat-label">Cache created</span>
          <span className="token-stat-value">{user.total_cache_creation_tokens.toLocaleString()}</span>
        </div>
        <div className="token-stat">
          <span className="token-stat-label">Messages</span>
          <span className="token-stat-value highlight">{user.total_messages.toLocaleString()}</span>
        </div>
        <div className="token-stat">
          <span className="token-stat-label">Tool uses</span>
          <span className="token-stat-value highlight">{user.total_tool_uses.toLocaleString()}</span>
        </div>
      </div>
    </div>
  );
}

export default function Stats() {
  const { data: summary, loading: summaryLoading, error: summaryError } = usePolling<SummaryResponse>(
    () => api.getSummary(),
    30000,
  );

  // Try admin stats, but gracefully degrade if the endpoint doesn't exist
  const { data: adminStats } = usePolling<AdminStats | null>(
    async () => {
      try {
        return await api.getStats();
      } catch {
        return null;
      }
    },
    30000,
  );

  // Merge 5h and 7d user data
  const mergedUsers = (() => {
    if (!summary) return [];
    const byUser: Record<string, UserSummary & { _7d?: UserSummary }> = {};

    for (const u of summary.window_5h) {
      byUser[u.username] = { ...u };
    }
    for (const u of summary.window_7d) {
      if (byUser[u.username]) {
        byUser[u.username]._7d = u;
        // Backfill 7d percent if 5h data didn't have it
        if (byUser[u.username].latest_percent_7d === null) {
          byUser[u.username].latest_percent_7d = u.latest_percent_7d;
        }
      } else {
        byUser[u.username] = { ...u, _7d: u };
      }
    }

    return Object.values(byUser).sort((a, b) => {
      const aMax = Math.max(a.latest_percent_5h ?? -1, a.latest_percent_7d ?? -1);
      const bMax = Math.max(b.latest_percent_5h ?? -1, b.latest_percent_7d ?? -1);
      return bMax - aMax;
    });
  })();

  // Compute totals from summary data when admin stats not available
  const totalUsers = adminStats?.total_users ?? mergedUsers.length;
  const totalReports = adminStats?.total_reports ?? mergedUsers.reduce((s, u) => s + u.report_count, 0);
  const totalTokens = adminStats?.total_tokens_processed ??
    mergedUsers.reduce((s, u) => s + u.total_input_tokens + u.total_output_tokens, 0);
  const activeUsers5h = adminStats?.active_users_5h ?? summary?.window_5h.length ?? 0;
  const activeUsers7d = adminStats?.active_users_7d ?? summary?.window_7d.length ?? 0;
  const dbSize = adminStats?.db_size_bytes ?? null;

  return (
    <>
      <div className="page-header">
        <h2>Dashboard</h2>
        <p>Real-time usage tracking across Claude Code instances</p>
      </div>

      <div className="refresh-bar">
        <div className="refresh-dot" />
        <span>Auto-refreshes every 30s</span>
      </div>

      {summaryError && (
        <div className="alert alert-error" style={{ marginBottom: 20 }}>
          Failed to load data: {summaryError}
        </div>
      )}

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-card-label">Total Users</div>
          <div className="stat-card-value">{summaryLoading ? '--' : totalUsers}</div>
        </div>
        <div className="stat-card">
          <div className="stat-card-label">Total Reports</div>
          <div className="stat-card-value">{summaryLoading ? '--' : formatNumber(totalReports)}</div>
        </div>
        <div className="stat-card">
          <div className="stat-card-label">Active (5h)</div>
          <div className="stat-card-value accent">{summaryLoading ? '--' : activeUsers5h}</div>
        </div>
        <div className="stat-card">
          <div className="stat-card-label">Active (7d)</div>
          <div className="stat-card-value">{summaryLoading ? '--' : activeUsers7d}</div>
        </div>
        <div className="stat-card">
          <div className="stat-card-label">Total Tokens</div>
          <div className="stat-card-value">{summaryLoading ? '--' : formatNumber(totalTokens)}</div>
        </div>
        {dbSize !== null && (
          <div className="stat-card">
            <div className="stat-card-label">DB Size</div>
            <div className="stat-card-value">{formatBytes(dbSize)}</div>
          </div>
        )}
      </div>

      <p className="section-title">Quota Usage by User</p>

      {summaryLoading ? (
        <div className="loading-state">
          <div className="spinner" />
          Loading usage data...
        </div>
      ) : mergedUsers.length === 0 ? (
        <div className="loading-state" style={{ color: 'var(--text-dim)' }}>
          No user data available yet.
        </div>
      ) : (
        <div className="users-grid">
          {mergedUsers.map((user) => (
            <UserCard key={user.username} user={user} />
          ))}
        </div>
      )}

      {/* Per-user detail table */}
      {mergedUsers.length > 0 && (
        <div className="table-card" style={{ marginTop: 16 }}>
          <div className="table-card-header">
            <span className="table-card-title">User Summary Table</span>
          </div>
          <div style={{ overflowX: 'auto' }}>
            <table>
              <thead>
                <tr>
                  <th>User</th>
                  <th>5h Quota</th>
                  <th>7d Quota</th>
                  <th>Input Tokens</th>
                  <th>Output Tokens</th>
                  <th>Messages</th>
                  <th>Reports</th>
                  <th>Last Active</th>
                </tr>
              </thead>
              <tbody>
                {mergedUsers.map((u) => {
                  const pct5h = u.latest_percent_5h;
                  const pct7d = u.latest_percent_7d ?? u._7d?.latest_percent_7d ?? null;
                  return (
                    <tr key={u.username}>
                      <td style={{ fontWeight: 600, color: 'var(--text)' }}>{u.username}</td>
                      <td>
                        {pct5h !== null ? (
                          <span className={`badge badge-${percentColor(pct5h)}`}>
                            {pct5h.toFixed(1)}%
                          </span>
                        ) : (
                          <span className="badge badge-muted">N/A</span>
                        )}
                      </td>
                      <td>
                        {pct7d !== null ? (
                          <span className={`badge badge-${percentColor(pct7d)}`}>
                            {pct7d.toFixed(1)}%
                          </span>
                        ) : (
                          <span className="badge badge-muted">N/A</span>
                        )}
                      </td>
                      <td>{u.total_input_tokens.toLocaleString()}</td>
                      <td>{u.total_output_tokens.toLocaleString()}</td>
                      <td>{u.total_messages.toLocaleString()}</td>
                      <td>{u.report_count.toLocaleString()}</td>
                      <td style={{ color: 'var(--text-dim)' }}>{relativeTime(u.last_active)}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </>
  );
}
