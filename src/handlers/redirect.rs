use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, Redirect, IntoResponse};
use std::collections::HashMap;
use std::sync::Arc;
use std::net::SocketAddr;
use axum::extract::ConnectInfo;
use chrono::Utc;

use crate::engine::{cloaking, helpers};
use crate::models::AccessLog;
use crate::templates;
use crate::AppState;

pub async fn handle_home(State(state): State<Arc<AppState>>) -> Html<String> {
    let config = state.db.config.read().await;
    let safe_url = config.safe_page_url.clone();
    drop(config);

    if !safe_url.trim().is_empty() {
        if let Some(body) = helpers::fetch_safe_page_html(&safe_url, &state.http_client).await {
            return Html(body);
        }
    }
    Html(templates::safe_page())
}

pub async fn handle_redirect(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let slug = slug.split('/').next().unwrap_or("").split('?').next().unwrap_or("").trim().to_string();

    // Busca link ativo pelo slug
    let link = {
        let links = state.db.links.read().await;
        links.values().find(|l| l.active && l.slug.eq_ignore_ascii_case(&slug)).cloned()
    };

    let link = match link {
        Some(l) => l,
        None => {
            log_access(&state, &headers, Some(addr), "", true, "Link nao encontrado", "", None).await;
            return show_safe_page(&state, None).await.into_response();
        }
    };

    let client_ip = helpers::get_client_ip(&headers, Some(addr));
    let geo = state.geo_cache.get_geo_info(&client_ip).await;
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("");
    let referer = headers.get("referer").and_then(|v| v.to_str().ok()).unwrap_or("");

    let result = cloaking::validate_access(
        &state, &headers, &params, &client_ip, user_agent, referer, &geo, &link,
    ).await;

    if result.allowed {
        // Incrementa cliques
        {
            let mut links = state.db.links.write().await;
            if let Some(l) = links.get_mut(&link.id) {
                l.clicks += 1;
            }
        }
        let pool = state.pool.clone();
        let lid = link.id.clone();
        tokio::spawn(async move { crate::storage::increment_link_stats(&pool, &lid, 1, 0).await; });

        log_access(&state, &headers, Some(addr), &link.id, false, "Acesso valido", &link.offer_url, Some(&geo)).await;
        Redirect::to(&link.offer_url).into_response()
    } else {
        // Incrementa bloqueados
        {
            let mut links = state.db.links.write().await;
            if let Some(l) = links.get_mut(&link.id) {
                l.blocked += 1;
            }
        }
        let pool = state.pool.clone();
        let lid = link.id.clone();
        tokio::spawn(async move { crate::storage::increment_link_stats(&pool, &lid, 0, 1).await; });

        log_access(&state, &headers, Some(addr), &link.id, true, &result.reason, "", Some(&geo)).await;
        show_safe_page(&state, Some(&link)).await.into_response()
    }
}

async fn show_safe_page(state: &Arc<AppState>, link: Option<&crate::models::RedirectLink>) -> Html<String> {
    let safe_url = if let Some(l) = link {
        if !l.safe_page_url.trim().is_empty() {
            l.safe_page_url.clone()
        } else {
            let cfg = state.db.config.read().await;
            cfg.safe_page_url.clone()
        }
    } else {
        let cfg = state.db.config.read().await;
        cfg.safe_page_url.clone()
    };

    if !safe_url.trim().is_empty() {
        if let Some(body) = helpers::fetch_safe_page_html(&safe_url, &state.http_client).await {
            return Html(body);
        }
    }
    Html(templates::safe_page())
}

async fn log_access(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    addr: Option<SocketAddr>,
    link_id: &str,
    blocked: bool,
    reason: &str,
    redirect_to: &str,
    geo: Option<&crate::models::GeoInfo>,
) {
    let log_entry = AccessLog {
        timestamp: Utc::now(),
        link_id: link_id.to_string(),
        ip: helpers::get_client_ip(headers, addr),
        user_agent: headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("").to_string(),
        referer: headers.get("referer").and_then(|v| v.to_str().ok()).unwrap_or("").to_string(),
        country: geo.map(|g| helpers::fix_utf8(&g.country)).unwrap_or_default(),
        country_code: geo.map(|g| g.country_code.clone()).unwrap_or_default(),
        region: geo.map(|g| g.region.clone()).unwrap_or_default(),
        region_name: geo.map(|g| helpers::fix_utf8(&g.region_name)).unwrap_or_default(),
        city: geo.map(|g| helpers::fix_utf8(&g.city)).unwrap_or_default(),
        isp: geo.map(|g| helpers::fix_utf8(&g.isp)).unwrap_or_default(),
        is_vpn: geo.map(|g| g.proxy || g.hosting).unwrap_or(false),
        blocked,
        reason: reason.to_string(),
        redirect_to: redirect_to.to_string(),
    };

    // Broadcast via SSE
    let _ = state.log_tx.send(log_entry.clone());

    // Persiste no DB (fire and forget)
    let pool = state.pool.clone();
    tokio::spawn(async move {
        crate::storage::insert_log(&pool, &log_entry).await;
    });
}
