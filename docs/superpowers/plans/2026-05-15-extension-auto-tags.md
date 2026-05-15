# Extension Auto-Tag Inference Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** When the extension saves a URL, the server automatically infers tags from the page using meta tags and existing vocabulary — no tags sent by the client.

**Architecture:** Remove `tags` from `CreateArticleRequest`. In `create_article`, replace the one-shot `fetch_and_extract` call with `fetch_and_parse` → `get_all_existing_tags` → `suggest_tags` → `into_article`. The extension drops `tags: []` from its request body.

**Tech Stack:** Rust/Axum, sqlx/SQLite, existing `suggest_tags` and `extractor` modules, JavaScript (Manifest V3 extension).

---

### Task 1: Remove `tags` from `CreateArticleRequest`

**Files:**
- Modify: `crates/shared/src/models.rs`

- [ ] **Step 1: Remove the `tags` field**

In `crates/shared/src/models.rs`, change `CreateArticleRequest` from:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArticleRequest {
    pub url: String,
    pub tags: Vec<String>,
}
```

to:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArticleRequest {
    pub url: String,
}
```

- [ ] **Step 2: Verify it compiles (errors expected in routes.rs)**

```bash
cargo check --workspace 2>&1 | head -40
```

Expected: compile errors in `crates/server/src/routes.rs` referencing `req.tags`. That's fine — Task 2 fixes them.

---

### Task 2: Wire auto-tag inference into `create_article`

**Files:**
- Modify: `crates/server/src/routes.rs`

- [ ] **Step 1: Rewrite `create_article`**

Replace the current body of `create_article` in `crates/server/src/routes.rs`:

```rust
pub async fn create_article(
    State(state): State<AppState>,
    Json(req): Json<CreateArticleRequest>,
) -> impl IntoResponse {
    match extractor::fetch_and_extract(&req.url, req.tags).await {
        Ok(article) => match db::insert_article(&state.pool, &article).await {
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
```

with:

```rust
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
            )
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
```

- [ ] **Step 2: Verify it compiles cleanly**

```bash
cargo check --workspace 2>&1
```

Expected: no errors.

- [ ] **Step 3: Run existing tests**

```bash
cargo test --workspace 2>&1
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
cargo fmt --all
git add crates/shared/src/models.rs crates/server/src/routes.rs
git commit -m "feat: auto-infer tags from page on extension save

Remove tags from CreateArticleRequest; create_article now calls
fetch_and_parse + suggest_tags + into_article instead of fetch_and_extract."
```

---

### Task 3: Drop `tags` from extension request

**Files:**
- Modify: `extension/background.js`

- [ ] **Step 1: Remove `tags: []` from the fetch body**

In `extension/background.js`, change:

```js
body: JSON.stringify({ url, tags: [] }),
```

to:

```js
body: JSON.stringify({ url }),
```

- [ ] **Step 2: Commit**

```bash
git add extension/background.js
git commit -m "feat(extension): remove tags from save request — server now infers them"
```

---

### Task 4: Add integration test for tag inference in `create_article`

**Files:**
- Modify: `crates/server/src/routes.rs` (add a `#[cfg(test)]` block at the bottom)

This test verifies the wiring: given HTML with known meta keywords, `suggest_tags` returns them, and the article is saved with those tags. We test `suggest_tags` directly (not through HTTP) since the route's HTTP fetch can't easily be mocked — the existing `suggest_tags` unit tests already cover extraction logic. What we add here verifies the fallback: if `get_all_existing_tags` fails (empty pool / error), inference still runs and returns meta-tag results.

- [ ] **Step 1: Write the test**

Add at the bottom of `crates/server/src/routes.rs`:

```rust
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

        assert!(tags.contains(&"rust".to_string()), "expected 'rust' in {tags:?}");
        assert!(tags.contains(&"async".to_string()), "expected 'async' in {tags:?}");
        assert!(tags.contains(&"web".to_string()), "expected 'web' in {tags:?}");
    }

    #[tokio::test]
    async fn suggest_tags_merges_vocabulary_matches() {
        let html = r#"<html><head>
            <meta name="keywords" content="rust">
        </head><body><p>A deep dive into async programming with Tokio.</p></body></html>"#;
        let text = "A deep dive into async programming with Tokio.";
        let existing = vec!["tokio".to_string(), "python".to_string()];

        let tags = suggest_tags::suggest_tags(html, text, &existing).await;

        assert!(tags.contains(&"rust".to_string()), "expected meta tag 'rust'");
        assert!(tags.contains(&"tokio".to_string()), "expected vocabulary match 'tokio'");
        assert!(!tags.contains(&"python".to_string()), "python should not match");
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test -p clipstash-server suggest_tags 2>&1
```

Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
cargo fmt --all
cargo clippy --workspace -- -A warnings
git add crates/server/src/routes.rs
git commit -m "test(server): verify suggest_tags wiring for extension save path"
```
