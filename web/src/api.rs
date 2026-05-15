use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description_entity_id: Uuid,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Raw,
    Knowledge,
    Summary,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Raw => write!(f, "raw"),
            EntityType::Knowledge => write!(f, "knowledge"),
            EntityType::Summary => write!(f, "summary"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Entity {
    pub id: Uuid,
    pub project_id: Uuid,
    pub entity_type: EntityType,
    pub content: serde_json::Value,
    pub contributing_entity_ids: Vec<Uuid>,
    pub created_at: String,
}

const BASE_URL: &str = "http://127.0.0.1:3000";

pub async fn fetch_projects() -> Result<Vec<Project>, String> {
    let resp = gloo_net::http::Request::get(&format!("{BASE_URL}/projects"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Vec<Project>>().await.map_err(|e| e.to_string())
}

pub async fn fetch_project(id: Uuid) -> Result<Project, String> {
    let resp = gloo_net::http::Request::get(&format!("{BASE_URL}/projects/{id}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status() == 404 {
        return Err("Not found".into());
    }
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Project>().await.map_err(|e| e.to_string())
}

pub async fn fetch_project_entities(project_id: Uuid) -> Result<Vec<Entity>, String> {
    let resp = gloo_net::http::Request::get(&format!("{BASE_URL}/projects/{project_id}/entities"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Vec<Entity>>().await.map_err(|e| e.to_string())
}

pub async fn fetch_entity(id: Uuid) -> Result<Entity, String> {
    let resp = gloo_net::http::Request::get(&format!("{BASE_URL}/entities/{id}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status() == 404 {
        return Err("Not found".into());
    }
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Entity>().await.map_err(|e| e.to_string())
}
