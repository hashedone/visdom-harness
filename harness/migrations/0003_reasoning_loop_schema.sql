-- Entity references: replaces contributing_entity_ids_json with a proper m2m table.
-- Existing JSON data is migrated below before the column is dropped.
CREATE TABLE IF NOT EXISTS entity_references (
    source_id BLOB NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    target_id BLOB NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    PRIMARY KEY (source_id, target_id)
);

-- Migrate existing contributing_entity_ids_json data into entity_references.
-- Each row in entities has a JSON array of UUIDs; we expand them into rows.
-- SQLite's json_each is available from 3.38+ (bundled with sqlx's libsqlite3).
INSERT OR IGNORE INTO entity_references (source_id, target_id)
SELECT
    e.id AS source_id,
    CAST(ref.value AS BLOB) AS target_id
FROM entities e, json_each(e.contributing_entity_ids_json) AS ref
WHERE e.contributing_entity_ids_json != '[]';

-- Drop the now-redundant JSON column.
-- SQLite requires recreating the table to drop a column (pre-3.35 workaround not needed;
-- sqlx bundles 3.46+ which supports DROP COLUMN directly).
ALTER TABLE entities DROP COLUMN contributing_entity_ids_json;

-- Questions: stores questions asked by the reasoning loop.
CREATE TABLE IF NOT EXISTS questions (
    id BLOB PRIMARY KEY,
    project_id BLOB NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    instructions TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('open', 'answered')) DEFAULT 'open',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_questions_project_status ON questions(project_id, status);

-- Answers: links an answer (stored as a raw entity) to the question it answers.
-- Multiple answers per question are valid (multiple integrations may reply).
CREATE TABLE IF NOT EXISTS answers (
    id BLOB PRIMARY KEY,
    question_id BLOB NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    entity_id BLOB NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    received_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_answers_question ON answers(question_id);

-- Add current_summary_id to projects: tracks the most recent summary entity.
ALTER TABLE projects ADD COLUMN current_summary_id BLOB REFERENCES entities(id);
