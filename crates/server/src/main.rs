use axum::{Router, http, routing};
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:clipstash.db?mode=rwc".into());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await?;

    clipstash_server::db::run_migrations(&pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // CORS for browser extension access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
        ])
        .allow_headers([http::header::CONTENT_TYPE]);

    // JSON API routes
    let api_routes = Router::new()
        .route("/articles", routing::get(clipstash_server::routes::list_articles))
        .route("/articles", routing::post(clipstash_server::routes::create_article))
        .route("/articles/{id}", routing::get(clipstash_server::routes::get_article))
        .route("/articles/{id}", routing::delete(clipstash_server::routes::delete_article))
        .route("/articles/{id}/tags", routing::put(clipstash_server::routes::update_tags))
        .layer(cors);

    // HTML page routes
    let page_routes = Router::new()
        .route("/", routing::get(clipstash_server::pages::index))
        .route("/preview-clip", routing::post(clipstash_server::pages::preview_clip))
        .route("/clip", routing::post(clipstash_server::pages::clip_article))
        .route("/article/{id}", routing::get(clipstash_server::pages::view_article))
        .route("/article/{id}/delete", routing::post(clipstash_server::pages::delete_article));

    let app = Router::new()
        .merge(page_routes)
        .nest("/api", api_routes)
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
