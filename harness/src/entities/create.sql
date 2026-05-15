INSERT INTO entities (id, project_id, entity_type, content_json, contributing_entity_ids_json)
VALUES (?, ?, ?, ?, ?) RETURNING *
