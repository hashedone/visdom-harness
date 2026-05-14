INSERT INTO inferences (id, system_prompt, request_messages_json, response_text, tool_calls_json)
VALUES (?, ?, ?, ?, ?)
RETURNING id, system_prompt, request_messages_json, response_text, tool_calls_json, created_at
