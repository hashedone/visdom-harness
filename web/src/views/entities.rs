use dioxus::prelude::*;
use uuid::Uuid;

use crate::api::{self, ApiError, Entity};
use crate::routes::Route;

#[allow(non_snake_case)]
pub fn Entities() -> Element {
    rsx! {
        div { class: "split-view",
            div { class: "list-pane",
                div { class: "pane-toolbar",
                    h2 { class: "pane-title", "Entities" }
                }
                div { class: "empty-state",
                    p { "Navigate to an entity from the Projects view." }
                    p { class: "small", "Select a project and click an entity row to inspect it." }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn EntityDetail(entity_id: Uuid) -> Element {
    let entity: Resource<Result<Entity, ApiError>> =
        use_resource(move || async move { api::fetch_entity(entity_id).await });

    rsx! {
        div { class: "split-view",
            // Narrow placeholder list on the left
            div { class: "list-pane list-pane--narrow",
                div { class: "pane-toolbar",
                    h2 { class: "pane-title", "Entities" }
                }
                div { class: "empty-state",
                    p { "Navigate to an entity from the Projects view." }
                }
            }

            div { class: "detail-pane",
                button {
                    class: "detail-close",
                    onclick: move |_| { navigator().push(Route::Entities {}); },
                    "✕"
                }

                match &*entity.read() {
                    None => rsx! { div { class: "loading", "Loading entity…" } },
                    Some(Err(e)) => rsx! { div { class: "error", "Error: {e}" } },
                    Some(Ok(e)) => rsx! {
                        div { class: "detail-header",
                            div { class: "detail-type-badge",
                                span { class: "badge badge--lg", "{e.entity_type}" }
                            }
                            div { class: "detail-meta",
                                span { class: "label", "ID" }
                                span { class: "mono small", "{e.id}" }
                            }
                            div { class: "detail-meta",
                                span { class: "label", "Project" }
                                Link {
                                    to: Route::ProjectDetail { project_id: e.project_id },
                                    class: "entity-link mono small",
                                    "{e.project_id}"
                                }
                            }
                            div { class: "detail-meta",
                                span { class: "label", "Created" }
                                span { class: "mono small", "{e.created_at}" }
                            }
                            if !e.contributing_entity_ids.is_empty() {
                                div { class: "detail-section",
                                    h3 { class: "section-title", "Contributing entities" }
                                    ul { class: "id-list",
                                        for cid in &e.contributing_entity_ids {
                                            {
                                                let cid = *cid;
                                                rsx! {
                                                    li {
                                                        Link {
                                                            to: Route::EntityDetail { entity_id: cid },
                                                            class: "entity-link mono small",
                                                            "{cid}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "detail-section",
                            h3 { class: "section-title", "Content" }
                            pre { class: "content-json",
                                {serde_json::to_string_pretty(&e.content).unwrap_or_default()}
                            }
                        }
                    }
                }
            }
        }
    }
}
