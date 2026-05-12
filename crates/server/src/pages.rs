use axum::{
    Form,
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect},
};
use clipstash_frontend::templates::{ArticleTemplate, ErrorTemplate, IndexTemplate};
use clipstash_shared::models::SearchQuery;
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

use crate::{db, extractor};

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
