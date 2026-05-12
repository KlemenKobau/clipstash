use clipstash_shared::error::ClipstashError;
use clipstash_shared::models::Article;
use chrono::Utc;
use uuid::Uuid;

pub async fn fetch_and_extract(url: &str, tags: Vec<String>) -> Result<Article, ClipstashError> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| ClipstashError::FetchError(e.to_string()))?;

    let html = response
        .text()
        .await
        .map_err(|e| ClipstashError::FetchError(e.to_string()))?;

    let parsed_url = url::Url::parse(url)
        .map_err(|e| ClipstashError::InvalidInput(e.to_string()))?;

    let mut html_bytes = html.as_bytes();
    let extracted = readability::extractor::extract(&mut html_bytes, &parsed_url)
        .map_err(|e| ClipstashError::ParseError(e.to_string()))?;

    let domain = parsed_url
        .host_str()
        .unwrap_or("unknown")
        .to_string();

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

    let now = Utc::now();

    Ok(Article {
        id: Uuid::new_v4(),
        url: url.to_string(),
        title,
        domain,
        excerpt,
        content: extracted.content,
        tags,
        created_at: now,
        updated_at: now,
    })
}

fn extract_title_from_html(html: &str) -> Option<String> {
    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse("title").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|t| !t.is_empty())
}
