use dioxus::prelude::*;
use uuid::Uuid;

use crate::api::{self, ApiError, Entity, Project};
use crate::routes::Route;

#[derive(Clone, Copy, PartialEq)]
enum SortCol {
    Name,
    CreatedAt,
}

#[derive(Clone, Copy, PartialEq)]
enum SortDir {
    Asc,
    Desc,
}

#[allow(non_snake_case)]
pub fn Projects() -> Element {
    projects_view(None)
}

#[allow(non_snake_case)]
#[component]
pub fn ProjectDetail(project_id: Uuid) -> Element {
    projects_view(Some(project_id))
}

fn projects_view(project_id: Option<Uuid>) -> Element {
    let projects = use_resource(api::fetch_projects);
    let mut filter = use_signal(String::new);
    let mut sort_col = use_signal(|| SortCol::CreatedAt);
    let mut sort_dir = use_signal(|| SortDir::Desc);

    let selected_entities: Resource<Option<Result<Vec<Entity>, ApiError>>> =
        use_resource(move || async move {
            let id = project_id?;
            Some(api::fetch_project_entities(id, 0, 50).await.map(|p| p.items))
        });

    let selected_project: Resource<Option<Result<Project, ApiError>>> =
        use_resource(move || async move {
            let id = project_id?;
            Some(api::fetch_project(id).await)
        });

    let description_entity: Resource<Option<Result<Entity, ApiError>>> =
        use_resource(move || async move {
            let id = project_id?;
            // fetch project first to get description_entity_id, then fetch that entity
            let project = api::fetch_project(id).await.ok()?;
            Some(api::fetch_entity(project.description_entity_id).await)
        });

    rsx! {
        div { class: "split-view",
            div { class: if project_id.is_some() { "list-pane list-pane--narrow" } else { "list-pane" },
                div { class: "pane-toolbar",
                    h2 { class: "pane-title", "Projects" }
                    input {
                        class: "filter-input",
                        r#type: "search",
                        placeholder: "Filter by name…",
                        value: "{filter}",
                        oninput: move |e| filter.set(e.value()),
                    }
                }

                div { style: "overflow-y: auto; flex: 1;",
                    match &*projects.read() {
                        None => rsx! { div { class: "loading", "Loading…" } },
                        Some(Err(e)) => rsx! { div { class: "error", "Error: {e}" } },
                        Some(Ok(list)) => {
                            let mut items: Vec<Project> = list
                                .iter()
                                .filter(|p| {
                                    let q = filter.read().to_lowercase();
                                    q.is_empty() || p.name.to_lowercase().contains(&q)
                                })
                                .cloned()
                                .collect();

                            match (*sort_col.read(), *sort_dir.read()) {
                                (SortCol::Name, SortDir::Asc) => items.sort_by(|a, b| a.name.cmp(&b.name)),
                                (SortCol::Name, SortDir::Desc) => items.sort_by(|a, b| b.name.cmp(&a.name)),
                                (SortCol::CreatedAt, SortDir::Asc) => items.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
                                (SortCol::CreatedAt, SortDir::Desc) => items.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
                            }

                            rsx! {
                                table { class: "data-table",
                                    thead {
                                        tr {
                                            th {
                                                class: "sortable",
                                                onclick: move |_| {
                                                    if *sort_col.read() == SortCol::Name {
                                                        sort_dir.set(if *sort_dir.read() == SortDir::Asc { SortDir::Desc } else { SortDir::Asc });
                                                    } else {
                                                        sort_col.set(SortCol::Name);
                                                        sort_dir.set(SortDir::Asc);
                                                    }
                                                },
                                                span { "Name " }
                                                if *sort_col.read() == SortCol::Name {
                                                    if *sort_dir.read() == SortDir::Asc { "↑" } else { "↓" }
                                                }
                                            }
                                            th {
                                                class: "sortable",
                                                onclick: move |_| {
                                                    if *sort_col.read() == SortCol::CreatedAt {
                                                        sort_dir.set(if *sort_dir.read() == SortDir::Asc { SortDir::Desc } else { SortDir::Asc });
                                                    } else {
                                                        sort_col.set(SortCol::CreatedAt);
                                                        sort_dir.set(SortDir::Desc);
                                                    }
                                                },
                                                span { "Created " }
                                                if *sort_col.read() == SortCol::CreatedAt {
                                                    if *sort_dir.read() == SortDir::Asc { "↑" } else { "↓" }
                                                }
                                            }
                                        }
                                    }
                                    tbody {
                                        for p in items {
                                            {
                                                let is_selected = project_id == Some(p.id);
                                                let pid = p.id;
                                                rsx! {
                                                    tr {
                                                        class: if is_selected { "row row--selected" } else { "row" },
                                                        onclick: move |_| {
                                                            navigator().push(Route::ProjectDetail { project_id: pid });
                                                        },
                                                        td { "{p.name}" }
                                                        td { class: "mono small", "{p.created_at}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(_id) = project_id {
                div { class: "detail-pane",
                    button {
                        class: "detail-close",
                        onclick: move |_| { navigator().push(Route::Projects {}); },
                        "✕"
                    }

                    match &*selected_project.read() {
                        None => rsx! { div { class: "loading", "Loading project…" } },
                        Some(None) => rsx! { div { class: "loading", "Loading…" } },
                        Some(Some(Err(e))) => rsx! { div { class: "error", "Error: {e}" } },
                        Some(Some(Ok(p))) => rsx! {
                            div { class: "detail-header",
                                h2 { class: "detail-title", "{p.name}" }
                                div { class: "detail-meta",
                                    span { class: "label", "ID" }
                                    span { class: "mono small", "{p.id}" }
                                }
                                div { class: "detail-meta",
                                    span { class: "label", "Created" }
                                    span { class: "mono small", "{p.created_at}" }
                                }
                                div { class: "detail-meta",
                                    span { class: "label", "Description entity" }
                                    Link {
                                        to: Route::EntityDetail { entity_id: p.description_entity_id },
                                        class: "entity-link mono small",
                                        "{p.description_entity_id}"
                                    }
                                }

                                div { class: "detail-section",
                                    h3 { class: "section-title", "Description" }
                                    match &*description_entity.read() {
                                        None | Some(None) => rsx! { div { class: "loading", "Loading…" } },
                                        Some(Some(Err(e))) => rsx! { div { class: "error", "Error: {e}" } },
                                        Some(Some(Ok(desc))) => {
                                            let preview = desc.content
                                                .get("text")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or_default()
                                                .to_string();
                                            rsx! {
                                                div { class: "prose-block", "{preview}" }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "detail-section",
                                h3 { class: "section-title", "Entities" }
                                match &*selected_entities.read() {
                                    None | Some(None) => rsx! { div { class: "loading", "Loading…" } },
                                    Some(Some(Err(e))) => rsx! { div { class: "error", "Error: {e}" } },
                                    Some(Some(Ok(entities))) => rsx! {
                                        table { class: "data-table data-table--compact",
                                            thead {
                                                tr {
                                                    th { "Type" }
                                                    th { "ID" }
                                                    th { "Created" }
                                                }
                                            }
                                            tbody {
                                                for e in entities {
                                                    {
                                                        let eid = e.id;
                                                        rsx! {
                                                            tr {
                                                                class: "row",
                                                                onclick: move |_| {
                                                                    navigator().push(Route::EntityDetail { entity_id: eid });
                                                                },
                                                                td { class: "badge", "{e.entity_type}" }
                                                                td { class: "mono small", "{e.id}" }
                                                                td { class: "mono small", "{e.created_at}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
