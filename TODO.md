# TODO

## Done

- [x] Infer tag suggestions from HTML meta tags (og:tags, article:tag, keywords)
- [x] Suggest tags from user's existing tag vocabulary based on article content
- [x] Pre-fill suggested tags in the clip form
- [x] Add tests for article extraction and database operations
- [x] Browser extension (Manifest V3) — right-click context menu to save pages
- [x] CORS support on API routes for extension access
- [x] Docker container with multi-stage build
- [x] Docker Compose for local deployment
- [x] GitHub Actions CI/CD pipeline (test + build + push to GHCR)
- [x] Configurable bind address (`CLIPSTASH_HOST`, `CLIPSTASH_PORT`)
- [x] Release workflow: extension zip attached to GitHub Releases on tag push; rolling `latest-dev` pre-release on every `main` push
- [x] Full-text search: live as-you-type, prefix matching, AND/OR/-word operators

## Next Up

- [ ] Add input validation (URL format, tag length limits)
- [ ] Handle duplicate URL submissions gracefully
- [ ] Add pagination to article list

## Security (public VM hosting)

- [x] **Authentication** — web UI protected with username+password session login; API routes require `Authorization: Bearer <CLIPSTASH_API_KEY>`; constant-time comparisons
- [x] **Extension auth** — extension stores and sends API key via options page
- [ ] **HTTPS** — document (or automate) TLS termination; add Caddy/nginx reverse proxy config to `docker-compose.yml`; update extension to support `https://` server URLs

## Future Enhancements

- [ ] Keyword/TF-IDF extraction for tag inference (e.g. rake crate)
- [ ] Tag management UI (edit/remove tags from article view)
- [ ] Offline/fallback content for failed fetches
- [ ] Export articles (Markdown, PDF)
- [ ] Dark mode toggle
- [ ] Reading time estimate per article
- [ ] Bulk operations (delete multiple, tag multiple)
