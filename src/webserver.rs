use axum::{
    extract::State,
    response::{Html, Json},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;

use crate::metrics::{MetricsCollector, RelayMetrics};

pub struct AppState {
    pub metrics: Arc<MetricsCollector>,
}

pub fn create_router(metrics: Arc<MetricsCollector>) -> Router {
    let state = AppState { metrics };

    Router::new()
        .route("/", get(homepage))
        .route("/api/metrics", get(metrics_json))
        .route("/api/peerid", get(peer_id_json))
        .with_state(Arc::new(state))
        .layer(ServiceBuilder::new())
}

async fn homepage(State(state): State<Arc<AppState>>) -> Html<String> {
    let metrics = state.metrics.get_metrics();
    
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>LibP2P Relay Metrics</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 40px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 800px;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .header {{
            text-align: center;
            color: #333;
            border-bottom: 2px solid #eee;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }}
        .metrics-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .metric-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 6px;
            border-left: 4px solid #007bff;
        }}
        .metric-value {{
            font-size: 2em;
            font-weight: bold;
            color: #007bff;
        }}
        .metric-label {{
            color: #666;
            font-size: 0.9em;
            margin-top: 5px;
        }}
        .relay-addresses {{
            margin: 20px 0;
        }}
        .relay-address {{
            background: #e9ecef;
            padding: 15px;
            border-radius: 4px;
            font-family: monospace;
            word-break: break-all;
            margin: 10px 0;
            border-left: 4px solid #007bff;
        }}
        .connections-table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }}
        .connections-table th,
        .connections-table td {{
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        .connections-table th {{
            background-color: #f8f9fa;
            font-weight: bold;
        }}
        .footer {{
            text-align: center;
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #eee;
            color: #666;
            font-size: 0.9em;
        }}
        .auto-refresh {{
            float: right;
            color: #666;
            font-size: 0.8em;
        }}
    </style>
    <meta http-equiv="refresh" content="30">
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🌐 LibP2P Relay Metrics</h1>
            <div class="auto-refresh">Auto-refreshes every 30 seconds</div>
        </div>
        
        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Total Connections</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Active Connections</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.2} MB</div>
                <div class="metric-label">Data Transferred (24h)</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Messages Relayed (24h)</div>
            </div>
        </div>

        <h3>Relay Addresses</h3>
        <div class="relay-addresses">
            {}
        </div>

        <h3>Uptime: {} hours</h3>

        <h3>Active Connections</h3>
        <table class="connections-table">
            <tr>
                <th>Peer ID</th>
                <th>Connected Since</th>
                <th>Bytes Sent</th>
                <th>Bytes Received</th>
                <th>Messages Relayed</th>
            </tr>
            {}
        </table>

        <div class="footer">
            <p>LibP2P Relay Server - Last updated: {}</p>
            <p><a href="/api/metrics">View JSON API</a></p>
        </div>
    </div>
</body>
</html>"#,
        metrics.total_connections,
        metrics.active_connections,
        metrics.total_bytes_transferred_24h as f64 / 1_048_576.0, // Convert bytes to MB
        metrics.total_messages_relayed_24h,
        if metrics.relay_addresses.is_empty() {
            "<div class=\"relay-address\">No addresses available yet...</div>".to_string()
        } else {
            metrics.relay_addresses
                .iter()
                .map(|addr| format!("<div class=\"relay-address\">{}</div>", addr))
                .collect::<Vec<String>>()
                .join("")
        },
        metrics.uptime.num_hours(),
        metrics.connections.iter()
            .map(|conn| format!(
                "<tr>
                    <td>...{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>",
                &conn.peer_id[conn.peer_id.len().saturating_sub(5)..], // Show last 5 chars of peer ID
                conn.connected_at.format("%Y-%m-%d %H:%M:%S UTC"),
                format_bytes(conn.bytes_sent),
                format_bytes(conn.bytes_received),
                conn.messages_relayed
            ))
            .collect::<Vec<String>>()
            .join(""),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    Html(html)
}

async fn metrics_json(State(state): State<Arc<AppState>>) -> Json<RelayMetrics> {
    let metrics = state.metrics.get_metrics();
    Json(metrics)
}

async fn peer_id_json(State(state): State<Arc<AppState>>) -> Json<String> {
    let peer_id = state.metrics.get_peer_id();
    Json(peer_id)
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}