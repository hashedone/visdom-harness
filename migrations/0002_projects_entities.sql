CREATE TABLE IF NOT EXISTS projects (
    id BLOB PRIMARY KEY,
    name TEXT NOT NULL,
    description_entity_id BLOB,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS entities (
    id BLOB PRIMARY KEY,
    project_id BLOB NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL CHECK(entity_type IN ('raw', 'knowledge', 'summary')),
    content_json TEXT NOT NULL,
    contributing_entity_ids_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_entities_project_created ON entities(project_id, created_at DESC);
