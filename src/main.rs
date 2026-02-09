use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::Router;
use axum_extra::extract::cookie::CookieJar;
use tokio::sync::broadcast;
use tracing_subscriber::EnvFilter;

mod auth;
mod config;
mod engine;
mod handlers;
mod models;
mod storage;
mod templates;

// ========================================
// AppState compartilhado por todos handlers
// ========================================
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub db: Arc<models::AppDatabase>,
    pub sessions: auth::Sessions,
    pub geo_cache: Arc<engine::geoip::GeoCache>,
    pub log_tx: broadcast::Sender<models::AccessLog>,
    pub clear_tx: broadcast::Sender<()>,
    pub brazil_map_svg: String,
    pub http_client: reqwest::Client,
}

// ========================================
// Middleware de autenticação para rotas admin
// ========================================
async fn require_auth(
    jar: CookieJar,
    state: axum::extract::State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    if auth::is_authenticated(&jar, &state.sessions).await {
        next.run(req).await.into_response()
    } else {
        Redirect::to("/m4ciel7/login").into_response()
    }
}

// ========================================
// Entry-point
// ========================================
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing / logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Banco de dados
    let db_url = config::database_url();
    let pool = storage::create_pool(&db_url).await?;
    storage::migrate(&pool).await?;

    let db = Arc::new(models::AppDatabase::new());
    storage::load(&pool, &db).await?;

    // Broadcast channels (SSE)
    let (log_tx, _) = broadcast::channel::<models::AccessLog>(256);
    let (clear_tx, _) = broadcast::channel::<()>(16);

    // Mapa do Brasil (SVG embutido no binário)
    let brazil_map_svg = include_str!("../br_admin1.svg").to_string();

    // HTTP client reutilizável
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Sessões e cache GeoIP
    let sessions = auth::new_sessions();
    let geo_cache = Arc::new(engine::geoip::GeoCache::new(http_client.clone()));

    let state = Arc::new(AppState {
        pool: pool.clone(),
        db: db.clone(),
        sessions: sessions.clone(),
        geo_cache: geo_cache.clone(),
        log_tx,
        clear_tx,
        brazil_map_svg,
        http_client,
    });

    // Background: limpeza periódica (cache, sessões, seen_ips, param_cache)
    spawn_cleanup(state.clone());

    // ---- Rotas públicas ----
    let public_routes = Router::new()
        .route("/", get(handlers::redirect::handle_home))
        .route(
            "/go/:slug",
            get(handlers::redirect::handle_redirect).post(handlers::redirect::handle_redirect),
        )
        .route(
            "/m4ciel7/login",
            get(handlers::admin::handle_login).post(handlers::admin::handle_login),
        )
        .route("/api/geoip", get(handlers::api::handle_geoip_test));

    // ---- Rotas admin (protegidas por auth middleware) ----
    let admin_routes = Router::new()
        .route("/m4ciel7", get(handlers::admin::handle_dashboard))
        .route("/m4ciel7/logout", get(handlers::admin::handle_logout))
        .route("/m4ciel7/links", get(handlers::admin::handle_links))
        .route(
            "/m4ciel7/create",
            get(handlers::admin::handle_create_link).post(handlers::admin::handle_create_link),
        )
        .route(
            "/m4ciel7/edit",
            get(handlers::admin::handle_edit_link).post(handlers::admin::handle_edit_link),
        )
        .route("/m4ciel7/delete", post(handlers::admin::handle_delete_link))
        .route("/m4ciel7/stats", get(handlers::api::handle_stats))
        .route("/m4ciel7/map-stats", get(handlers::api::handle_map_stats))
        .route("/m4ciel7/logs", get(handlers::api::handle_logs))
        .route(
            "/m4ciel7/logs/stream",
            get(handlers::api::handle_logs_stream),
        )
        .route(
            "/m4ciel7/logs/clear",
            post(handlers::api::handle_logs_clear),
        )
        .route(
            "/m4ciel7/config",
            get(handlers::admin::handle_config).post(handlers::admin::handle_config),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let app = public_routes.merge(admin_routes).with_state(state);

    // Porta
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Cloaker Rust iniciando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

// ========================================
// Task de limpeza em background
// ========================================
fn spawn_cleanup(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;

            // Limpa cache GeoIP expirado
            state.geo_cache.cleanup().await;

            // Limpa sessões expiradas (> 24 h)
            {
                let mut sess = state.sessions.write().await;
                let cutoff = chrono::Utc::now() - chrono::TimeDelta::hours(24);
                sess.retain(|_, ts| *ts > cutoff);
            }

            // Limpa seen_ips antigos (ou bloqueio já expirado)
            {
                let mut seen = state.db.seen_ips.write().await;
                let now = chrono::Utc::now();
                let seen_cutoff = now - chrono::TimeDelta::hours(24);
                seen.retain(|_, ip| {
                    let recently_seen = ip.last_seen > seen_cutoff;
                    let block_still_active = ip
                        .blocked_at
                        .map(|blocked_at| {
                            now.signed_duration_since(blocked_at).num_seconds() <= 120
                        })
                        .unwrap_or(false);
                    recently_seen || block_still_active
                });
            }

            // Limpa param_cache antigo (> 24h)
            {
                let mut cache = state.db.param_cache.write().await;
                let now = chrono::Utc::now();
                cache.retain(|_, pc| (now - pc.created_at).num_hours() < 24);
            }
        }
    });
}
