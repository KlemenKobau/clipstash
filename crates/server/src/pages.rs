use axum::{
    Form,
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect},
};
use clipstash_frontend::templates::{
    ArticleTemplate, ConfirmClipTemplate, ErrorTemplate, IndexTemplate,
};
use clipstash_shared::models::SearchQuery;
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

use crate::{db, extractor, suggest_tags};

pub async fn index(
    State(pool): State<SqlitePool>,
    Query(search): Query<SearchQuery>,
) -> impl IntoResponse {
    let result = match (&search.q, &search.tag) {
        (Some(q), _) if !q.is_empty() => db::search_articles(&pool, q).await,
        (_, Some(tag)) if !tag.is_empty() => db::search_by_tag(&pool, tag).await,
        _ => db::list_articles(&pool).await,
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
    State(pool): State<SqlitePool>,
    Form(form): Form<PreviewForm>,
) -> impl IntoResponse {
    let content = match extractor::fetch_and_parse(&form.url).await {
        Ok(c) => c,
        Err(e) => return error_page(&e.to_string()),
    };

    let existing_tags = suggest_tags::get_all_existing_tags(&pool)
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
    State(pool): State<SqlitePool>,
    Form(form): Form<ClipForm>,
) -> impl IntoResponse {
    let tags: Vec<String> = form
        .tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    match extractor::fetch_and_extract(&form.url, tags).await {
        Ok(article) => match db::insert_article(&pool, &article).await {
            Ok(()) => Redirect::to("/").into_response(),
            Err(e) => error_page(&e.to_string()),
        },
        Err(e) => error_page(&e.to_string()),
    }
}

pub async fn view_article(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::get_article(&pool, id).await {
        Ok(article) => {
            let template = ArticleTemplate { article };
            Html(template.to_string()).into_response()
        }
        Err(e) => error_page(&e.to_string()),
    }
}

pub async fn delete_article(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::delete_article(&pool, id).await {
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
