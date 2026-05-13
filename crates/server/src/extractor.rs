use chrono::Utc;
use clipstash_shared::error::ClipstashError;
use clipstash_shared::models::Article;
use uuid::Uuid;

/// Intermediate result from fetching and parsing a URL, before tags are finalized.
pub struct ExtractedContent {
    pub url: String,
    pub title: String,
    pub domain: String,
    pub excerpt: String,
    pub content_html: String,
    pub text: String,
    pub raw_html: String,
}

/// Fetch a URL and extract its readable content, returning intermediate data.
pub async fn fetch_and_parse(url: &str) -> Result<ExtractedContent, ClipstashError> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| ClipstashError::FetchError(e.to_string()))?;

    let html = response
        .text()
        .await
        .map_err(|e| ClipstashError::FetchError(e.to_string()))?;

    let parsed_url =
        url::Url::parse(url).map_err(|e| ClipstashError::InvalidInput(e.to_string()))?;

    let mut html_bytes = html.as_bytes();
    let extracted = readability::extractor::extract(&mut html_bytes, &parsed_url)
        .map_err(|e| ClipstashError::ParseError(e.to_string()))?;

    let domain = parsed_url.host_str().unwrap_or("unknown").to_string();

    let title = if extracted.title.is_empty() {
        extract_title_from_html(&html).unwrap_or_else(|| domain.clone())
    } else {
        extracted.title
    };

    let excerpt = extracted
        .text
        .chars()
        .take(300)
        .collect::<String>()
        .trim()
        .to_string();

    Ok(ExtractedContent {
        url: url.to_string(),
        title,
        domain,
        excerpt,
        content_html: extracted.content,
        text: extracted.text,
        raw_html: html,
    })
}

/// Build a final Article from extracted content and chosen tags.
pub fn into_article(content: ExtractedContent, tags: Vec<String>) -> Article {
    let now = Utc::now();
    Article {
        id: Uuid::new_v4(),
        url: content.url,
        title: content.title,
        domain: content.domain,
        excerpt: content.excerpt,
        content: content.content_html,
        tags,
        created_at: now,
        updated_at: now,
    }
}

/// Convenience: fetch, extract, and build article in one step.
pub async fn fetch_and_extract(url: &str, tags: Vec<String>) -> Result<Article, ClipstashError> {
    let content = fetch_and_parse(url).await?;
    Ok(into_article(content, tags))
}

pub(crate) fn extract_title_from_html(html: &str) -> Option<String> {
    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse("title").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|t| !t.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_title_from_valid_html() {
        let html = "<html><head><title>My Article</title></head><body></body></html>";
        assert_eq!(
            extract_title_from_html(html),
            Some("My Article".to_string())
        );
    }

    #[test]
    fn returns_none_when_no_title_tag() {
        let html = "<html><head></head><body><p>Hello</p></body></html>";
        assert_eq!(extract_title_from_html(html), None);
    }

    #[test]
    fn returns_none_for_empty_title() {
        let html = "<html><head><title>   </title></head><body></body></html>";
        assert_eq!(extract_title_from_html(html), None);
    }

    #[test]
    fn trims_whitespace_from_title() {
        let html = "<html><head><title>  Padded Title  </title></head><body></body></html>";
        assert_eq!(
            extract_title_from_html(html),
            Some("Padded Title".to_string())
        );
    }
}
