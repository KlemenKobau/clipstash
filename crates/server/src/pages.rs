use axum::{
    Form,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use clipstash_frontend::templates::{
    ArticleTemplate, ConfirmClipTemplate, ErrorTemplate, IndexTemplate, LoginTemplate,
};
use clipstash_shared::models::SearchQuery;
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    auth::{AppState, SESSION_KEY},
    db, extractor, suggest_tags,
};

pub async fn index(
    State(state): State<AppState>,
    Query(search): Query<SearchQuery>,
) -> impl IntoResponse {
    let result = match (&search.q, &search.tag) {
        (Some(q), _) if !q.is_empty() => db::search_articles(&state.pool, q).await,
        (_, Some(tag)) if !tag.is_empty() => db::search_by_tag(&state.pool, tag).await,
        _ => db::list_articles(&state.pool).await,
    };

    match result {
        Ok(articles) => {
            let template = IndexTemplate {
                articles,
                search: search.q.unwrap_or_default(),
                tag_filter: search.tag.unwrap_or_default(),
                error: None,
                success: None,
            };
            Html(template.to_string()).into_response()
        }
        Err(e) => {
            let template = ErrorTemplate {
                message: e.to_string(),
            };
            Html(template.to_string()).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct PreviewForm {
    pub url: String,
}

pub async fn preview_clip(
    State(state): State<AppState>,
    Form(form): Form<PreviewForm>,
) -> impl IntoResponse {
    let content = match extractor::fetch_and_parse(&form.url).await {
        Ok(c) => c,
        Err(e) => return error_page(&e.to_string()),
    };

    let existing_tags = suggest_tags::get_all_existing_tags(&state.pool)
        .await
        .unwrap_or_default();

    let suggestions =
        suggest_tags::suggest_tags(&content.raw_html, &content.text, &existing_tags).await;

    let template = ConfirmClipTemplate {
        url: content.url,
        title: content.title,
        domain: content.domain,
        excerpt: content.excerpt,
        suggested_tags: suggestions.join(", "),
    };
    Html(template.to_string()).into_response()
}

#[derive(Deserialize)]
pub struct ClipForm {
    pub url: String,
    pub tags: String,
}

pub async fn clip_article(
    State(state): State<AppState>,
    Form(form): Form<ClipForm>,
) -> impl IntoResponse {
    let tags: Vec<String> = form
        .tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    match extractor::fetch_and_extract(&form.url, tags).await {
        Ok(article) => match db::insert_article(&state.pool, &article).await {
            Ok(()) => Redirect::to("/").into_response(),
            Err(e) => error_page(&e.to_string()),
        },
        Err(e) => error_page(&e.to_string()),
    }
}

pub async fn view_article(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::get_article(&state.pool, id).await {
        Ok(article) => {
            let template = ArticleTemplate { article };
            Html(template.to_string()).into_response()
        }
        Err(e) => error_page(&e.to_string()),
    }
}

pub async fn delete_article(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::delete_article(&state.pool, id).await {
        Ok(()) => Redirect::to("/").into_response(),
        Err(e) => error_page(&e.to_string()),
    }
}

fn error_page(message: &str) -> axum::response::Response {
    let template = ErrorTemplate {
        message: message.to_string(),
    };
    Html(template.to_string()).into_response()
}

pub async fn login_page() -> impl IntoResponse {
    Html(LoginTemplate { error: None }.to_string())
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login_submit(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    if state.auth.verify_password(&form.username, &form.password) {
        session.insert(SESSION_KEY, true).await.ok();
        Redirect::to("/").into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Html(
                LoginTemplate {
                    error: Some("Invalid username or password.".into()),
                }
                .to_string(),
            ),
        )
            .into_response()
    }
}

pub async fn logout(session: Session) -> impl IntoResponse {
    session.flush().await.ok();
    Redirect::to("/login").into_response()
}
