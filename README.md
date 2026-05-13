# Clipstash

A self-hosted web article clipper built entirely in Rust. Paste a URL, and Clipstash fetches the page, extracts readable content, and saves it for later reading — like Pocket or Instapaper, but simple and self-contained.

## Features

- **Clip articles** — paste a URL and Clipstash extracts the title, domain, excerpt, and clean readable content
- **Smart tag suggestions** — auto-suggests tags from HTML meta tags (`keywords`, `article:tag`) and matches against your existing tag vocabulary
- **Full-text search** — search across all saved articles using SQLite FTS5
- **Tag filtering** — click any tag to filter your reading list
- **Clean reader view** — distraction-free article reading with responsive typography
- **JSON API** — full CRUD API at `/api/articles` for programmatic access
- **Browser extension** — right-click any page to save it (Chrome + Firefox, Manifest V3)

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Axum |
| Frontend | Askama (server-rendered HTML) |
| Database | SQLite via sqlx |
| Content extraction | readability + scraper |
| HTTP client | reqwest |
| Search | SQLite FTS5 |

## Getting Started

### Prerequisites

- Rust 1.85+ (2024 edition)
- SQLite 3

### Run (local)

```sh
cargo run -p clipstash-server
```

The server starts at [http://localhost:3000](http://localhost:3000). A SQLite database (`clipstash.db`) is created automatically.

### Run (Docker)

```sh
docker compose up -d
```

Or build and run manually:

```sh
docker build -t clipstash .
docker run -p 3000:3000 -v clipstash-data:/home/clipstash/data clipstash
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:clipstash.db?mode=rwc` | SQLite connection string |
| `CLIPSTASH_HOST` | `127.0.0.1` | Bind address (`0.0.0.0` for containers) |
| `CLIPSTASH_PORT` | `3000` | Listen port |

### Test

```sh
cargo test --workspace
```

## Usage

1. Open [http://localhost:3000](http://localhost:3000)
2. Paste a URL and click **Clip**
3. Review the extracted article and suggested tags — edit tags if needed
4. Click **Save Article** to add it to your library
5. Use the search bar or click tags to find saved articles

## API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/articles` | List articles (`?q=` for search, `?tag=` for filter) |
| POST | `/api/articles` | Create article (`{"url": "...", "tags": [...]}`) |
| GET | `/api/articles/{id}` | Get article |
| PUT | `/api/articles/{id}/tags` | Update tags (`{"tags": [...]}`) |
| DELETE | `/api/articles/{id}` | Delete article |

## Browser Extension

The `extension/` directory contains a Manifest V3 browser extension for Chrome and Firefox.

### Install (Firefox)

1. Open `about:debugging#/runtime/this-firefox`
2. Click **Load Temporary Add-on** and select `extension/manifest.json`

### Install (Chrome)

1. Open `chrome://extensions`
2. Enable **Developer mode**
3. Click **Load unpacked** and select the `extension/` directory

Right-click any page and choose **Save to Clipstash**. Configure the server URL in the extension options (defaults to `http://localhost:3000`).

Pre-built extension zips are available on the [Releases](../../releases) page — download and load as above.

## Project Structure

```
clipstash/
├── crates/
│   ├── server/       # Axum backend (API + HTML pages + extractor + tag suggestions)
│   ├── frontend/     # Askama templates
│   └── shared/       # Shared types, models, errors
├── extension/        # Browser extension (Manifest V3)
├── migrations/       # SQLite migrations
├── static/           # CSS
├── .github/workflows # CI/CD pipeline
├── Dockerfile        # Multi-stage container build
└── docker-compose.yml
```

## License

GPL 3.0
