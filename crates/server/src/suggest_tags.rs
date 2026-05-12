use clipstash_shared::error::ClipstashError;

/// Extract tag candidates from HTML meta tags.
///
/// Looks for: <meta name="keywords">, <meta property="article:tag">,
/// <meta property="og:article:tag">, and <meta name="news_keywords">.
pub fn extract_meta_tags(html: &str) -> Vec<String> {
    let document = scraper::Html::parse_document(html);
    let mut tags = Vec::new();

    // meta name="keywords" — comma-separated list
    if let Some(sel) = scraper::Selector::parse(r#"meta[name="keywords"]"#).ok() {
        for el in document.select(&sel) {
            if let Some(content) = el.value().attr("content") {
                tags.extend(split_and_clean(content, ','));
            }
        }
    }

    // meta name="news_keywords" — comma-separated (AP, Reuters style)
    if let Some(sel) = scraper::Selector::parse(r#"meta[name="news_keywords"]"#).ok() {
        for el in document.select(&sel) {
            if let Some(content) = el.value().attr("content") {
                tags.extend(split_and_clean(content, ','));
            }
        }
    }

    // meta property="article:tag" — one per tag (Open Graph)
    if let Some(sel) = scraper::Selector::parse(r#"meta[property="article:tag"]"#).ok() {
        for el in document.select(&sel) {
            if let Some(content) = el.value().attr("content") {
                let trimmed = content.trim().to_lowercase();
                if !trimmed.is_empty() {
                    tags.push(trimmed);
                }
            }
        }
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    tags.retain(|t| seen.insert(t.clone()));

    tags
}

/// Match article text against existing user tags.
///
/// Returns tags from the vocabulary that appear as whole words in the text.
pub fn match_existing_tags(text: &str, existing_tags: &[String]) -> Vec<String> {
    let text_lower = text.to_lowercase();
    existing_tags
        .iter()
        .filter(|tag| {
            let tag_lower = tag.to_lowercase();
            // Check for whole-word match: the tag must appear surrounded by
            // non-alphanumeric chars (or at string boundaries)
            text_lower
                .match_indices(&tag_lower)
                .any(|(pos, matched)| {
                    let before_ok = pos == 0
                        || !text_lower.as_bytes()[pos - 1].is_ascii_alphanumeric();
                    let after_pos = pos + matched.len();
                    let after_ok = after_pos >= text_lower.len()
                        || !text_lower.as_bytes()[after_pos].is_ascii_alphanumeric();
                    before_ok && after_ok
                })
        })
        .cloned()
        .collect()
}

/// Combine meta-tag extraction and vocabulary matching, deduplicating the results.
pub async fn suggest_tags(
    html: &str,
    article_text: &str,
    existing_tags: &[String],
) -> Vec<String> {
    let mut suggestions = extract_meta_tags(html);
    let matched = match_existing_tags(article_text, existing_tags);

    let mut seen: std::collections::HashSet<String> =
        suggestions.iter().cloned().collect();
    for tag in matched {
        if seen.insert(tag.clone()) {
            suggestions.push(tag);
        }
    }

    suggestions
}

/// Fetch all distinct tag names the user has ever used.
pub async fn get_all_existing_tags(
    pool: &sqlx::sqlite::SqlitePool,
) -> Result<Vec<String>, ClipstashError> {
    let tags = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT name FROM tags ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;
    Ok(tags)
}

fn split_and_clean(s: &str, sep: char) -> Vec<String> {
    s.split(sep)
        .map(|part| part.trim().to_lowercase())
        .filter(|part| !part.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_keywords_meta_tag() {
        let html = r#"<html><head>
            <meta name="keywords" content="Rust, WebAssembly, Programming">
        </head><body></body></html>"#;
        let tags = extract_meta_tags(html);
        assert_eq!(tags, vec!["rust", "webassembly", "programming"]);
    }

    #[test]
    fn extracts_article_tag_meta() {
        let html = r#"<html><head>
            <meta property="article:tag" content="Machine Learning">
            <meta property="article:tag" content="AI">
        </head><body></body></html>"#;
        let tags = extract_meta_tags(html);
        assert_eq!(tags, vec!["machine learning", "ai"]);
    }

    #[test]
    fn deduplicates_across_sources() {
        let html = r#"<html><head>
            <meta name="keywords" content="rust, programming">
            <meta property="article:tag" content="Rust">
        </head><body></body></html>"#;
        let tags = extract_meta_tags(html);
        assert_eq!(tags, vec!["rust", "programming"]);
    }

    #[test]
    fn returns_empty_for_no_meta_tags() {
        let html = "<html><head></head><body></body></html>";
        let tags = extract_meta_tags(html);
        assert!(tags.is_empty());
    }

    #[test]
    fn matches_existing_tags_whole_word() {
        let existing = vec!["rust".into(), "go".into(), "python".into()];
        let text = "This article is about Rust and Python programming";
        let matched = match_existing_tags(text, &existing);
        assert_eq!(matched, vec!["rust".to_string(), "python".to_string()]);
    }

    #[test]
    fn does_not_match_partial_words() {
        let existing = vec!["go".into()];
        let text = "This is about Google and Golang features";
        let matched = match_existing_tags(text, &existing);
        assert!(matched.is_empty());
    }

    #[test]
    fn matches_tag_at_string_boundaries() {
        let existing = vec!["rust".into()];
        let text = "rust is great";
        let matched = match_existing_tags(text, &existing);
        assert_eq!(matched, vec!["rust".to_string()]);

        let text2 = "I love rust";
        let matched2 = match_existing_tags(text2, &existing);
        assert_eq!(matched2, vec!["rust".to_string()]);
    }
}
