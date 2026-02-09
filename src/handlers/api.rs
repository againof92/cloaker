use axum::extract::ConnectInfo;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Redirect};
use axum::Json;
use futures::stream::Stream;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::engine::{cloaking, helpers};
use crate::storage;
use crate::templates;
use crate::AppState;

// ==========================================
// STATS (JSON)
// ==========================================
pub async fn handle_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let links = state.db.links.read().await;
    let mut total_links = 0i32;
    let mut total_clicks = 0i32;
    let mut total_blocked = 0i32;
    for link in links.values() {
        if link.active {
            total_links += 1;
        }
        total_clicks += link.clicks;
        total_blocked += link.blocked;
    }
    Json(serde_json::json!({
        "total_links": total_links,
        "total_clicks": total_clicks,
        "total_blocked": total_blocked,
    }))
}

// ==========================================
// MAP STATS (JSON)
// ==========================================
pub async fn handle_map_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let logs = storage::get_logs(&state.pool, 1000)
        .await
        .unwrap_or_default();
    let counts = cloaking::build_state_counts(&logs);
    Json(counts)
}

// ==========================================
// LOGS PAGE
// ==========================================
pub async fn handle_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let logs = storage::get_logs(&state.pool, 1000)
        .await
        .unwrap_or_default();

    if headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .contains("application/json")
    {
        return Json(serde_json::to_value(&logs).unwrap_or_default()).into_response();
    }

    let logs_b64 = helpers::encode_json_base64(&logs);
    Html(templates::logs_page(&logs_b64)).into_response()
}

// ==========================================
// LOGS SSE STREAM
// ==========================================
pub async fn handle_logs_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let log_rx = state.log_tx.subscribe();
    let clear_rx = state.clear_tx.subscribe();

    let log_stream = BroadcastStream::new(log_rx).filter_map(|result| match result {
        Ok(entry) => {
            let data = serde_json::to_string(&entry).unwrap_or_default();
            Some(Ok(Event::default().event("log").data(data)))
        }
        Err(_) => None,
    });

    let clear_stream = BroadcastStream::new(clear_rx).filter_map(|result| match result {
        Ok(_) => Some(Ok(Event::default().event("clear").data("{}"))),
        Err(_) => None,
    });

    let merged = futures::stream::select(log_stream, clear_stream);

    Sse::new(merged).keep_alive(KeepAlive::default())
}

// ==========================================
// LOGS CLEAR
// ==========================================
pub async fn handle_logs_clear(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = storage::clear_logs(&state.pool).await;
    let _ = state.clear_tx.send(());
    Redirect::to("/m4ciel7/logs")
}

// ==========================================
// GEOIP TEST
// ==========================================
pub async fn handle_geoip_test(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let ip = q
        .get("ip")
        .cloned()
        .unwrap_or_else(|| helpers::get_client_ip(&headers, Some(addr)));
    let geo = state.geo_cache.get_geo_info(&ip).await;
    Json(geo)
}
