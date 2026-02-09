use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::config::ServerConfig;

// ========== Link de redirecionamento ==========
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectLink {
    pub id: String,
    pub slug: String,
    pub param_hash: String,
    pub param_code: String,
    pub offer_url: String,
    pub safe_page_url: String,
    pub clicks: i32,
    pub blocked: i32,
    pub created_at: DateTime<Utc>,
    pub active: bool,
    pub cloaker_active: bool,
    pub ad_verify_mode: bool,
    pub max_clicks: i32,
    pub param_ttl: i32,
    pub allowed_countries: Vec<String>,
    pub blocked_countries: Vec<String>,
    pub blocked_ips: Vec<String>,
    pub blocked_isps: Vec<String>,
    pub block_vpn: bool,
    pub mobile_only: bool,
    pub allowed_hours: String,
    pub require_facebook: bool,
    pub protection_total: bool,
    pub strict_param_required: bool,
    pub only_facebook_ads: bool,
    pub advanced_fingerprint: bool,
    pub ml_bot_detection: bool,
    pub dynamic_referrer_spoof: bool,
}

// ========== Log de acesso ==========
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLog {
    pub timestamp: DateTime<Utc>,
    pub link_id: String,
    pub ip: String,
    pub user_agent: String,
    pub referer: String,
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub region_name: String,
    pub city: String,
    pub isp: String,
    pub is_vpn: bool,
    pub blocked: bool,
    pub reason: String,
    pub redirect_to: String,
}

// ========== GeoIP ==========
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoInfo {
    pub status: String,
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub region_name: String,
    pub city: String,
    pub isp: String,
    pub org: String,
    #[serde(rename = "as")]
    pub as_info: String,
    pub proxy: bool,
    pub hosting: bool,
}

impl Default for GeoInfo {
    fn default() -> Self {
        Self {
            status: String::new(),
            country: "Desconhecido".into(),
            country_code: "XX".into(),
            region: String::new(),
            region_name: String::new(),
            city: String::new(),
            isp: String::new(),
            org: String::new(),
            as_info: String::new(),
            proxy: false,
            hosting: false,
        }
    }
}

// ========== Cache ==========
#[derive(Debug, Clone)]
pub struct GeoCacheEntry {
    pub info: GeoInfo,
    pub expires: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ParamCache {
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub uses: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeenIP {
    pub ip: String,
    pub link_id: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub attempts: i32,
    pub blocked_at: Option<DateTime<Utc>>,
    pub user_agent: String,
}

// ========== Estado em memória ==========
pub struct AppDatabase {
    pub links: RwLock<HashMap<String, RedirectLink>>,
    pub config: RwLock<ServerConfig>,
    pub param_cache: RwLock<HashMap<String, ParamCache>>,
    pub seen_ips: RwLock<HashMap<String, SeenIP>>,
}

impl AppDatabase {
    pub fn new() -> Self {
        Self {
            links: RwLock::new(HashMap::new()),
            config: RwLock::new(ServerConfig::default()),
            param_cache: RwLock::new(HashMap::new()),
            seen_ips: RwLock::new(HashMap::new()),
        }
    }
}
