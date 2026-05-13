use chrono::Utc;
use clipstash_server::db;
use clipstash_shared::error::ClipstashError;
use clipstash_shared::models::Article;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use uuid::Uuid;

async fn setup_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .expect("Failed to enable foreign keys");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

fn make_article(url: &str, title: &str, tags: Vec<String>) -> Article {
    let now = Utc::now();
    Article {
        id: Uuid::new_v4(),
        url: url.to_string(),
        title: title.to_string(),
        domain: "example.com".to_string(),
        excerpt: format!("Excerpt for {title}"),
        content: format!("<p>Content for {title}</p>"),
        tags,
        created_at: now,
        updated_at: now,
    }
}

#[tokio::test]
async fn insert_and_get_article() {
    let pool = setup_db().await;
    let article = make_article("https://example.com/1", "Test Article", vec!["rust".into()]);
    let id = article.id;

    db::insert_article(&pool, &article).await.unwrap();

    let fetched = db::get_article(&pool, id).await.unwrap();
    assert_eq!(fetched.title, "Test Article");
    assert_eq!(fetched.url, "https://example.com/1");
    assert_eq!(fetched.domain, "example.com");
    assert_eq!(fetched.tags, vec!["rust".to_string()]);
}

#[tokio::test]
async fn get_nonexistent_article_returns_not_found() {
    let pool = setup_db().await;
    let result = db::get_article(&pool, Uuid::new_v4()).await;
    assert!(matches!(result, Err(ClipstashError::NotFound)));
}

#[tokio::test]
async fn list_articles_returns_all_in_order() {
    let pool = setup_db().await;

    let a1 = make_article("https://example.com/1", "First", vec![]);
    let a2 = make_article("https://example.com/2", "Second", vec!["tag".into()]);
    db::insert_article(&pool, &a1).await.unwrap();
    db::insert_article(&pool, &a2).await.unwrap();

    let list = db::list_articles(&pool).await.unwrap();
    assert_eq!(list.len(), 2);
    // Most recent first
    assert_eq!(list[0].title, "Second");
    assert_eq!(list[1].title, "First");
}

#[tokio::test]
async fn delete_article_removes_it() {
    let pool = setup_db().await;
    let article = make_article("https://example.com/1", "To Delete", vec!["tag".into()]);
    let id = article.id;

    db::insert_article(&pool, &article).await.unwrap();
    db::delete_article(&pool, id).await.unwrap();

    let result = db::get_article(&pool, id).await;
    assert!(matches!(result, Err(ClipstashError::NotFound)));
}

#[tokio::test]
async fn delete_nonexistent_article_returns_not_found() {
    let pool = setup_db().await;
    let result = db::delete_article(&pool, Uuid::new_v4()).await;
    assert!(matches!(result, Err(ClipstashError::NotFound)));
}

#[tokio::test]
async fn update_tags_replaces_existing() {
    let pool = setup_db().await;
    let article = make_article("https://example.com/1", "Tagged", vec!["old-tag".into()]);
    let id = article.id;

    db::insert_article(&pool, &article).await.unwrap();
    db::update_tags(&pool, id, &["new-tag".into(), "another".into()])
        .await
        .unwrap();

    let fetched = db::get_article(&pool, id).await.unwrap();
    assert_eq!(
        fetched.tags,
        vec!["another".to_string(), "new-tag".to_string()]
    );
}

#[tokio::test]
async fn update_tags_on_nonexistent_article_returns_not_found() {
    let pool = setup_db().await;
    let result = db::update_tags(&pool, Uuid::new_v4(), &["tag".into()]).await;
    assert!(matches!(result, Err(ClipstashError::NotFound)));
}

#[tokio::test]
async fn search_by_tag_filters_correctly() {
    let pool = setup_db().await;

    let a1 = make_article("https://example.com/1", "Rust Article", vec!["rust".into()]);
    let a2 = make_article("https://example.com/2", "Go Article", vec!["go".into()]);
    db::insert_article(&pool, &a1).await.unwrap();
    db::insert_article(&pool, &a2).await.unwrap();

    let results = db::search_by_tag(&pool, "rust").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Article");
}

#[tokio::test]
async fn full_text_search_finds_matching_articles() {
    let pool = setup_db().await;

    let a1 = make_article("https://example.com/1", "Rust Programming", vec![]);
    let a2 = make_article("https://example.com/2", "Cooking Recipes", vec![]);
    db::insert_article(&pool, &a1).await.unwrap();
    db::insert_article(&pool, &a2).await.unwrap();

    let results = db::search_articles(&pool, "Rust").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Programming");
}

#[tokio::test]
async fn full_text_search_matches_partial_prefix() {
    let pool = setup_db().await;

    let a1 = make_article("https://example.com/1", "Rust Programming", vec![]);
    let a2 = make_article("https://example.com/2", "Cooking Recipes", vec![]);
    db::insert_article(&pool, &a1).await.unwrap();
    db::insert_article(&pool, &a2).await.unwrap();

    let results = db::search_articles(&pool, "Prog").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Programming");

    let results = db::search_articles(&pool, "Coo").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Cooking Recipes");
}

#[tokio::test]
async fn full_text_search_implicit_and_for_multiple_words() {
    let pool = setup_db().await;

    let a1 = make_article("https://example.com/1", "Rust Programming", vec![]);
    let a2 = make_article("https://example.com/2", "Rust Cooking", vec![]);
    db::insert_article(&pool, &a1).await.unwrap();
    db::insert_article(&pool, &a2).await.unwrap();

    // Both words must appear
    let results = db::search_articles(&pool, "Rust Programming")
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Programming");
}

#[tokio::test]
async fn full_text_search_explicit_or() {
    let pool = setup_db().await;

    let a1 = make_article("https://example.com/1", "Rust Programming", vec![]);
    let a2 = make_article("https://example.com/2", "Cooking Recipes", vec![]);
    let a3 = make_article("https://example.com/3", "Go Programming", vec![]);
    db::insert_article(&pool, &a1).await.unwrap();
    db::insert_article(&pool, &a2).await.unwrap();
    db::insert_article(&pool, &a3).await.unwrap();

    let results = db::search_articles(&pool, "Rust OR Cooking").await.unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn full_text_search_exclude_word() {
    let pool = setup_db().await;

    let a1 = make_article("https://example.com/1", "Rust Programming", vec![]);
    let a2 = make_article("https://example.com/2", "Rust Cooking", vec![]);
    db::insert_article(&pool, &a1).await.unwrap();
    db::insert_article(&pool, &a2).await.unwrap();

    // Rust articles excluding those with "Cooking"
    let results = db::search_articles(&pool, "Rust -Cooking").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Programming");
}

#[tokio::test]
async fn full_text_search_rejects_fts5_injection() {
    let pool = setup_db().await;

    let a1 = make_article("https://example.com/1", "Rust Programming", vec![]);
    db::insert_article(&pool, &a1).await.unwrap();

    // Attempts to inject FTS5 column filter syntax — should not error, just return no results
    // (special chars stripped, leaving empty or harmless token)
    let result = db::search_articles(&pool, "title:Rust").await;
    assert!(result.is_ok());

    let result = db::search_articles(&pool, "NEAR(Rust Programming)").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_article_cascades_to_tags() {
    let pool = setup_db().await;

    let article = make_article(
        "https://example.com/1",
        "With Tags",
        vec!["a".into(), "b".into()],
    );
    let id = article.id;

    db::insert_article(&pool, &article).await.unwrap();
    db::delete_article(&pool, id).await.unwrap();

    // Tags should be gone too (CASCADE)
    let tag_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE article_id = ?")
        .bind(id.to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tag_count, 0);
}
