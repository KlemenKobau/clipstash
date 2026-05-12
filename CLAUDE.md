# Clipstash — Full-Stack Rust Web Clipper

## Project Overview

A web-based article clipper/reader (like Pocket/Instapaper) built entirely in Rust.
Paste a URL → the app fetches the page, extracts readable content, and saves it.
Organize with tags and search across all saved articles.

## Tech Stack

- **Frontend:** Leptos (SSR + client-side hydration)
- **Backend:** Axum (HTTP server, API routes)
- **Database:** SQLite via `rusqlite` or `sqlx`
- **HTML Parsing:** `readability` / `scraper` crates for content extraction
- **HTTP Client:** `reqwest` for fetching URLs
- **Full-text Search:** SQLite FTS5
- **Workspace:** Cargo workspace with separate crates for frontend, backend, and shared types

## Architecture

```
clipstash/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── server/             # Axum backend + Leptos SSR
│   ├── frontend/           # Leptos components & pages
│   └── shared/             # Shared types, models, validation
├── migrations/             # SQLite migrations
├── static/                 # Static assets (CSS, icons)
└── CLAUDE.md
```

## Conventions

- Use `thiserror` for error types, `anyhow` only at binary boundaries
- Prefer `sqlx` with compile-time checked queries when possible
- Keep Leptos components small and focused
- Use server functions (`#[server]`) for client→server communication
- Name database models with a `Db` prefix (e.g., `DbArticle`), API types without

## MVP Features

1. Paste a URL → fetch & parse readable article content
2. Save articles with metadata (title, domain, date, excerpt)
3. Tag articles
4. Full-text search across saved content
5. Clean, minimal reader UI

## Development

```sh
# Install trunk + cargo-leptos if needed
cargo install cargo-leptos

# Run dev server
cargo leptos watch

# Run tests
cargo test --workspace
```
