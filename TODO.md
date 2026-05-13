# TODO

## Done

- [x] Infer tag suggestions from HTML meta tags (og:tags, article:tag, keywords)
- [x] Suggest tags from user's existing tag vocabulary based on article content
- [x] Pre-fill suggested tags in the clip form

## Next Up

- [x] Add tests for article extraction and database operations
- [ ] Add input validation (URL format, tag length limits)
- [ ] Handle duplicate URL submissions gracefully
- [ ] Add pagination to article list

## Done

- [x] Browser extension (Manifest V3) — right-click context menu to save pages
- [x] CORS support on API routes for extension access

## Done

- [x] Docker container with multi-stage build
- [x] Docker Compose for local deployment
- [x] GitHub Actions CI/CD pipeline (test + build + push to GHCR)
- [x] Configurable bind address (`CLIPSTASH_HOST`, `CLIPSTASH_PORT`)

## Future Enhancements

- [ ] Keyword/TF-IDF extraction for tag inference (e.g. rake crate)
- [ ] Tag management UI (edit/remove tags from article view)
- [ ] Offline/fallback content for failed fetches
- [ ] Export articles (Markdown, PDF)
- [ ] Dark mode toggle
- [ ] Reading time estimate per article
- [ ] Bulk operations (delete multiple, tag multiple)
