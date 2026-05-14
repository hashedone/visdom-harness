CREATE TABLE IF NOT EXISTS inferences (
    id TEXT PRIMARY KEY,
    system_prompt TEXT NOT NULL,
    request_messages_json TEXT NOT NULL,
    response_text TEXT NOT NULL,
    tool_calls_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_inferences_created_at ON inferences (created_at DESC);
