SELECT id, project_id, question, instructions, status, created_at
FROM questions
WHERE project_id = ?
ORDER BY created_at DESC
LIMIT ? OFFSET ?
