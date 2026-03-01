pub fn dashboard_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Claude Code Quota Monitor</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background: #0f0f23;
            color: #c8c8e8;
            min-height: 100vh;
            padding: 24px;
        }

        .container {
            max-width: 1600px;
            margin: 0 auto;
        }

        header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 36px;
            flex-wrap: wrap;
            gap: 12px;
        }

        .header-left h1 {
            font-size: 2rem;
            font-weight: 700;
            color: #e8e8ff;
            letter-spacing: -0.5px;
        }

        .header-left .subtitle {
            font-size: 0.9rem;
            color: #6868a0;
            margin-top: 4px;
        }

        .refresh-info {
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 0.82rem;
            color: #6868a0;
        }

        .refresh-dot {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background: #3b3bff;
            animation: pulse 2s infinite;
        }

        @keyframes pulse {
            0%, 100% { opacity: 1; transform: scale(1); }
            50% { opacity: 0.5; transform: scale(0.8); }
        }

        /* Section headings */
        .section-title {
            font-size: 0.75rem;
            font-weight: 600;
            letter-spacing: 0.12em;
            text-transform: uppercase;
            color: #4a4a7a;
            margin-bottom: 16px;
        }

        /* User quota cards grid */
        .users-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
            gap: 20px;
            margin-bottom: 32px;
        }

        .user-card {
            background: #1a1a3e;
            border: 1px solid #2a2a5a;
            border-radius: 16px;
            padding: 24px;
            transition: border-color 0.2s, transform 0.2s;
        }

        .user-card:hover {
            border-color: #4a4a9a;
            transform: translateY(-2px);
        }

        .user-card-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 24px;
        }

        .user-avatar {
            width: 40px;
            height: 40px;
            border-radius: 50%;
            background: linear-gradient(135deg, #3b3bff, #8b5cf6);
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: 700;
            font-size: 1rem;
            color: #fff;
            flex-shrink: 0;
        }

        .user-info {
            flex: 1;
            margin-left: 12px;
        }

        .user-name {
            font-size: 1.05rem;
            font-weight: 600;
            color: #e8e8ff;
        }

        .user-last-active {
            font-size: 0.78rem;
            color: #5a5a8a;
            margin-top: 2px;
        }

        /* Circular progress indicators */
        .gauges-row {
            display: flex;
            justify-content: center;
            gap: 32px;
            margin-bottom: 24px;
        }

        .gauge-wrap {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 8px;
        }

        .gauge-label {
            font-size: 0.72rem;
            font-weight: 600;
            letter-spacing: 0.08em;
            text-transform: uppercase;
            color: #5a5a8a;
        }

        .circular-progress {
            position: relative;
            width: 110px;
            height: 110px;
        }

        .circular-progress svg {
            transform: rotate(-90deg);
            width: 110px;
            height: 110px;
        }

        .circular-progress .track {
            fill: none;
            stroke: #2a2a50;
            stroke-width: 8;
        }

        .circular-progress .fill {
            fill: none;
            stroke-width: 8;
            stroke-linecap: round;
            transition: stroke-dashoffset 0.8s ease, stroke 0.4s ease;
        }

        .circular-progress .center-text {
            position: absolute;
            inset: 0;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
        }

        .gauge-percent {
            font-size: 1.5rem;
            font-weight: 700;
            line-height: 1;
        }

        .gauge-unit {
            font-size: 0.65rem;
            color: #5a5a8a;
            margin-top: 2px;
        }

        /* Color classes */
        .color-green  { color: #22c55e; }
        .color-yellow { color: #eab308; }
        .color-orange { color: #f97316; }
        .color-red    { color: #ef4444; }
        .color-null   { color: #4a4a7a; }

        .stroke-green  { stroke: #22c55e; }
        .stroke-yellow { stroke: #eab308; }
        .stroke-orange { stroke: #f97316; }
        .stroke-red    { stroke: #ef4444; }
        .stroke-null   { stroke: #2a2a50; }

        /* Token stats below gauges */
        .token-stats {
            border-top: 1px solid #2a2a50;
            padding-top: 16px;
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 10px;
        }

        .token-stat {
            display: flex;
            flex-direction: column;
            gap: 2px;
        }

        .token-stat-label {
            font-size: 0.7rem;
            color: #4a4a7a;
            text-transform: uppercase;
            letter-spacing: 0.06em;
        }

        .token-stat-value {
            font-size: 0.88rem;
            font-weight: 600;
            color: #a0a0c8;
        }

        .token-stat-value.highlight {
            color: #8b5cf6;
        }

        /* Status badge */
        .status-badge {
            padding: 3px 8px;
            border-radius: 20px;
            font-size: 0.68rem;
            font-weight: 600;
            letter-spacing: 0.06em;
        }

        .badge-green  { background: #14532d; color: #4ade80; }
        .badge-yellow { background: #422006; color: #fbbf24; }
        .badge-orange { background: #431407; color: #fb923c; }
        .badge-red    { background: #450a0a; color: #f87171; }
        .badge-null   { background: #1a1a3e; color: #4a4a7a; border: 1px solid #2a2a50; }

        /* Timeline chart card */
        .chart-card {
            background: #1a1a3e;
            border: 1px solid #2a2a5a;
            border-radius: 16px;
            padding: 24px;
            margin-bottom: 32px;
        }

        .chart-card-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 20px;
            flex-wrap: wrap;
            gap: 12px;
        }

        .chart-card-title {
            font-size: 1rem;
            font-weight: 600;
            color: #e8e8ff;
        }

        .chart-container {
            position: relative;
            height: 320px;
        }

        /* Loading / empty states */
        .loading-state {
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 200px;
            color: #4a4a7a;
            font-size: 0.9rem;
        }

        .spinner {
            width: 24px;
            height: 24px;
            border: 3px solid #2a2a50;
            border-top-color: #3b3bff;
            border-radius: 50%;
            animation: spin 0.8s linear infinite;
            margin-right: 10px;
        }

        @keyframes spin {
            to { transform: rotate(360deg); }
        }

        .no-data {
            text-align: center;
            padding: 48px 24px;
            color: #4a4a7a;
        }

        .no-data-icon {
            font-size: 2rem;
            margin-bottom: 8px;
            opacity: 0.4;
        }

        /* Responsive */
        @media (max-width: 640px) {
            body { padding: 12px; }
            .users-grid { grid-template-columns: 1fr; }
            header { flex-direction: column; align-items: flex-start; }
            .gauges-row { gap: 20px; }
            .circular-progress, .circular-progress svg { width: 90px; height: 90px; }
            .gauge-percent { font-size: 1.2rem; }
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <div class="header-left">
                <h1>Claude Code Quota Monitor</h1>
                <p class="subtitle">Real-time usage tracking across Claude Code instances</p>
            </div>
            <div class="refresh-info">
                <div class="refresh-dot"></div>
                <span id="refresh-label">Refreshes every 30s</span>
            </div>
        </header>

        <p class="section-title">Quota Usage by User</p>
        <div class="users-grid" id="users-grid">
            <div class="loading-state">
                <div class="spinner"></div>
                Loading users...
            </div>
        </div>

        <div class="chart-card">
            <div class="chart-card-header">
                <span class="chart-card-title">Hourly Token Usage (7 Days)</span>
                <span id="chart-last-updated" style="font-size:0.78rem;color:#4a4a7a;"></span>
            </div>
            <div class="chart-container">
                <canvas id="timelineChart"></canvas>
            </div>
        </div>
    </div>

    <script>
        let timelineChart = null;

        // ── Colour helpers ────────────────────────────────────────────────────
        function percentColor(pct) {
            if (pct === null || pct === undefined) return 'null';
            if (pct < 50)  return 'green';
            if (pct < 75)  return 'yellow';
            if (pct < 90)  return 'orange';
            return 'red';
        }

        function badgeText(pct) {
            if (pct === null || pct === undefined) return 'N/A';
            if (pct < 50)  return 'OK';
            if (pct < 75)  return 'MODERATE';
            if (pct < 90)  return 'HIGH';
            return 'CRITICAL';
        }

        // ── Circular progress SVG ────────────────────────────────────────────
        // r=46, circumference = 2 * π * 46 ≈ 289.03
        const CIRC = 2 * Math.PI * 46;

        function circularGauge(pct, label) {
            const color = percentColor(pct);
            const displayPct = (pct === null || pct === undefined) ? '—' : pct.toFixed(1) + '%';
            const offset = (pct === null || pct === undefined)
                ? CIRC
                : CIRC * (1 - Math.min(pct, 100) / 100);

            return `
                <div class="gauge-wrap">
                    <div class="gauge-label">${label}</div>
                    <div class="circular-progress">
                        <svg viewBox="0 0 110 110">
                            <circle class="track" cx="55" cy="55" r="46"/>
                            <circle
                                class="fill stroke-${color}"
                                cx="55" cy="55" r="46"
                                stroke-dasharray="${CIRC.toFixed(2)}"
                                stroke-dashoffset="${offset.toFixed(2)}"
                            />
                        </svg>
                        <div class="center-text">
                            <span class="gauge-percent color-${color}">${displayPct}</span>
                            <span class="gauge-unit">of quota</span>
                        </div>
                    </div>
                </div>
            `;
        }

        // ── Time formatting ──────────────────────────────────────────────────
        function relativeTime(isoStr) {
            if (!isoStr) return 'never';
            const diff = Date.now() - new Date(isoStr).getTime();
            const m = Math.floor(diff / 60000);
            if (m < 1)  return 'just now';
            if (m < 60) return `${m}m ago`;
            const h = Math.floor(m / 60);
            if (h < 24) return `${h}h ago`;
            return `${Math.floor(h / 24)}d ago`;
        }

        // ── Render per-user cards ────────────────────────────────────────────
        function renderUsersGrid(summary) {
            const users5h  = summary.window_5h  || [];
            const users7d  = summary.window_7d  || [];

            // Merge by username
            const byUser = {};
            for (const u of users5h) {
                byUser[u.username] = { ...byUser[u.username], ...u, _5h: u };
            }
            for (const u of users7d) {
                byUser[u.username] = { ...byUser[u.username], _7d: u };
            }

            const userList = Object.values(byUser);
            if (userList.length === 0) {
                document.getElementById('users-grid').innerHTML = `
                    <div class="no-data" style="grid-column:1/-1;">
                        <div class="no-data-icon">○</div>
                        <div>No users found</div>
                    </div>`;
                return;
            }

            // Sort by max of (5h%, 7d%) descending
            userList.sort((a, b) => {
                const aMax = Math.max(a._5h?.latest_percent_5h ?? -1, a._5h?.latest_percent_7d ?? -1);
                const bMax = Math.max(b._5h?.latest_percent_5h ?? -1, b._5h?.latest_percent_7d ?? -1);
                return bMax - aMax;
            });

            const grid = document.getElementById('users-grid');
            grid.innerHTML = userList.map(u => {
                const base     = u._5h || u._7d || u;
                const pct5h    = base.latest_percent_5h;
                const pct7d    = base.latest_percent_7d;
                const worstClr = percentColor(Math.max(pct5h ?? -1, pct7d ?? -1) < 0 ? null : Math.max(pct5h ?? 0, pct7d ?? 0));
                const initial  = (base.username || '?')[0].toUpperCase();
                const lastActive = relativeTime(base.last_active);

                const inp  = (base.total_input_tokens          || 0).toLocaleString();
                const out  = (base.total_output_tokens         || 0).toLocaleString();
                const cread= (base.total_cache_read_tokens     || 0).toLocaleString();
                const ccre = (base.total_cache_creation_tokens || 0).toLocaleString();
                const msgs = (base.total_messages              || 0).toLocaleString();
                const tools= (base.total_tool_uses             || 0).toLocaleString();

                return `
                <div class="user-card">
                    <div class="user-card-header">
                        <div class="user-avatar">${initial}</div>
                        <div class="user-info">
                            <div class="user-name">${base.username}</div>
                            <div class="user-last-active">Active ${lastActive}</div>
                        </div>
                        <span class="status-badge badge-${worstClr}">${badgeText(Math.max(pct5h ?? -1, pct7d ?? -1) < 0 ? null : Math.max(pct5h ?? 0, pct7d ?? 0))}</span>
                    </div>

                    <div class="gauges-row">
                        ${circularGauge(pct5h, '5h window')}
                        ${circularGauge(pct7d, '7d window')}
                    </div>

                    <div class="token-stats">
                        <div class="token-stat">
                            <span class="token-stat-label">Input tokens</span>
                            <span class="token-stat-value">${inp}</span>
                        </div>
                        <div class="token-stat">
                            <span class="token-stat-label">Output tokens</span>
                            <span class="token-stat-value">${out}</span>
                        </div>
                        <div class="token-stat">
                            <span class="token-stat-label">Cache read</span>
                            <span class="token-stat-value">${cread}</span>
                        </div>
                        <div class="token-stat">
                            <span class="token-stat-label">Cache created</span>
                            <span class="token-stat-value">${ccre}</span>
                        </div>
                        <div class="token-stat">
                            <span class="token-stat-label">Messages</span>
                            <span class="token-stat-value highlight">${msgs}</span>
                        </div>
                        <div class="token-stat">
                            <span class="token-stat-label">Tool uses</span>
                            <span class="token-stat-value highlight">${tools}</span>
                        </div>
                    </div>
                </div>`;
            }).join('');
        }

        // ── Render timeline chart ────────────────────────────────────────────
        const CHART_COLORS = [
            '#6366f1','#22c55e','#f59e0b','#ec4899','#14b8a6',
            '#f97316','#8b5cf6','#06b6d4','#84cc16','#ef4444'
        ];

        function renderTimeline(hourly) {
            const ctx = document.getElementById('timelineChart').getContext('2d');
            const users = [...new Set(hourly.map(h => h.username))].sort();

            // Build hour labels (unique, sorted)
            const hourLabels = [...new Set(hourly.map(h => h.hour))].sort();

            const datasets = users.map((user, idx) => {
                const userMap = {};
                hourly.filter(h => h.username === user).forEach(h => {
                    userMap[h.hour] = (h.input_tokens || 0) + (h.output_tokens || 0);
                });
                return {
                    label: user,
                    data: hourLabels.map(hr => userMap[hr] ?? null),
                    borderColor: CHART_COLORS[idx % CHART_COLORS.length],
                    backgroundColor: CHART_COLORS[idx % CHART_COLORS.length] + '18',
                    borderWidth: 2,
                    tension: 0.3,
                    fill: true,
                    spanGaps: true,
                    pointRadius: 2,
                    pointHoverRadius: 5
                };
            });

            const labels = hourLabels.map(h => {
                const d = new Date(h);
                return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
                       + ' ' + d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
            });

            if (timelineChart) timelineChart.destroy();
            timelineChart = new Chart(ctx, {
                type: 'line',
                data: { labels, datasets },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    interaction: { mode: 'index', intersect: false },
                    plugins: {
                        legend: {
                            display: true,
                            position: 'top',
                            labels: {
                                color: '#8888b0',
                                boxWidth: 12,
                                padding: 16,
                                font: { size: 12 }
                            }
                        },
                        tooltip: {
                            backgroundColor: '#1a1a3e',
                            borderColor: '#2a2a5a',
                            borderWidth: 1,
                            titleColor: '#c8c8e8',
                            bodyColor: '#8888b0',
                            callbacks: {
                                label: ctx => {
                                    const v = ctx.parsed.y;
                                    return v === null ? null : ` ${ctx.dataset.label}: ${v.toLocaleString()} tokens`;
                                }
                            }
                        }
                    },
                    scales: {
                        x: {
                            ticks: {
                                color: '#4a4a7a',
                                maxTicksLimit: 12,
                                font: { size: 11 }
                            },
                            grid: { color: '#1e1e40' }
                        },
                        y: {
                            beginAtZero: true,
                            ticks: {
                                color: '#4a4a7a',
                                font: { size: 11 },
                                callback: v => v >= 1000 ? (v / 1000).toFixed(0) + 'k' : v
                            },
                            grid: { color: '#1e1e40' }
                        }
                    }
                }
            });

            document.getElementById('chart-last-updated').textContent =
                'Updated ' + new Date().toLocaleTimeString();
        }

        // ── Main load ────────────────────────────────────────────────────────
        async function loadData() {
            try {
                const [summaryRes, hourlyRes] = await Promise.all([
                    fetch('/api/summary'),
                    fetch('/api/hourly')
                ]);

                if (!summaryRes.ok) throw new Error('Summary endpoint returned ' + summaryRes.status);
                if (!hourlyRes.ok)  throw new Error('Hourly endpoint returned '  + hourlyRes.status);

                const summary = await summaryRes.json();
                const hourly  = await hourlyRes.json();

                renderUsersGrid(summary);
                renderTimeline(hourly);

            } catch (e) {
                console.error('Failed to load data:', e);
                document.getElementById('users-grid').innerHTML = `
                    <div style="grid-column:1/-1;background:#2a0a0a;border:1px solid #5a1a1a;border-radius:12px;padding:24px;color:#f87171;">
                        Failed to load data: ${e.message}
                    </div>`;
            }
        }

        setInterval(loadData, 30000);
        loadData();
    </script>
</body>
</html>"#.to_string()
}
