CREATE TABLE IF NOT EXISTS articles (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    domain TEXT NOT NULL,
    excerpt TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    article_id TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    UNIQUE(article_id, name)
);

CREATE INDEX IF NOT EXISTS idx_tags_article_id ON tags(article_id);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);

-- Full-text search index
CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
    title,
    excerpt,
    content,
    content='articles',
    content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
    INSERT INTO articles_fts(rowid, title, excerpt, content)
    VALUES (new.rowid, new.title, new.excerpt, new.content);
END;

CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, excerpt, content)
    VALUES ('delete', old.rowid, old.title, old.excerpt, old.content);
END;

CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, excerpt, content)
    VALUES ('delete', old.rowid, old.title, old.excerpt, old.content);
    INSERT INTO articles_fts(rowid, title, excerpt, content)
    VALUES (new.rowid, new.title, new.excerpt, new.content);
END;
