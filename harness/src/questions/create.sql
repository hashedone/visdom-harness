INSERT INTO questions (id, project_id, question, instructions, status)
VALUES (?, ?, ?, ?, 'open')
RETURNING id, project_id, question, instructions, status, created_at
