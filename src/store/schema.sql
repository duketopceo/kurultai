-- Kurultai hot store schema (Phase 1). Idempotent via IF NOT EXISTS.

CREATE TABLE IF NOT EXISTS store_meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS atoms (
    id TEXT PRIMARY KEY NOT NULL,
    source TEXT NOT NULL,
    source_id TEXT NOT NULL,
    source_uri TEXT,
    title TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    question TEXT,
    resolution TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    provenance TEXT,
    source_updated_at TEXT NOT NULL,
    indexed_at TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    has_embedding INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_atoms_source ON atoms(source);
CREATE INDEX IF NOT EXISTS idx_atoms_content_hash ON atoms(content_hash);

CREATE VIRTUAL TABLE IF NOT EXISTS atoms_fts USING fts5(
    title,
    content,
    summary,
    tags,
    content='atoms',
    content_rowid='rowid'
);

CREATE TRIGGER IF NOT EXISTS atoms_ai AFTER INSERT ON atoms BEGIN
    INSERT INTO atoms_fts(rowid, title, content, summary, tags)
    VALUES (new.rowid, new.title, new.content, new.summary, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS atoms_ad AFTER DELETE ON atoms BEGIN
    INSERT INTO atoms_fts(atoms_fts, rowid, title, content, summary, tags)
    VALUES ('delete', old.rowid, old.title, old.content, old.summary, old.tags);
END;

CREATE TRIGGER IF NOT EXISTS atoms_au AFTER UPDATE ON atoms BEGIN
    INSERT INTO atoms_fts(atoms_fts, rowid, title, content, summary, tags)
    VALUES ('delete', old.rowid, old.title, old.content, old.summary, old.tags);
    INSERT INTO atoms_fts(rowid, title, content, summary, tags)
    VALUES (new.rowid, new.title, new.content, new.summary, new.tags);
END;
