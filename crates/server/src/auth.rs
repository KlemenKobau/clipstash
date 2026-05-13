use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use sqlx::sqlite::SqlitePool;
use subtle::ConstantTimeEq;
use tower_sessions::Session;

/// Key used to mark a session as authenticated.
pub const SESSION_KEY: &str = "authenticated";

/// Combined application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub auth: AuthConfig,
}

/// Auth configuration loaded from environment variables at startup.
#[derive(Clone)]
pub struct AuthConfig {
    pub username: String,
    pub password: String,
    pub api_key: String,
}

impl AuthConfig {
    /// Load from environment. Panics at startup if any variable is missing.
    pub fn from_env() -> Self {
        let username = std::env::var("CLIPSTASH_USER").expect("CLIPSTASH_USER env var must be set");
        let password =
            std::env::var("CLIPSTASH_PASSWORD").expect("CLIPSTASH_PASSWORD env var must be set");
        let api_key =
            std::env::var("CLIPSTASH_API_KEY").expect("CLIPSTASH_API_KEY env var must be set");
        Self {
            username,
            password,
            api_key,
        }
    }

    /// Constant-time username + password check to prevent timing attacks.
    pub fn verify_password(&self, username: &str, password: &str) -> bool {
        let user_ok = username.as_bytes().ct_eq(self.username.as_bytes());
        let pass_ok = password.as_bytes().ct_eq(self.password.as_bytes());
        (user_ok & pass_ok).into()
    }

    /// Constant-time API key check.
    pub fn verify_api_key(&self, key: &str) -> bool {
        key.as_bytes().ct_eq(self.api_key.as_bytes()).into()
    }
}

/// Middleware: require a valid `Authorization: Bearer <api_key>` header.
/// Used on `/api/*` routes.
pub async fn require_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Response {
    let authorized = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|key| state.auth.verify_api_key(key))
        .unwrap_or(false);

    if authorized {
        next.run(request).await
    } else {
        (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}

/// Middleware: require an authenticated session.
/// Used on all web UI routes except `/login`.
pub async fn require_session(session: Session, request: Request<Body>, next: Next) -> Response {
    let authenticated: bool = session
        .get(SESSION_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or(false);

    if authenticated {
        next.run(request).await
    } else {
        Redirect::to("/login").into_response()
    }
}
