use crate::config::ServerConfig;
use crate::models::{AccessLog, AppDatabase, RedirectLink, SeenIP};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tracing::{error, info};

/// Cria pool de conexões PostgreSQL
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .min_connections(2)
        .max_lifetime(std::time::Duration::from_secs(3600))
        .idle_timeout(std::time::Duration::from_secs(1800))
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(database_url)
        .await?;
    info!("Storage: PostgreSQL conectado");
    Ok(pool)
}

/// Executa migrações no banco
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    let stmts = vec![
        r#"CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            port TEXT NOT NULL,
            secret_key TEXT NOT NULL,
            safe_page_url TEXT NOT NULL DEFAULT '',
            block_bots BOOLEAN NOT NULL,
            require_param BOOLEAN NOT NULL,
            only_facebook_ads BOOLEAN NOT NULL DEFAULT TRUE,
            param_name TEXT NOT NULL
        )"#,
        r#"CREATE TABLE IF NOT EXISTS links (
            id TEXT PRIMARY KEY,
            slug TEXT NOT NULL,
            param_hash TEXT NOT NULL,
            param_code TEXT NOT NULL DEFAULT '',
            offer_url TEXT NOT NULL,
            safe_page_url TEXT NOT NULL DEFAULT '',
            clicks INTEGER NOT NULL DEFAULT 0,
            blocked INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL,
            active BOOLEAN NOT NULL,
            cloaker_active BOOLEAN NOT NULL,
            ad_verify_mode BOOLEAN NOT NULL DEFAULT FALSE,
            max_clicks INTEGER NOT NULL DEFAULT 0,
            param_ttl INTEGER NOT NULL DEFAULT 0,
            allowed_countries JSONB NOT NULL DEFAULT '[]'::jsonb,
            blocked_countries JSONB NOT NULL DEFAULT '[]'::jsonb,
            blocked_ips JSONB NOT NULL DEFAULT '[]'::jsonb,
            blocked_isps JSONB NOT NULL DEFAULT '[]'::jsonb,
            block_vpn BOOLEAN NOT NULL DEFAULT FALSE,
            mobile_only BOOLEAN NOT NULL DEFAULT FALSE,
            allowed_hours TEXT NOT NULL DEFAULT '',
            require_facebook BOOLEAN NOT NULL DEFAULT FALSE,
            protection_total BOOLEAN NOT NULL DEFAULT FALSE,
            strict_param_required BOOLEAN NOT NULL DEFAULT FALSE,
            only_facebook_ads BOOLEAN NOT NULL DEFAULT FALSE,
            advanced_fingerprint BOOLEAN NOT NULL DEFAULT FALSE,
            ml_bot_detection BOOLEAN NOT NULL DEFAULT FALSE,
            dynamic_referrer_spoof BOOLEAN NOT NULL DEFAULT FALSE
        )"#,
        r#"CREATE TABLE IF NOT EXISTS logs (
            id BIGSERIAL PRIMARY KEY,
            timestamp TIMESTAMPTZ NOT NULL,
            link_id TEXT,
            ip TEXT,
            user_agent TEXT,
            referer TEXT,
            country TEXT,
            country_code TEXT,
            region TEXT,
            region_name TEXT,
            city TEXT,
            isp TEXT,
            is_vpn BOOLEAN,
            blocked BOOLEAN,
            reason TEXT,
            redirect_to TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS seen_ips (
            key TEXT PRIMARY KEY,
            ip TEXT NOT NULL,
            link_id TEXT NOT NULL,
            first_seen TIMESTAMPTZ,
            last_seen TIMESTAMPTZ,
            attempts INTEGER NOT NULL,
            blocked_at TIMESTAMPTZ,
            user_agent TEXT
        )"#,
        // Migrações incrementais (idempotentes)
        "ALTER TABLE links ADD COLUMN IF NOT EXISTS protection_total BOOLEAN NOT NULL DEFAULT FALSE",
        "ALTER TABLE links ADD COLUMN IF NOT EXISTS strict_param_required BOOLEAN NOT NULL DEFAULT FALSE",
        "ALTER TABLE links ADD COLUMN IF NOT EXISTS only_facebook_ads BOOLEAN NOT NULL DEFAULT FALSE",
        "ALTER TABLE links ADD COLUMN IF NOT EXISTS advanced_fingerprint BOOLEAN NOT NULL DEFAULT FALSE",
        "ALTER TABLE links ADD COLUMN IF NOT EXISTS ml_bot_detection BOOLEAN NOT NULL DEFAULT FALSE",
        "ALTER TABLE links ADD COLUMN IF NOT EXISTS dynamic_referrer_spoof BOOLEAN NOT NULL DEFAULT FALSE",
        "ALTER TABLE links ADD COLUMN IF NOT EXISTS mobile_only BOOLEAN NOT NULL DEFAULT FALSE",
        "ALTER TABLE config ADD COLUMN IF NOT EXISTS only_facebook_ads BOOLEAN NOT NULL DEFAULT TRUE",
        "ALTER TABLE logs ADD COLUMN IF NOT EXISTS country_code TEXT",
        "ALTER TABLE logs ADD COLUMN IF NOT EXISTS region TEXT",
        "ALTER TABLE logs ADD COLUMN IF NOT EXISTS region_name TEXT",
    ];

    for sql in stmts {
        if let Err(e) = sqlx::query(sql).execute(pool).await {
            // ALTER TABLE ... ADD COLUMN IF NOT EXISTS pode falhar em versões
            // antigas do PG sem IF NOT EXISTS. Ignora erros de coluna duplicada.
            let msg = e.to_string().to_lowercase();
            if msg.contains("already exists") || msg.contains("duplicate column") {
                continue;
            }
            return Err(e);
        }
    }
    info!("Storage: Migrações executadas");
    Ok(())
}

/// Carrega todos os dados do banco para memória
pub async fn load(pool: &PgPool, db: &AppDatabase) -> Result<(), sqlx::Error> {
    // 1. Config
    let config_row = sqlx::query(
        "SELECT port, secret_key, safe_page_url, block_bots, require_param, only_facebook_ads, param_name FROM config WHERE id = 1",
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = config_row {
        let mut cfg = db.config.write().await;
        cfg.port = row.get::<String, _>("port");
        cfg.secret_key = row.get::<String, _>("secret_key");
        cfg.safe_page_url = row.get::<Option<String>, _>("safe_page_url").unwrap_or_default();
        cfg.block_bots = row.get::<bool, _>("block_bots");
        cfg.require_param = row.get::<bool, _>("require_param");
        cfg.only_facebook_ads = row.get::<bool, _>("only_facebook_ads");
        cfg.param_name = row.get::<String, _>("param_name");
    } else {
        info!("Nenhuma config no DB, usando defaults.");
    }

    // 2. Links
    let link_rows = sqlx::query(
        r#"SELECT id, slug, param_hash, param_code, offer_url, safe_page_url,
           clicks, blocked, created_at, active, cloaker_active, ad_verify_mode,
           max_clicks, param_ttl, allowed_countries, blocked_countries,
           blocked_ips, blocked_isps, block_vpn, mobile_only, allowed_hours, require_facebook,
           COALESCE(protection_total, FALSE) as protection_total,
           COALESCE(strict_param_required, FALSE) as strict_param_required,
           COALESCE(only_facebook_ads, FALSE) as only_facebook_ads,
           COALESCE(advanced_fingerprint, FALSE) as advanced_fingerprint,
           COALESCE(ml_bot_detection, FALSE) as ml_bot_detection,
           COALESCE(dynamic_referrer_spoof, FALSE) as dynamic_referrer_spoof
           FROM links"#,
    )
    .fetch_all(pool)
    .await?;

    let mut links = db.links.write().await;
    links.clear();
    for row in link_rows {
        let link = RedirectLink {
            id: row.get("id"),
            slug: row.get("slug"),
            param_hash: row.get("param_hash"),
            param_code: row.get::<String, _>("param_code"),
            offer_url: row.get("offer_url"),
            safe_page_url: row.get::<String, _>("safe_page_url"),
            clicks: row.get("clicks"),
            blocked: row.get("blocked"),
            created_at: row.get("created_at"),
            active: row.get("active"),
            cloaker_active: row.get("cloaker_active"),
            ad_verify_mode: row.get("ad_verify_mode"),
            max_clicks: row.get("max_clicks"),
            param_ttl: row.get("param_ttl"),
            allowed_countries: decode_json_slice(row.get::<serde_json::Value, _>("allowed_countries")),
            blocked_countries: decode_json_slice(row.get::<serde_json::Value, _>("blocked_countries")),
            blocked_ips: decode_json_slice(row.get::<serde_json::Value, _>("blocked_ips")),
            blocked_isps: decode_json_slice(row.get::<serde_json::Value, _>("blocked_isps")),
            block_vpn: row.get("block_vpn"),
            mobile_only: row.get("mobile_only"),
            allowed_hours: row.get::<String, _>("allowed_hours"),
            require_facebook: row.get("require_facebook"),
            protection_total: row.get("protection_total"),
            strict_param_required: row.get("strict_param_required"),
            only_facebook_ads: row.get("only_facebook_ads"),
            advanced_fingerprint: row.get("advanced_fingerprint"),
            ml_bot_detection: row.get("ml_bot_detection"),
            dynamic_referrer_spoof: row.get("dynamic_referrer_spoof"),
        };
        links.insert(link.id.clone(), link);
    }

    // 3. Seen IPs
    let seen_rows = sqlx::query(
        "SELECT key, ip, link_id, first_seen, last_seen, attempts, blocked_at, user_agent FROM seen_ips",
    )
    .fetch_all(pool)
    .await?;

    let mut seen_ips = db.seen_ips.write().await;
    seen_ips.clear();
    for row in seen_rows {
        let key: String = row.get("key");
        let entry = SeenIP {
            ip: row.get("ip"),
            link_id: row.get("link_id"),
            first_seen: row.get::<Option<DateTime<Utc>>, _>("first_seen").unwrap_or_else(|| Utc::now()),
            last_seen: row.get::<Option<DateTime<Utc>>, _>("last_seen").unwrap_or_else(|| Utc::now()),
            attempts: row.get("attempts"),
            blocked_at: row.get::<Option<DateTime<Utc>>, _>("blocked_at"),
            user_agent: row.get::<Option<String>, _>("user_agent").unwrap_or_default(),
        };
        seen_ips.insert(key, entry);
    }

    Ok(())
}

// ==========================================
// Operações de escrita atômicas
// ==========================================

pub async fn save_config(pool: &PgPool, c: &ServerConfig) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO config (id, port, secret_key, safe_page_url, block_bots, require_param, only_facebook_ads, param_name)
           VALUES (1, $1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (id) DO UPDATE SET
             port = EXCLUDED.port,
             secret_key = EXCLUDED.secret_key,
             safe_page_url = EXCLUDED.safe_page_url,
             block_bots = EXCLUDED.block_bots,
             require_param = EXCLUDED.require_param,
             only_facebook_ads = EXCLUDED.only_facebook_ads,
             param_name = EXCLUDED.param_name"#,
    )
    .bind(&c.port)
    .bind(&c.secret_key)
    .bind(&c.safe_page_url)
    .bind(c.block_bots)
    .bind(c.require_param)
    .bind(c.only_facebook_ads)
    .bind(&c.param_name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn save_link(pool: &PgPool, l: &RedirectLink) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO links (
            id, slug, param_hash, param_code, offer_url, safe_page_url, clicks, blocked, created_at,
            active, cloaker_active, ad_verify_mode, max_clicks, param_ttl, allowed_countries, blocked_countries,
            blocked_ips, blocked_isps, block_vpn, mobile_only, allowed_hours, require_facebook,
            protection_total, strict_param_required, only_facebook_ads, advanced_fingerprint, ml_bot_detection, dynamic_referrer_spoof
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15::jsonb,$16::jsonb,$17::jsonb,$18::jsonb,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28
        )
        ON CONFLICT (id) DO UPDATE SET
            slug=EXCLUDED.slug, param_hash=EXCLUDED.param_hash, param_code=EXCLUDED.param_code,
            offer_url=EXCLUDED.offer_url, safe_page_url=EXCLUDED.safe_page_url,
            active=EXCLUDED.active, cloaker_active=EXCLUDED.cloaker_active,
            ad_verify_mode=EXCLUDED.ad_verify_mode, max_clicks=EXCLUDED.max_clicks, param_ttl=EXCLUDED.param_ttl,
            allowed_countries=EXCLUDED.allowed_countries, blocked_countries=EXCLUDED.blocked_countries,
            blocked_ips=EXCLUDED.blocked_ips, blocked_isps=EXCLUDED.blocked_isps, block_vpn=EXCLUDED.block_vpn,
            mobile_only=EXCLUDED.mobile_only, allowed_hours=EXCLUDED.allowed_hours, require_facebook=EXCLUDED.require_facebook,
            protection_total=EXCLUDED.protection_total, strict_param_required=EXCLUDED.strict_param_required,
            only_facebook_ads=EXCLUDED.only_facebook_ads, advanced_fingerprint=EXCLUDED.advanced_fingerprint,
            ml_bot_detection=EXCLUDED.ml_bot_detection, dynamic_referrer_spoof=EXCLUDED.dynamic_referrer_spoof"#,
    )
    .bind(&l.id).bind(&l.slug).bind(&l.param_hash).bind(&l.param_code)
    .bind(&l.offer_url).bind(&l.safe_page_url)
    .bind(l.clicks).bind(l.blocked).bind(l.created_at)
    .bind(l.active).bind(l.cloaker_active).bind(l.ad_verify_mode)
    .bind(l.max_clicks).bind(l.param_ttl)
    .bind(encode_json_slice(&l.allowed_countries))
    .bind(encode_json_slice(&l.blocked_countries))
    .bind(encode_json_slice(&l.blocked_ips))
    .bind(encode_json_slice(&l.blocked_isps))
    .bind(l.block_vpn).bind(l.mobile_only).bind(&l.allowed_hours).bind(l.require_facebook)
    .bind(l.protection_total).bind(l.strict_param_required).bind(l.only_facebook_ads)
    .bind(l.advanced_fingerprint).bind(l.ml_bot_detection).bind(l.dynamic_referrer_spoof)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_link(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM links WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn increment_link_stats(pool: &PgPool, link_id: &str, clicks: i32, blocked: i32) {
    if let Err(e) = sqlx::query("UPDATE links SET clicks = clicks + $1, blocked = blocked + $2 WHERE id = $3")
        .bind(clicks)
        .bind(blocked)
        .bind(link_id)
        .execute(pool)
        .await
    {
        error!("[ERRO-DB] Falha ao incrementar stats link {}: {}", link_id, e);
    }
}

pub async fn insert_log(pool: &PgPool, l: &AccessLog) {
    if let Err(e) = sqlx::query(
        r#"INSERT INTO logs (timestamp, link_id, ip, user_agent, referer, country, country_code, region, region_name, city, isp, is_vpn, blocked, reason, redirect_to)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)"#,
    )
    .bind(l.timestamp)
    .bind(&l.link_id)
    .bind(&l.ip)
    .bind(&l.user_agent)
    .bind(&l.referer)
    .bind(&l.country)
    .bind(&l.country_code)
    .bind(&l.region)
    .bind(&l.region_name)
    .bind(&l.city)
    .bind(&l.isp)
    .bind(l.is_vpn)
    .bind(l.blocked)
    .bind(&l.reason)
    .bind(&l.redirect_to)
    .execute(pool)
    .await
    {
        error!("[ERRO-DB] Falha ao salvar log: {}", e);
    }
}

pub async fn upsert_seen_ip(pool: &PgPool, key: &str, s: &SeenIP) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO seen_ips (key, ip, link_id, first_seen, last_seen, attempts, blocked_at, user_agent)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
           ON CONFLICT (key) DO UPDATE SET
             last_seen = EXCLUDED.last_seen,
             attempts = seen_ips.attempts + 1,
             blocked_at = COALESCE(EXCLUDED.blocked_at, seen_ips.blocked_at)"#,
    )
    .bind(key)
    .bind(&s.ip)
    .bind(&s.link_id)
    .bind(s.first_seen)
    .bind(s.last_seen)
    .bind(s.attempts)
    .bind(s.blocked_at)
    .bind(&s.user_agent)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_logs(pool: &PgPool, limit: i64) -> Result<Vec<AccessLog>, sqlx::Error> {
    let limit = if limit <= 0 { 1000 } else { limit };
    let rows = sqlx::query(
        r#"SELECT timestamp, link_id, ip, user_agent, referer, country, country_code, region, region_name, city, isp, is_vpn, blocked, reason, redirect_to
           FROM logs ORDER BY timestamp DESC LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut logs = Vec::with_capacity(rows.len());
    for row in rows {
        logs.push(AccessLog {
            timestamp: row.get("timestamp"),
            link_id: row.get::<Option<String>, _>("link_id").unwrap_or_default(),
            ip: row.get::<Option<String>, _>("ip").unwrap_or_default(),
            user_agent: row.get::<Option<String>, _>("user_agent").unwrap_or_default(),
            referer: row.get::<Option<String>, _>("referer").unwrap_or_default(),
            country: row.get::<Option<String>, _>("country").unwrap_or_default(),
            country_code: row.get::<Option<String>, _>("country_code").unwrap_or_default(),
            region: row.get::<Option<String>, _>("region").unwrap_or_default(),
            region_name: row.get::<Option<String>, _>("region_name").unwrap_or_default(),
            city: row.get::<Option<String>, _>("city").unwrap_or_default(),
            isp: row.get::<Option<String>, _>("isp").unwrap_or_default(),
            is_vpn: row.get::<Option<bool>, _>("is_vpn").unwrap_or(false),
            blocked: row.get::<Option<bool>, _>("blocked").unwrap_or(false),
            reason: row.get::<Option<String>, _>("reason").unwrap_or_default(),
            redirect_to: row.get::<Option<String>, _>("redirect_to").unwrap_or_default(),
        });
    }
    Ok(logs)
}

pub async fn clear_logs(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM logs").execute(pool).await?;
    Ok(())
}

// Helpers
fn encode_json_slice(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_else(|_| "[]".into())
}

fn decode_json_slice(val: serde_json::Value) -> Vec<String> {
    match val {
        serde_json::Value::Array(arr) => arr
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    }
}
