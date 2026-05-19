SELECT id, project_id, question, instructions, status, created_at
FROM questions
WHERE id = ?
