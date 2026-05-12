use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use clipstash_shared::models::{CreateArticleRequest, SearchQuery, UpdateTagsRequest};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

use crate::{db, extractor};

pub async fn create_article(
    State(pool): State<SqlitePool>,
    Json(req): Json<CreateArticleRequest>,
) -> impl IntoResponse {
    match extractor::fetch_and_extract(&req.url, req.tags).await {
        Ok(article) => match db::insert_article(&pool, &article).await {
            Ok(()) => (StatusCode::CREATED, Json(serde_json::json!(article))),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ),
        },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn get_article(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::get_article(&pool, id).await {
        Ok(article) => (StatusCode::OK, Json(serde_json::json!(article))),
        Err(clipstash_shared::error::ClipstashError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Article not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn list_articles(
    State(pool): State<SqlitePool>,
    Query(search): Query<SearchQuery>,
) -> impl IntoResponse {
    let result = match (&search.q, &search.tag) {
        (Some(q), _) if !q.is_empty() => db::search_articles(&pool, q).await,
        (_, Some(tag)) if !tag.is_empty() => db::search_by_tag(&pool, tag).await,
        _ => db::list_articles(&pool).await,
    };

    match result {
        Ok(articles) => (StatusCode::OK, Json(serde_json::json!(articles))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn update_tags(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTagsRequest>,
) -> impl IntoResponse {
    match db::update_tags(&pool, id, &req.tags).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(clipstash_shared::error::ClipstashError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Article not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete_article(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::delete_article(&pool, id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(clipstash_shared::error::ClipstashError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Article not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
