use crate::models::{AccessLog, GeoInfo, ParamCache, RedirectLink, SeenIP};
use crate::engine::bot_detect;
use crate::engine::helpers;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use crate::AppState;

const SEEN_IP_MAX_ATTEMPTS: i32 = 10;
const SEEN_IP_BLOCK_SECONDS: i64 = 60;

/// Resultado da validação de acesso
pub struct AccessResult {
    pub allowed: bool,
    pub reason: String,
}

/// Valida se o acesso deve ser redirecionado para a oferta ou bloqueado
pub async fn validate_access(
    state: &Arc<AppState>,
    _headers: &axum::http::HeaderMap,
    query_params: &HashMap<String, String>,
    client_ip: &str,
    user_agent: &str,
    referer: &str,
    geo: &GeoInfo,
    link: &RedirectLink,
) -> AccessResult {
    let config = state.db.config.read().await;
    let param_name = config.param_name.clone();
    let only_fb_ads = config.only_facebook_ads;
    drop(config);

    let param_code = query_params.get(&param_name).cloned().unwrap_or_default();
    let ua_lower = user_agent.to_lowercase();

    // Regras sempre ativas
    let strict_param = true;
    let mobile_only = true;
    let bot_protection = true;

    // ==========================================
    // PARÂMETRO OBRIGATÓRIO
    // ==========================================
    if strict_param {
        if param_code.is_empty() {
            return AccessResult {
                allowed: false,
                reason: "Parametro obrigatorio ausente".into(),
            };
        }
        if !helpers::verify_param(link, &param_code) {
            return AccessResult {
                allowed: false,
                reason: "Parametro invalido".into(),
            };
        }
        if link.param_ttl > 0 {
            if is_param_expired(state, link, &param_code).await {
                return AccessResult {
                    allowed: false,
                    reason: "Parametro expirado".into(),
                };
            }
        }
    }

    // IP temporariamente bloqueado
    if let Some(reason) = is_ip_temporarily_blocked(state, &link.id, client_ip).await {
        return AccessResult {
            allowed: false,
            reason,
        };
    }

    let mut allowed = true;
    let mut reason = String::new();

    // Somente tráfego de anúncio FB/IG
    if allowed && only_fb_ads {
        if !bot_detect::is_facebook_ad_traffic(referer, user_agent, query_params) {
            allowed = false;
            reason = "Acesso apenas via anuncio Facebook/Instagram".into();
        }
    }

    // Limite de cliques
    if allowed && link.max_clicks > 0 && link.clicks >= link.max_clicks {
        allowed = false;
        reason = "Limite de cliques atingido".into();
    }

    // Horário permitido
    if allowed && !link.allowed_hours.is_empty() && !helpers::is_within_allowed_hours(&link.allowed_hours) {
        allowed = false;
        reason = "Acesso fora do horario permitido".into();
    }

    // Filtros de país
    if allowed {
        let cc = geo.country_code.to_uppercase();
        if !cc.is_empty() && cc != "XX" {
            if !link.allowed_countries.is_empty()
                && !helpers::contains_ignore_case(&link.allowed_countries, &cc)
            {
                allowed = false;
                reason = "Pais nao permitido".into();
            }
            if allowed
                && !link.blocked_countries.is_empty()
                && helpers::contains_ignore_case(&link.blocked_countries, &cc)
            {
                allowed = false;
                reason = "Pais bloqueado".into();
            }
        }
    }

    // IPs bloqueados
    if allowed && !link.blocked_ips.is_empty() && helpers::is_ip_blocked(client_ip, &link.blocked_ips) {
        allowed = false;
        reason = "IP bloqueado".into();
    }

    // ISPs bloqueados
    if allowed && !link.blocked_isps.is_empty() && helpers::is_isp_blocked(&geo.isp, &geo.org, &link.blocked_isps) {
        allowed = false;
        reason = "ISP bloqueado".into();
    }

    // Apenas mobile
    if allowed && mobile_only && !bot_detect::is_mobile_device(user_agent) {
        allowed = false;
        reason = "Apenas dispositivos moveis permitidos".into();
    }

    // Anti-bot
    if allowed && bot_protection {
        if bot_detect::is_bot(&ua_lower) {
            allowed = false;
            reason = "Bot detectado (UA)".into();
        } else if bot_detect::is_automation_tool(&ua_lower) {
            allowed = false;
            reason = "Ferramenta de automacao detectada".into();
        }
    }

    // Registra seen IP
    let result = register_seen_ip(state, &link.id, client_ip, user_agent, allowed, &reason).await;
    AccessResult {
        allowed: result.0,
        reason: result.1,
    }
}

/// Verifica se IP está temporariamente bloqueado
async fn is_ip_temporarily_blocked(state: &Arc<AppState>, link_id: &str, ip: &str) -> Option<String> {
    let key = format!("{}:{}", link_id, ip);
    let seen_ips = state.db.seen_ips.read().await;
    if let Some(entry) = seen_ips.get(&key) {
        if let Some(blocked_at) = entry.blocked_at {
            let elapsed = Utc::now() - blocked_at;
            if elapsed < Duration::seconds(SEEN_IP_BLOCK_SECONDS) {
                let remaining = (SEEN_IP_BLOCK_SECONDS - elapsed.num_seconds()).max(1);
                return Some(format!(
                    "IP bloqueado por excesso de tentativas. Aguarde {}s",
                    remaining
                ));
            }
        }
    }
    None
}

/// Registra IP visto e aplica rate limiting
async fn register_seen_ip(
    state: &Arc<AppState>,
    link_id: &str,
    ip: &str,
    user_agent: &str,
    allowed: bool,
    reason: &str,
) -> (bool, String) {
    let now = Utc::now();
    let key = format!("{}:{}", link_id, ip);

    let mut seen_ips = state.db.seen_ips.write().await;
    let entry = seen_ips.entry(key.clone()).or_insert_with(|| SeenIP {
        ip: ip.to_string(),
        link_id: link_id.to_string(),
        first_seen: now,
        last_seen: now,
        attempts: 0,
        blocked_at: None,
        user_agent: String::new(),
    });

    // Limpa bloqueio expirado
    if let Some(blocked_at) = entry.blocked_at {
        if (now - blocked_at).num_seconds() >= SEEN_IP_BLOCK_SECONDS {
            entry.blocked_at = None;
            entry.attempts = 0;
        }
    }

    // Persiste no DB (fire and forget)
    let entry_clone = entry.clone();
    let pool = state.pool.clone();
    tokio::spawn(async move {
        let _ = crate::storage::upsert_seen_ip(&pool, &key, &entry_clone).await;
    });

    if allowed {
        entry.last_seen = now;
        entry.attempts = 0;
        entry.user_agent = user_agent.to_string();
        return (true, reason.to_string());
    }

    entry.attempts += 1;
    entry.last_seen = now;
    entry.user_agent = user_agent.to_string();

    if entry.attempts >= SEEN_IP_MAX_ATTEMPTS {
        entry.blocked_at = Some(now);
        return (
            false,
            "IP bloqueado! Limite de 10 tentativas atingido. Aguarde 1 minuto".into(),
        );
    }

    (false, reason.to_string())
}

/// Verifica se o parâmetro expirou
async fn is_param_expired(state: &Arc<AppState>, link: &RedirectLink, param: &str) -> bool {
    if link.param_ttl <= 0 || param.is_empty() {
        return false;
    }
    let now = Utc::now();
    let key = link.id.clone();

    let mut cache = state.db.param_cache.write().await;
    let entry = cache.entry(key).or_insert_with(|| ParamCache {
        code: param.to_string(),
        created_at: now,
        uses: 0,
    });

    if entry.code != param {
        *entry = ParamCache {
            code: param.to_string(),
            created_at: now,
            uses: 0,
        };
    }

    let expired = (now - entry.created_at).num_minutes() > link.param_ttl as i64;
    if !expired {
        entry.uses += 1;
    }
    expired
}

/// Constrói contagem de acessos por estado (BR) para o mapa
pub fn build_state_counts(logs: &[AccessLog]) -> HashMap<String, i32> {
    let mut counts: HashMap<String, i32> = HashMap::new();
    let mut seen: HashMap<String, std::collections::HashSet<String>> = HashMap::new();

    for entry in logs {
        if entry.blocked {
            continue;
        }
        let cc = entry.country_code.trim().to_uppercase();
        if cc.is_empty() || cc == "XX" {
            let cn = entry.country.trim().to_lowercase();
            if cn != "brasil" && cn != "brazil" {
                continue;
            }
        } else if cc != "BR" {
            continue;
        }

        let mut code = entry.region.trim().to_uppercase();
        if code.is_empty() && !entry.region_name.is_empty() {
            code = helpers::state_code_from_name(&entry.region_name);
        }
        if code.is_empty() {
            continue;
        }

        let ip = entry.ip.trim();
        if ip.is_empty() {
            continue;
        }

        let date_key = entry.timestamp.format("%Y-%m-%d").to_string();
        let unique_key = format!("{}|{}", date_key, ip);

        seen.entry(code.clone())
            .or_default()
            .insert(unique_key);
    }

    for (code, ips) in &seen {
        counts.insert(code.clone(), ips.len() as i32);
    }
    counts
}
