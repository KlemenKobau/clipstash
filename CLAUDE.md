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
├── extension/              # Browser extension (Manifest V3, Chrome+Firefox)
├── migrations/             # SQLite migrations
├── static/                 # Static assets (CSS)
├── .github/workflows/      # CI/CD (ci.yml + release.yml)
├── Dockerfile              # Multi-stage container build
├── docker-compose.yml      # Local container deployment
└── CLAUDE.md
```

## Conventions

- Use `thiserror` for error types, `anyhow` only at binary boundaries
- Prefer `sqlx` for database access
- Keep templates small and focused
- Name database models with a `Db` prefix (e.g., `DbArticle`), API types without
- HTML pages served at `/`, `/preview-clip`, `/clip`, `/article/{id}`; JSON API at `/api/articles`

## Features

1. Paste a URL → fetch & parse readable article content
2. Save articles with metadata (title, domain, date, excerpt)
3. Tag articles with auto-suggestions (HTML meta tags + existing vocabulary matching)
4. Full-text search across saved content (FTS5)
5. Clean, minimal reader UI
6. Browser extension (Manifest V3) — right-click to save pages (Chrome + Firefox)

## Development

```sh
# Run server (http://localhost:3000)
cargo run -p clipstash-server

# Run tests
cargo test --workspace

# Run with Docker
docker compose up -d
```

## General

1. Use AskUserQuestions, nothing should be ambiguous
2. Make frequent commits and have meaningful descriptions. Use Conventional Commits
3. Use only trusted crates
4. Add small features and add tests for these features
   1. Testing policy: high value tests only
5. Use KISS, YAGNI and SOLID principles
6. Track a todo in TODO.md
7. Keep CLAUDE.md up to date, but keep it short and compact
8. Keep the README.md up to date
9. Check project security with `cargo audit`
10. Always run `cargo fmt --all` before committing
11. Run `cargo clippy --workspace` after implementations to catch warnings and write idiomatic Rust