use axum::{Router, http, middleware, routing};
use clipstash_server::auth::{AppState, AuthConfig};
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_sessions::{MemoryStore, SessionManagerLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env if present (optional).
    let _ = dotenvy::dotenv();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:clipstash.db?mode=rwc".into());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;

    clipstash_server::db::run_migrations(&pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let state = AppState {
        pool,
        auth: AuthConfig::from_env(),
    };

    // In-memory session store (sessions are lost on restart)
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

    // CORS for browser extension access — include Authorization header
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
        ])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION]);

    // JSON API routes — protected by API key
    let api_routes = Router::new()
        .route(
            "/articles",
            routing::get(clipstash_server::routes::list_articles),
        )
        .route(
            "/articles",
            routing::post(clipstash_server::routes::create_article),
        )
        .route(
            "/articles/{id}",
            routing::get(clipstash_server::routes::get_article),
        )
        .route(
            "/articles/{id}",
            routing::delete(clipstash_server::routes::delete_article),
        )
        .route(
            "/articles/{id}/tags",
            routing::put(clipstash_server::routes::update_tags),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            clipstash_server::auth::require_api_key,
        ))
        .layer(cors);

    // Public routes — no session required
    let public_routes = Router::new()
        .route("/login", routing::get(clipstash_server::pages::login_page))
        .route(
            "/login",
            routing::post(clipstash_server::pages::login_submit),
        )
        .route("/logout", routing::post(clipstash_server::pages::logout));

    // Protected HTML page routes — require valid session
    let protected_routes = Router::new()
        .route("/", routing::get(clipstash_server::pages::index))
        .route(
            "/preview-clip",
            routing::post(clipstash_server::pages::preview_clip),
        )
        .route(
            "/clip",
            routing::post(clipstash_server::pages::clip_article),
        )
        .route(
            "/article/{id}",
            routing::get(clipstash_server::pages::view_article),
        )
        .route(
            "/article/{id}/delete",
            routing::post(clipstash_server::pages::delete_article),
        )
        .layer(middleware::from_fn(clipstash_server::auth::require_session));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .nest("/api", api_routes)
        .nest_service("/static", ServeDir::new("static"))
        .layer(session_layer)
        .with_state(state);

    let host: std::net::IpAddr = std::env::var("CLIPSTASH_HOST")
        .unwrap_or_else(|_| "127.0.0.1".into())
        .parse()
        .expect("Invalid CLIPSTASH_HOST");
    let port: u16 = std::env::var("CLIPSTASH_PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()
        .expect("Invalid CLIPSTASH_PORT");
    let addr = std::net::SocketAddr::from((host, port));
    println!("Listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
