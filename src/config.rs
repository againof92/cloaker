use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: String,
    pub secret_key: String,
    pub safe_page_url: String,
    pub block_bots: bool,
    pub require_param: bool,
    pub only_facebook_ads: bool,
    pub param_name: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: std::env::var("PORT").unwrap_or_else(|_| "8080".into()),
            secret_key: crate::engine::helpers::generate_code(32),
            safe_page_url: String::new(),
            block_bots: true,
            require_param: true,
            only_facebook_ads: true,
            param_name: "apx".into(),
        }
    }
}

pub fn admin_credentials() -> (String, String) {
    let user = std::env::var("CLOAKER_ADMIN_USER").unwrap_or_else(|_| "admin".into());
    let pass = std::env::var("CLOAKER_ADMIN_PASS").unwrap_or_else(|_| "change-me".into());
    (user.trim().to_string(), pass.trim().to_string())
}

pub fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("CLOAKER_DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/postgres".into())
}
