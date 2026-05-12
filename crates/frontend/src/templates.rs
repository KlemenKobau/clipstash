use askama::Template;
use clipstash_shared::models::{Article, ArticleSummary};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub articles: Vec<ArticleSummary>,
    pub search: String,
    pub tag_filter: String,
    pub error: Option<String>,
    pub success: Option<String>,
}

#[derive(Template)]
#[template(path = "article.html")]
pub struct ArticleTemplate {
    pub article: Article,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub message: String,
}
