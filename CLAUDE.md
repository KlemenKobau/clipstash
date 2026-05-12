# Clipstash — Full-Stack Rust Web Clipper

## Project Overview

A web-based article clipper/reader (like Pocket/Instapaper) built entirely in Rust.
Paste a URL → the app fetches the page, extracts readable content, and saves it.
Organize with tags and search across all saved articles.

## Tech Stack

- **Frontend:** Askama templates (server-rendered HTML)
- **Backend:** Axum (HTML pages + JSON API)
- **Database:** SQLite via `sqlx`
- **HTML Parsing:** `readability` + `scraper` crates for content extraction
- **HTTP Client:** `reqwest` for fetching URLs
- **Full-text Search:** SQLite FTS5
- **Workspace:** Cargo workspace with `server`, `frontend`, `shared` crates

## Architecture

```
clipstash/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── server/             # Axum backend (API + HTML pages)
│   ├── frontend/           # Askama templates
│   └── shared/             # Shared types, models, errors
├── migrations/             # SQLite migrations
├── static/                 # Static assets (CSS)
└── CLAUDE.md
```

## Conventions

- Use `thiserror` for error types, `anyhow` only at binary boundaries
- Prefer `sqlx` for database access
- Keep templates small and focused
- Name database models with a `Db` prefix (e.g., `DbArticle`), API types without
- HTML pages served at `/`, `/article/{id}`; JSON API at `/api/articles`

## MVP Features

1. Paste a URL → fetch & parse readable article content
2. Save articles with metadata (title, domain, date, excerpt)
3. Tag articles
4. Full-text search across saved content
5. Clean, minimal reader UI

## Development

```sh
# Run server (http://localhost:3000)
cargo run -p clipstash-server

# Run tests
cargo test --workspace
```

## General

1. Use AskUserQuestions, nothing should be ambiguous
2. Make frequent commits and have meaningful descriptions. Use Conventional Commits
3. Use only trusted crates
4. Add small features and add tests for these features
   1. Testing policy: high value tests only
5. Use KISS, YAGNI and SOLID principles”
6. Track a todo in TODO.md
7. Keep CLAUDE.md up to date, but keep it short and compact