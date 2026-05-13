use chrono::{DateTime, Utc};
use clipstash_shared::error::ClipstashError;
use clipstash_shared::models::{Article, ArticleSummary};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), ClipstashError> {
    let migration_sql = include_str!("../../../migrations/001_initial.sql");
    sqlx::raw_sql(migration_sql)
        .execute(pool)
        .await
        .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub async fn insert_article(pool: &SqlitePool, article: &Article) -> Result<(), ClipstashError> {
    let id = article.id.to_string();
    let created_at = article.created_at.to_rfc3339();
    let updated_at = article.updated_at.to_rfc3339();

    sqlx::query(
        "INSERT INTO articles (id, url, title, domain, excerpt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&article.url)
    .bind(&article.title)
    .bind(&article.domain)
    .bind(&article.excerpt)
    .bind(&article.content)
    .bind(&created_at)
    .bind(&updated_at)
    .execute(pool)
    .await
    .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;

    for tag in &article.tags {
        sqlx::query("INSERT OR IGNORE INTO tags (article_id, name) VALUES (?, ?)")
            .bind(&id)
            .bind(tag)
            .execute(pool)
            .await
            .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

pub async fn get_article(pool: &SqlitePool, id: Uuid) -> Result<Article, ClipstashError> {
    let id_str = id.to_string();

    let row = sqlx::query_as::<_, DbArticle>(
        "SELECT id, url, title, domain, excerpt, content, created_at, updated_at
         FROM articles WHERE id = ?",
    )
    .bind(&id_str)
    .fetch_optional(pool)
    .await
    .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?
    .ok_or(ClipstashError::NotFound)?;

    let tags = get_tags(pool, &id_str).await?;
    row.into_article(tags)
}

pub async fn list_articles(pool: &SqlitePool) -> Result<Vec<ArticleSummary>, ClipstashError> {
    let rows = sqlx::query_as::<_, DbArticleSummary>(
        "SELECT id, url, title, domain, excerpt, created_at
         FROM articles ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;

    let mut articles = Vec::with_capacity(rows.len());
    for row in rows {
        let tags = get_tags(pool, &row.id).await?;
        articles.push(row.into_summary(tags)?);
    }
    Ok(articles)
}

/// Tokens produced by parsing the user query.
#[derive(Debug, PartialEq)]
enum QueryToken {
    Include(String), // plain word  → `word*`
    Exclude(String), // -word       → `NOT word*`
    And,
    Or,
}

/// Translate a user-facing search query into a safe FTS5 MATCH expression.
///
/// Supported syntax:
///   - Plain words  → prefix match (`rust` → `rust*`)
///   - `-word`      → exclude (`-rust` → `NOT rust*`)
///   - `AND` / `OR` → passed through as FTS5 boolean operators
///
/// Multiple plain words are implicitly AND-ed. Everything else (column
/// filters, NEAR, parentheses, quotes, etc.) is stripped so users cannot
/// inject arbitrary FTS5 syntax. The resulting string is passed as a bound
/// parameter and is never interpolated into the SQL statement itself.
fn build_fts5_query(query: &str) -> String {
    let mut tokens: Vec<QueryToken> = Vec::new();

    for raw in query.split_whitespace() {
        match raw {
            "AND" => tokens.push(QueryToken::And),
            "OR" => tokens.push(QueryToken::Or),
            t if t.starts_with('-') => {
                let word = sanitize_fts5_term(&t[1..]);
                if !word.is_empty() {
                    tokens.push(QueryToken::Exclude(word));
                }
            }
            t => {
                let word = sanitize_fts5_term(t);
                if !word.is_empty() {
                    tokens.push(QueryToken::Include(word));
                }
            }
        }
    }

    // Drop leading/trailing AND/OR
    while matches!(tokens.first(), Some(QueryToken::And) | Some(QueryToken::Or)) {
        tokens.remove(0);
    }
    while matches!(tokens.last(), Some(QueryToken::And) | Some(QueryToken::Or)) {
        tokens.pop();
    }

    if tokens.is_empty() {
        return String::new();
    }

    // Build the FTS5 expression.
    // FTS5 binary operators: `a AND b`, `a OR b`, `a NOT b`
    // Exclusions must be binary: the preceding term NOT excluded_term.
    // Implicit AND is inserted between adjacent include terms.
    let mut out = String::new();
    let mut need_op = false; // whether the next term needs a preceding operator

    for token in &tokens {
        match token {
            QueryToken::And => {
                out.push_str(" AND");
                need_op = false;
            }
            QueryToken::Or => {
                out.push_str(" OR");
                need_op = false;
            }
            QueryToken::Include(word) => {
                if need_op {
                    out.push_str(" AND");
                }
                out.push_str(&format!(" {word}*"));
                need_op = true;
            }
            QueryToken::Exclude(word) => {
                // `NOT` is binary in FTS5: requires a left-hand term.
                // If there's nothing to the left yet, skip the exclusion.
                if need_op {
                    out.push_str(&format!(" NOT {word}*"));
                    // need_op stays true; next term still needs an operator
                }
            }
        }
    }

    out.trim().to_string()
}

/// Strip characters that have special meaning in FTS5 term syntax.
/// Keeps only alphanumeric characters and underscores.
fn sanitize_fts5_term(term: &str) -> String {
    term.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

pub async fn search_articles(
    pool: &SqlitePool,
    query: &str,
) -> Result<Vec<ArticleSummary>, ClipstashError> {
    let fts_query = build_fts5_query(query);
    if fts_query.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query_as::<_, DbArticleSummary>(
        "SELECT a.id, a.url, a.title, a.domain, a.excerpt, a.created_at
         FROM articles a
         INNER JOIN articles_fts fts ON a.rowid = fts.rowid
         WHERE articles_fts MATCH ?
         ORDER BY rank",
    )
    .bind(fts_query)
    .fetch_all(pool)
    .await
    .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;

    let mut articles = Vec::with_capacity(rows.len());
    for row in rows {
        let tags = get_tags(pool, &row.id).await?;
        articles.push(row.into_summary(tags)?);
    }
    Ok(articles)
}

pub async fn search_by_tag(
    pool: &SqlitePool,
    tag: &str,
) -> Result<Vec<ArticleSummary>, ClipstashError> {
    let rows = sqlx::query_as::<_, DbArticleSummary>(
        "SELECT a.id, a.url, a.title, a.domain, a.excerpt, a.created_at
         FROM articles a
         INNER JOIN tags t ON a.id = t.article_id
         WHERE t.name = ?
         ORDER BY a.created_at DESC",
    )
    .bind(tag)
    .fetch_all(pool)
    .await
    .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;

    let mut articles = Vec::with_capacity(rows.len());
    for row in rows {
        let tags = get_tags(pool, &row.id).await?;
        articles.push(row.into_summary(tags)?);
    }
    Ok(articles)
}

pub async fn update_tags(
    pool: &SqlitePool,
    id: Uuid,
    tags: &[String],
) -> Result<(), ClipstashError> {
    let id_str = id.to_string();

    // Verify article exists
    let exists = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM articles WHERE id = ?")
        .bind(&id_str)
        .fetch_one(pool)
        .await
        .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;

    if exists == 0 {
        return Err(ClipstashError::NotFound);
    }

    sqlx::query("DELETE FROM tags WHERE article_id = ?")
        .bind(&id_str)
        .execute(pool)
        .await
        .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;

    for tag in tags {
        sqlx::query("INSERT INTO tags (article_id, name) VALUES (?, ?)")
            .bind(&id_str)
            .bind(tag)
            .execute(pool)
            .await
            .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

pub async fn delete_article(pool: &SqlitePool, id: Uuid) -> Result<(), ClipstashError> {
    let id_str = id.to_string();
    let result = sqlx::query("DELETE FROM articles WHERE id = ?")
        .bind(&id_str)
        .execute(pool)
        .await
        .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ClipstashError::NotFound);
    }
    Ok(())
}

async fn get_tags(pool: &SqlitePool, article_id: &str) -> Result<Vec<String>, ClipstashError> {
    let tags =
        sqlx::query_scalar::<_, String>("SELECT name FROM tags WHERE article_id = ? ORDER BY name")
            .bind(article_id)
            .fetch_all(pool)
            .await
            .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?;
    Ok(tags)
}

#[derive(sqlx::FromRow)]
struct DbArticle {
    id: String,
    url: String,
    title: String,
    domain: String,
    excerpt: String,
    content: String,
    created_at: String,
    updated_at: String,
}

impl DbArticle {
    fn into_article(self, tags: Vec<String>) -> Result<Article, ClipstashError> {
        Ok(Article {
            id: Uuid::parse_str(&self.id)
                .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?,
            url: self.url,
            title: self.title,
            domain: self.domain,
            excerpt: self.excerpt,
            content: self.content,
            tags,
            created_at: parse_datetime(&self.created_at)?,
            updated_at: parse_datetime(&self.updated_at)?,
        })
    }
}

#[derive(sqlx::FromRow)]
struct DbArticleSummary {
    id: String,
    url: String,
    title: String,
    domain: String,
    excerpt: String,
    created_at: String,
}

impl DbArticleSummary {
    fn into_summary(self, tags: Vec<String>) -> Result<ArticleSummary, ClipstashError> {
        Ok(ArticleSummary {
            id: Uuid::parse_str(&self.id)
                .map_err(|e| ClipstashError::DatabaseError(e.to_string()))?,
            url: self.url,
            title: self.title,
            domain: self.domain,
            excerpt: self.excerpt,
            tags,
            created_at: parse_datetime(&self.created_at)?,
        })
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ClipstashError> {
    // Try RFC3339 first, then SQLite default format
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|ndt| ndt.and_utc())
        })
        .map_err(|e| ClipstashError::DatabaseError(format!("Invalid datetime '{s}': {e}")))
}
