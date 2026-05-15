use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use clipstash_shared::models::{CreateArticleRequest, SearchQuery, UpdateTagsRequest};
use uuid::Uuid;

use crate::{auth::AppState, db, extractor};

pub async fn create_article(
    State(state): State<AppState>,
    Json(req): Json<CreateArticleRequest>,
) -> impl IntoResponse {
    let content = match extractor::fetch_and_parse(&req.url).await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            );
        }
    };

    let existing_tags = crate::suggest_tags::get_all_existing_tags(&state.pool)
        .await
        .unwrap_or_default();

    let tags =
        crate::suggest_tags::suggest_tags(&content.raw_html, &content.text, &existing_tags).await;

    let article = extractor::into_article(content, tags);

    match db::insert_article(&state.pool, &article).await {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!(article))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn get_article(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match db::get_article(&state.pool, id).await {
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
    State(state): State<AppState>,
    Query(search): Query<SearchQuery>,
) -> impl IntoResponse {
    let result = match (&search.q, &search.tag) {
        (Some(q), _) if !q.is_empty() => db::search_articles(&state.pool, q).await,
        (_, Some(tag)) if !tag.is_empty() => db::search_by_tag(&state.pool, tag).await,
        _ => db::list_articles(&state.pool).await,
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
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTagsRequest>,
) -> impl IntoResponse {
    match db::update_tags(&state.pool, id, &req.tags).await {
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
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::delete_article(&state.pool, id).await {
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

#[cfg(test)]
mod tests {
    use crate::suggest_tags;

    #[tokio::test]
    async fn suggest_tags_used_with_empty_vocabulary() {
        let html = r#"<html><head>
            <meta name="keywords" content="rust, async, web">
        </head><body><p>Article about Rust async web programming.</p></body></html>"#;
        let text = "Article about Rust async web programming.";
        let existing: Vec<String> = vec![];

        let tags = suggest_tags::suggest_tags(html, text, &existing).await;

        assert!(
            tags.contains(&"rust".to_string()),
            "expected 'rust' in {tags:?}"
        );
        assert!(
            tags.contains(&"async".to_string()),
            "expected 'async' in {tags:?}"
        );
        assert!(
            tags.contains(&"web".to_string()),
            "expected 'web' in {tags:?}"
        );
    }

    #[tokio::test]
    async fn suggest_tags_merges_vocabulary_matches() {
        let html = r#"<html><head>
            <meta name="keywords" content="rust">
        </head><body><p>A deep dive into async programming with Tokio.</p></body></html>"#;
        let text = "A deep dive into async programming with Tokio.";
        let existing = vec!["tokio".to_string(), "python".to_string()];

        let tags = suggest_tags::suggest_tags(html, text, &existing).await;

        assert!(
            tags.contains(&"rust".to_string()),
            "expected meta tag 'rust'"
        );
        assert!(
            tags.contains(&"tokio".to_string()),
            "expected vocabulary match 'tokio'"
        );
        assert!(
            !tags.contains(&"python".to_string()),
            "python should not match"
        );
    }
}
