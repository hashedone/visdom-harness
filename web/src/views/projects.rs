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
    rsx! { ProjectsView { selected_id: None } }
}

#[allow(non_snake_case)]
#[component]
pub fn ProjectDetail(project_id: Uuid) -> Element {
    rsx! { ProjectsView { selected_id: Some(project_id) } }
}

#[allow(non_snake_case)]
#[component]
fn ProjectDetailPane(project_id: Uuid) -> Element {
    let selected_entities: Resource<Result<Vec<Entity>, ApiError>> =
        use_resource(move || async move {
            api::fetch_project_entities(project_id, 0, 50)
                .await
                .map(|p| p.items)
        });

    let selected_project: Resource<Result<Project, ApiError>> =
        use_resource(move || async move { api::fetch_project(project_id).await });

    let description_entity: Resource<Option<Result<Entity, ApiError>>> =
        use_resource(move || async move {
            let project = api::fetch_project(project_id).await.ok()?;
            Some(api::fetch_entity(project.description_entity_id).await)
        });

    rsx! {
        div { class: "detail-pane",
            button {
                class: "detail-close",
                onclick: move |_| { navigator().push(Route::Projects {}); },
                "✕"
            }

            match &*selected_project.read() {
                None => rsx! { div { class: "loading", "Loading project…" } },
                Some(Err(e)) => rsx! { div { class: "error", "Error: {e}" } },
                Some(Ok(p)) => rsx! {
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
                            None => rsx! { div { class: "loading", "Loading…" } },
                            Some(Err(e)) => rsx! { div { class: "error", "Error: {e}" } },
                            Some(Ok(entities)) => rsx! {
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
                                                        td { span { class: "badge", "{e.entity_type}" } }
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

#[allow(non_snake_case)]
#[component]
fn ProjectsView(selected_id: Option<Uuid>) -> Element {
    let mut projects = use_resource(api::fetch_projects);
    let mut filter = use_signal(String::new);
    let mut sort_col = use_signal(|| SortCol::CreatedAt);
    let mut sort_dir = use_signal(|| SortDir::Desc);
    let mut show_form = use_signal(|| false);
    let mut new_name = use_signal(String::new);
    let mut new_desc = use_signal(String::new);
    let mut submit_error: Signal<Option<String>> = use_signal(|| None);

    let has_detail = selected_id.is_some();

    rsx! {
        div { class: "split-view",
            div { class: if has_detail { "list-pane list-pane--narrow" } else { "list-pane" },
                div { class: "pane-toolbar",
                    h2 { class: "pane-title", "Projects" }
                    input {
                        class: "filter-input",
                        r#type: "search",
                        placeholder: "Filter by name…",
                        value: "{filter}",
                        oninput: move |e| filter.set(e.value()),
                    }
                    button {
                        class: "btn-icon",
                        title: "New project",
                        onclick: move |_| {
                            let cur = *show_form.read();
                            show_form.set(!cur);
                        },
                        if *show_form.read() { "✕" } else { "+" }
                    }
                }

                if *show_form.read() {
                    div { class: "new-project-form",
                        input {
                            class: "form-input",
                            r#type: "text",
                            placeholder: "Project name",
                            value: "{new_name}",
                            oninput: move |e| new_name.set(e.value()),
                        }
                        textarea {
                            class: "form-textarea",
                            placeholder: "Description",
                            value: "{new_desc}",
                            oninput: move |e| new_desc.set(e.value()),
                        }
                        if let Some(err) = &*submit_error.read() {
                            div { class: "form-error", "{err}" }
                        }
                        div { class: "form-actions",
                            button {
                                class: "btn-primary",
                                onclick: move |_| {
                                    let name = new_name.read().trim().to_string();
                                    let desc = new_desc.read().trim().to_string();
                                    if name.is_empty() {
                                        submit_error.set(Some("Name is required.".into()));
                                        return;
                                    }
                                    if desc.is_empty() {
                                        submit_error.set(Some("Description is required.".into()));
                                        return;
                                    }
                                    submit_error.set(None);
                                    spawn(async move {
                                        match api::create_project(&name, &desc).await {
                                            Ok(_) => {
                                                new_name.set(String::new());
                                                new_desc.set(String::new());
                                                show_form.set(false);
                                                projects.restart();
                                            }
                                            Err(e) => {
                                                submit_error.set(Some(e.to_string()));
                                            }
                                        }
                                    });
                                },
                                "Create"
                            }
                            button {
                                class: "btn-ghost",
                                onclick: move |_| {
                                    show_form.set(false);
                                    submit_error.set(None);
                                },
                                "Cancel"
                            }
                        }
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
                                                let is_selected = selected_id == Some(p.id);
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

            if let Some(pid) = selected_id {
                ProjectDetailPane { key: "{pid}", project_id: pid }
            }
        }
    }
}
