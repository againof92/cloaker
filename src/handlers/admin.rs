use axum::extract::ConnectInfo;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{Html, IntoResponse, Redirect};
use axum::Form;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::auth;
use crate::config;
use crate::engine::helpers;
use crate::models::RedirectLink;
use crate::storage;
use crate::templates;
use crate::AppState;

// ==========================================
// LOGIN
// ==========================================
pub async fn handle_login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    form: Option<Form<HashMap<String, String>>>,
) -> impl IntoResponse {
    let client_ip = helpers::get_client_ip(&headers, Some(addr));
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Verificação anti-bot
    if let Some((status, body)) = auth::admin_bot_check(
        &state,
        &headers,
        &client_ip,
        user_agent,
        host,
        "/m4ciel7/login",
    )
    .await
    {
        return (status, jar, Html(body)).into_response();
    }

    // POST → processar login
    if method == Method::POST {
        let Some(Form(form_data)) = form else {
            return (
                StatusCode::BAD_REQUEST,
                jar,
                Html(templates::login_page("Requisicao invalida")),
            )
                .into_response();
        };
        let user = form_data.get("username").cloned().unwrap_or_default();
        let pass = form_data.get("password").cloned().unwrap_or_default();
        let (admin_user, admin_pass) = config::admin_credentials();

        if user == admin_user && pass == admin_pass {
            let session_id = auth::generate_session_id();
            let expiry = Utc::now() + Duration::hours(24);
            {
                let mut sessions = state.sessions.write().await;
                sessions.insert(session_id.clone(), expiry);
            }
            let cookie = Cookie::build(("session", session_id))
                .path("/")
                .http_only(true)
                .same_site(axum_extra::extract::cookie::SameSite::Lax)
                .max_age(::time::Duration::hours(24));
            let jar = jar.add(cookie);
            return (jar, Redirect::to("/m4ciel7")).into_response();
        }

        return (
            StatusCode::UNAUTHORIZED,
            jar,
            Html(templates::login_page("Usuario ou senha incorretos")),
        )
            .into_response();
    }

    // GET → mostrar formulário
    (jar, Html(templates::login_page(""))).into_response()
}

// ==========================================
// LOGOUT
// ==========================================
pub async fn handle_logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(cookie) = jar.get("session") {
        let mut sessions = state.sessions.write().await;
        sessions.remove(cookie.value());
    }
    let jar = jar.remove(Cookie::build("session").path("/"));
    (jar, Redirect::to("/m4ciel7/login"))
}

// ==========================================
// DASHBOARD
// ==========================================
pub async fn handle_dashboard(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Content negotiation JSON
    if headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .contains("application/json")
    {
        let (total_links, total_clicks, total_blocked) = get_link_stats(&state).await;
        let config = state.db.config.read().await;
        let json = serde_json::json!({
            "total_links": total_links,
            "total_clicks": total_clicks,
            "total_blocked": total_blocked,
            "config": *config,
        });
        return axum::Json(json).into_response();
    }

    let (total_links, total_clicks, total_blocked) = get_link_stats(&state).await;
    let config = state.db.config.read().await;
    let param_name = config.param_name.clone();
    let only_fb = config.only_facebook_ads;
    drop(config);

    let total = total_clicks + total_blocked;
    let block_rate = if total > 0 {
        (total_blocked * 100) / total
    } else {
        0
    };
    let fb_rule = if only_fb {
        "Somente anuncios Facebook/Instagram"
    } else {
        "Somente anuncios Facebook/Instagram (DESATIVADO)"
    };

    // Mapa de estado: pega logs do DB
    let logs = storage::get_logs(&state.pool, 1000)
        .await
        .unwrap_or_default();
    let state_counts = crate::engine::cloaking::build_state_counts(&logs);
    let state_counts_json = serde_json::to_string(&state_counts).unwrap_or("{}".into());
    let state_names_json = serde_json::to_string(&*helpers::BR_STATE_NAMES).unwrap_or("{}".into());
    let map_svg = helpers::clean_svg(&state.brazil_map_svg);

    Html(templates::dashboard_page(templates::DashboardPageData {
        total_links,
        total_clicks,
        total_blocked,
        block_rate,
        param_name: &param_name,
        fb_rule,
        map_svg: &map_svg,
        state_counts_json: &state_counts_json,
        state_names_json: &state_names_json,
    }))
    .into_response()
}

// ==========================================
// LINKS LIST
// ==========================================
pub async fn handle_links(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // JSON
    if headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .contains("application/json")
    {
        let links = state.db.links.read().await;
        let all: Vec<&RedirectLink> = links.values().collect();
        return axum::Json(serde_json::to_value(&all).unwrap_or_default()).into_response();
    }

    let config = state.db.config.read().await;
    let param_name = config.param_name.clone();
    drop(config);

    let links = state.db.links.read().await;
    let mut rows: Vec<(String, String, String, String, i32, i32, bool)> = links
        .values()
        .map(|l| {
            (
                l.id.clone(),
                l.slug.clone(),
                l.param_code.clone(),
                l.offer_url.clone(),
                l.clicks,
                l.blocked,
                l.active,
            )
        })
        .collect();
    drop(links);

    rows.sort_by(|a, b| {
        b.6.cmp(&a.6)
            .then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase()))
    });

    Html(templates::links_page(&param_name, &rows)).into_response()
}

// ==========================================
// CREATE LINK
// ==========================================
pub async fn handle_create_link(
    State(state): State<Arc<AppState>>,
    method: Method,
    form: Option<Form<HashMap<String, String>>>,
) -> impl IntoResponse {
    let config = state.db.config.read().await;
    let param_name = config.param_name.clone();
    drop(config);

    if method == Method::POST {
        let Some(Form(data)) = form else {
            return (
                StatusCode::BAD_REQUEST,
                Html(templates::error_page("Requisicao invalida")),
            )
                .into_response();
        };
        let slug = data
            .get("slug")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let param_code = data
            .get("param_code")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let offer_url = data
            .get("offer_url")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let safe_page_url = data
            .get("safe_page_url")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let active = data.get("active").map(|v| v == "on").unwrap_or(false);

        if let Err(e) = helpers::validate_slug(&slug) {
            return Html(templates::error_page(&e)).into_response();
        }
        if is_slug_taken(&state, &slug, "").await {
            return Html(templates::error_page("Slug ja em uso")).into_response();
        }
        if let Err(e) = helpers::validate_offer_url(&offer_url) {
            return Html(templates::error_page(&e)).into_response();
        }
        if !safe_page_url.is_empty() && helpers::validate_offer_url(&safe_page_url).is_err() {
            return Html(templates::error_page("Safe page URL invalida")).into_response();
        }

        let param_code = if param_code.is_empty() {
            helpers::generate_code(8)
        } else {
            param_code
        };
        let max_clicks: i32 = data
            .get("max_clicks")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
            .max(0);
        let param_ttl: i32 = data
            .get("param_ttl")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
            .max(0);
        let allowed_hours = data
            .get("allowed_hours")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let mut link = RedirectLink {
            id: helpers::generate_code(8),
            slug,
            offer_url,
            safe_page_url,
            param_hash: String::new(),
            param_code: String::new(),
            clicks: 0,
            blocked: 0,
            created_at: Utc::now(),
            active,
            cloaker_active: true,
            ad_verify_mode: false,
            block_vpn: false,
            mobile_only: true,
            require_facebook: true,
            max_clicks,
            param_ttl,
            allowed_hours,
            allowed_countries: helpers::parse_csv(
                data.get("allowed_countries").unwrap_or(&String::new()),
            ),
            blocked_countries: helpers::parse_csv(
                data.get("blocked_countries").unwrap_or(&String::new()),
            ),
            blocked_ips: helpers::parse_csv(data.get("blocked_ips").unwrap_or(&String::new())),
            blocked_isps: helpers::parse_csv(data.get("blocked_isps").unwrap_or(&String::new())),
            protection_total: false,
            strict_param_required: true,
            only_facebook_ads: true,
            advanced_fingerprint: false,
            ml_bot_detection: false,
            dynamic_referrer_spoof: false,
        };
        helpers::set_param(&mut link, &param_code);

        if storage::save_link(&state.pool, &link).await.is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(templates::error_page("Falha ao salvar link no banco")),
            )
                .into_response();
        }

        {
            let mut links = state.db.links.write().await;
            links.insert(link.id.clone(), link.clone());
        }

        return Redirect::to("/m4ciel7/links").into_response();
    }

    let new_code = helpers::generate_code(8);
    Html(templates::create_link_page(&param_name, &new_code)).into_response()
}

// ==========================================
// EDIT LINK
// ==========================================
pub async fn handle_edit_link(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
    method: Method,
    form: Option<Form<HashMap<String, String>>>,
) -> impl IntoResponse {
    let id = q.get("id").cloned().unwrap_or_default();
    if id.is_empty() {
        return Html(templates::error_page("ID do link ausente")).into_response();
    }

    let config = state.db.config.read().await;
    let param_name = config.param_name.clone();
    drop(config);

    if method == Method::POST {
        let Some(Form(data)) = form else {
            return (
                StatusCode::BAD_REQUEST,
                Html(templates::error_page("Requisicao invalida")),
            )
                .into_response();
        };
        let slug = data
            .get("slug")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let param_code = data
            .get("param_code")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let offer_url = data
            .get("offer_url")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let safe_page_url = data
            .get("safe_page_url")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let active = data.get("active").map(|v| v == "on").unwrap_or(false);

        if let Err(e) = helpers::validate_slug(&slug) {
            return Html(templates::error_page(&e)).into_response();
        }
        if let Err(e) = helpers::validate_offer_url(&offer_url) {
            return Html(templates::error_page(&e)).into_response();
        }
        if !safe_page_url.is_empty() && helpers::validate_offer_url(&safe_page_url).is_err() {
            return Html(templates::error_page("Safe page URL invalida")).into_response();
        }

        let max_clicks: i32 = data
            .get("max_clicks")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
            .max(0);
        let param_ttl: i32 = data
            .get("param_ttl")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
            .max(0);
        let allowed_hours = data
            .get("allowed_hours")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let mut updated_link = {
            let links = state.db.links.read().await;
            let slug_taken = links
                .values()
                .any(|l| l.id != id && l.slug.eq_ignore_ascii_case(&slug));
            let Some(current) = links.get(&id) else {
                return Html(templates::error_page("Link nao encontrado")).into_response();
            };
            if !slug.eq_ignore_ascii_case(&current.slug) && slug_taken {
                return Html(templates::error_page("Slug ja em uso")).into_response();
            }
            current.clone()
        };

        updated_link.slug = slug;
        updated_link.offer_url = offer_url;
        updated_link.safe_page_url = safe_page_url;
        updated_link.active = active;
        updated_link.max_clicks = max_clicks;
        updated_link.param_ttl = param_ttl;
        updated_link.allowed_hours = allowed_hours;
        updated_link.allowed_countries =
            helpers::parse_csv(data.get("allowed_countries").unwrap_or(&String::new()));
        updated_link.blocked_countries =
            helpers::parse_csv(data.get("blocked_countries").unwrap_or(&String::new()));
        updated_link.blocked_ips =
            helpers::parse_csv(data.get("blocked_ips").unwrap_or(&String::new()));
        updated_link.blocked_isps =
            helpers::parse_csv(data.get("blocked_isps").unwrap_or(&String::new()));
        // Regras fixas
        updated_link.cloaker_active = true;
        updated_link.ad_verify_mode = false;
        updated_link.block_vpn = false;
        updated_link.mobile_only = true;
        updated_link.require_facebook = true;
        updated_link.protection_total = false;
        updated_link.strict_param_required = true;
        updated_link.only_facebook_ads = true;
        updated_link.advanced_fingerprint = false;
        updated_link.ml_bot_detection = false;
        updated_link.dynamic_referrer_spoof = false;

        if !param_code.is_empty() && param_code != updated_link.param_code {
            helpers::set_param(&mut updated_link, &param_code);
        }

        if storage::save_link(&state.pool, &updated_link)
            .await
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(templates::error_page("Falha ao atualizar link no banco")),
            )
                .into_response();
        }

        {
            let mut links = state.db.links.write().await;
            links.insert(updated_link.id.clone(), updated_link);
        }

        return Redirect::to("/m4ciel7/links").into_response();
    }

    // GET → mostrar formulário de edição
    let links = state.db.links.read().await;
    let link = links.get(&id).cloned();
    drop(links);

    match link {
        Some(l) => Html(templates::edit_link_page(&param_name, &l)).into_response(),
        None => Html(templates::error_page("Link nao encontrado")).into_response(),
    }
}

// ==========================================
// DELETE LINK
// ==========================================
pub async fn handle_delete_link(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = q.get("id").cloned().unwrap_or_default();
    if id.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    if storage::delete_link(&state.pool, &id).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    {
        let mut links = state.db.links.write().await;
        links.remove(&id);
    }
    StatusCode::OK
}

// ==========================================
// CONFIG
// ==========================================
pub async fn handle_config(
    State(state): State<Arc<AppState>>,
    method: Method,
    form: Option<Form<HashMap<String, String>>>,
) -> impl IntoResponse {
    if method == Method::POST {
        let Some(Form(data)) = form else {
            return (
                StatusCode::BAD_REQUEST,
                Html(templates::error_page("Requisicao invalida")),
            )
                .into_response();
        };
        let param_name = data
            .get("param_name")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if let Err(e) = helpers::validate_param_name(&param_name) {
            return Html(templates::error_page(&e)).into_response();
        }
        let only_fb = data.get("only_fb_ads").map(|v| v == "on").unwrap_or(false);

        let mut updated_cfg = {
            let cfg = state.db.config.read().await;
            cfg.clone()
        };
        updated_cfg.param_name = param_name;
        updated_cfg.require_param = true;
        updated_cfg.only_facebook_ads = only_fb;

        if storage::save_config(&state.pool, &updated_cfg)
            .await
            .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(templates::error_page(
                    "Falha ao salvar configuracoes no banco",
                )),
            )
                .into_response();
        }

        {
            let mut cfg = state.db.config.write().await;
            *cfg = updated_cfg;
        }

        return Redirect::to("/m4ciel7/config").into_response();
    }

    let cfg = state.db.config.read().await;
    Html(templates::config_page(
        &cfg.param_name,
        cfg.only_facebook_ads,
    ))
    .into_response()
}

// Helpers
async fn get_link_stats(state: &Arc<AppState>) -> (i32, i32, i32) {
    let links = state.db.links.read().await;
    let mut total_links = 0;
    let mut total_clicks = 0;
    let mut total_blocked = 0;
    for link in links.values() {
        if link.active {
            total_links += 1;
        }
        total_clicks += link.clicks;
        total_blocked += link.blocked;
    }
    (total_links, total_clicks, total_blocked)
}

async fn is_slug_taken(state: &Arc<AppState>, slug: &str, exclude_id: &str) -> bool {
    let links = state.db.links.read().await;
    links
        .values()
        .any(|l| (exclude_id.is_empty() || l.id != exclude_id) && l.slug.eq_ignore_ascii_case(slug))
}
