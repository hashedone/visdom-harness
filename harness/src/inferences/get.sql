SELECT id, system_prompt, request_messages_json, response_text, tool_calls_json, created_at
FROM inferences
WHERE id = ?
