---
title: Extension Auto-Tag Inference
date: 2026-05-15
---

# Extension Auto-Tag Inference — Design

## Goal

When the browser extension saves a URL via `POST /api/articles`, the server automatically infers tags from the fetched page using both HTML meta tags and the user's existing tag vocabulary. The extension sends no tags.

## Current State

- `suggest_tags::suggest_tags(html, text, existing_tags)` already exists and combines meta-tag extraction with vocabulary matching.
- `extractor::fetch_and_parse(url)` returns `ExtractedContent` (includes `raw_html` and `text`).
- `extractor::into_article(content, tags)` builds the final `Article`.
- `routes::create_article` currently calls the one-shot `fetch_and_extract(url, tags)` passing tags from the request.
- `CreateArticleRequest` has a `tags: Vec<String>` field.
- The extension sends `{ url, tags: [] }`.

## Design

### Data model

Remove `tags` from `CreateArticleRequest`. The body sent to `POST /api/articles` becomes `{ "url": "..." }` only.

### Route (`routes::create_article`)

Replace the single `fetch_and_extract` call with three steps:

1. `extractor::fetch_and_parse(&req.url)` → `ExtractedContent`
2. `suggest_tags::get_all_existing_tags(&state.pool)` → `Vec<String>` (fall back to `[]` on DB error)
3. `suggest_tags::suggest_tags(&content.raw_html, &content.text, &existing_tags)` → `Vec<String>`
4. `extractor::into_article(content, tags)` → `Article`
5. Insert and return as before.

### Extension (`extension/background.js`)

Remove `tags: []` from the JSON body.

### Error handling

If `get_all_existing_tags` fails, log a warning and use an empty vocabulary. Tag inference continues using meta tags only. A DB error must not block the save.

### Testing

Add a unit test in `routes.rs` (or a separate integration test file) verifying that when `create_article` is called with a URL whose fetched HTML contains known meta keywords, the saved article has those tags. Since the route does HTTP fetching, mock at the extractor level or use an integration test with a local test server.

> Simpler approach: test `suggest_tags::suggest_tags` in isolation (already done). For the route, a lightweight integration test calling `create_article` with a real in-memory SQLite pool and a stub HTTP response is sufficient.
