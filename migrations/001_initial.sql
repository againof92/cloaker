-- Cloaker: Schema inicial
CREATE TABLE IF NOT EXISTS config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    port TEXT NOT NULL,
    secret_key TEXT NOT NULL,
    safe_page_url TEXT NOT NULL DEFAULT '',
    block_bots BOOLEAN NOT NULL,
    require_param BOOLEAN NOT NULL,
    only_facebook_ads BOOLEAN NOT NULL DEFAULT TRUE,
    param_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS links (
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
);

CREATE TABLE IF NOT EXISTS logs (
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
);

CREATE TABLE IF NOT EXISTS seen_ips (
    key TEXT PRIMARY KEY,
    ip TEXT NOT NULL,
    link_id TEXT NOT NULL,
    first_seen TIMESTAMPTZ,
    last_seen TIMESTAMPTZ,
    attempts INTEGER NOT NULL,
    blocked_at TIMESTAMPTZ,
    user_agent TEXT
);
