use dioxus::prelude::*;
use uuid::Uuid;

use crate::api::{self, ApiError, Entity, Page};
use crate::routes::Route;

const PAGE_SIZE: i64 = 50;

#[derive(Clone, Copy, PartialEq)]
enum SortCol {
    EntityType,
    CreatedAt,
}

#[derive(Clone, Copy, PartialEq)]
enum SortDir {
    Asc,
    Desc,
}

#[allow(non_snake_case)]
pub fn Entities() -> Element {
    entities_view(None)
}

#[allow(non_snake_case)]
#[component]
pub fn EntityDetail(entity_id: Uuid) -> Element {
    entities_view(Some(entity_id))
}

fn entities_view(selected_id: Option<Uuid>) -> Element {
    let mut project_filter = use_signal(|| Option::<Uuid>::None);
    let mut type_filter = use_signal(String::new);
    let mut sort_col = use_signal(|| SortCol::CreatedAt);
    let mut sort_dir = use_signal(|| SortDir::Desc);
    let mut offset = use_signal(|| 0i64);

    // Reset to page 0 when the project filter changes
    let pf_for_fetch = *project_filter.read();
    let offset_val = *offset.read();

    let entities: Resource<Result<Page<Entity>, ApiError>> = use_resource(move || async move {
        let _ = pf_for_fetch; // track dependency
        match pf_for_fetch {
            None => api::fetch_entities(offset_val, PAGE_SIZE).await,
            Some(pid) => api::fetch_project_entities(pid, offset_val, PAGE_SIZE).await,
        }
    });

    let selected_entity: Resource<Option<Result<Entity, ApiError>>> =
        use_resource(move || async move {
            let id = selected_id?;
            Some(api::fetch_entity(id).await)
        });

    rsx! {
        div { class: "split-view",
            div { class: if selected_id.is_some() { "list-pane list-pane--narrow" } else { "list-pane" },
                div { class: "pane-toolbar",
                    h2 { class: "pane-title", "Entities" }
                    input {
                        class: "filter-input",
                        r#type: "search",
                        placeholder: "Filter by type…",
                        value: "{type_filter}",
                        oninput: move |e| type_filter.set(e.value()),
                    }
                }

                div { style: "overflow-y: auto; flex: 1;",
                    match &*entities.read() {
                        None => rsx! { div { class: "loading", "Loading…" } },
                        Some(Err(e)) => rsx! { div { class: "error", "Error: {e}" } },
                        Some(Ok(page)) => {
                            let tf = type_filter.read().to_lowercase();
                            let mut items: Vec<Entity> = page.items
                                .iter()
                                .filter(|e| tf.is_empty() || e.entity_type.to_string().contains(&*tf))
                                .cloned()
                                .collect();

                            match (*sort_col.read(), *sort_dir.read()) {
                                (SortCol::EntityType, SortDir::Asc) => {
                                    items.sort_by_key(|e| e.entity_type.to_string())
                                }
                                (SortCol::EntityType, SortDir::Desc) => {
                                    items.sort_by_key(|e| std::cmp::Reverse(e.entity_type.to_string()))
                                }
                                (SortCol::CreatedAt, SortDir::Asc) => {
                                    items.sort_by(|a, b| a.created_at.cmp(&b.created_at))
                                }
                                (SortCol::CreatedAt, SortDir::Desc) => {
                                    items.sort_by(|a, b| b.created_at.cmp(&a.created_at))
                                }
                            }

                            let total = page.total;
                            let current_offset = page.offset;
                            let has_prev = current_offset > 0;
                            let has_next = current_offset + PAGE_SIZE < total;
                            let page_num = current_offset / PAGE_SIZE + 1;
                            let total_pages = (total + PAGE_SIZE - 1) / PAGE_SIZE;

                            rsx! {
                                table { class: "data-table",
                                    thead {
                                        tr {
                                            th {
                                                class: "sortable",
                                                onclick: move |_| {
                                                    if *sort_col.read() == SortCol::EntityType {
                                                        sort_dir.set(if *sort_dir.read() == SortDir::Asc { SortDir::Desc } else { SortDir::Asc });
                                                    } else {
                                                        sort_col.set(SortCol::EntityType);
                                                        sort_dir.set(SortDir::Asc);
                                                    }
                                                },
                                                span { "Type " }
                                                if *sort_col.read() == SortCol::EntityType {
                                                    if *sort_dir.read() == SortDir::Asc { "↑" } else { "↓" }
                                                }
                                            }
                                            th { "Project" }
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
                                        for e in items {
                                            {
                                                let is_selected = selected_id == Some(e.id);
                                                let eid = e.id;
                                                let pid = e.project_id;
                                                rsx! {
                                                    tr {
                                                        class: if is_selected { "row row--selected" } else { "row" },
                                                        onclick: move |_| {
                                                            navigator().push(Route::EntityDetail { entity_id: eid });
                                                        },
                                                        td { span { class: "badge", "{e.entity_type}" } }
                                                        td {
                                                            class: "mono small",
                                                            onclick: move |evt| {
                                                                evt.stop_propagation();
                                                                project_filter.set(Some(pid));
                                                                offset.set(0);
                                                            },
                                                            "{e.project_id}"
                                                        }
                                                        td { class: "mono small", "{e.created_at}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "pagination",
                                    button {
                                        class: "pagination__btn",
                                        disabled: !has_prev,
                                        onclick: move |_| offset.set((current_offset - PAGE_SIZE).max(0)),
                                        "← Prev"
                                    }
                                    span { class: "pagination__info",
                                        "Page {page_num} of {total_pages} ({total} total)"
                                    }
                                    button {
                                        class: "pagination__btn",
                                        disabled: !has_next,
                                        onclick: move |_| offset.set(current_offset + PAGE_SIZE),
                                        "Next →"
                                    }
                                }

                                if project_filter.read().is_some() {
                                    div { class: "filter-banner",
                                        span { "Filtered by project" }
                                        button {
                                            class: "filter-clear",
                                            onclick: move |_| {
                                                project_filter.set(None);
                                                offset.set(0);
                                            },
                                            "✕ Clear"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(_id) = selected_id {
                div { class: "detail-pane",
                    button {
                        class: "detail-close",
                        onclick: move |_| { navigator().push(Route::Entities {}); },
                        "✕"
                    }

                    match &*selected_entity.read() {
                        None => rsx! { div { class: "loading", "Loading entity…" } },
                        Some(None) => rsx! { div { class: "loading", "Loading…" } },
                        Some(Some(Err(e))) => rsx! { div { class: "error", "Error: {e}" } },
                        Some(Some(Ok(e))) => rsx! {
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
                                div { class: "content-fields",
                                    for (key, val) in e.content.as_object().into_iter().flatten() {
                                        if !val.is_null() {
                                            {
                                                let key = key.clone();
                                                let is_text = key == "text";
                                                let val_str = val.as_str()
                                                    .map(|s| s.to_string())
                                                    .unwrap_or_else(|| val.to_string());
                                                rsx! {
                                                    div {
                                                        div { class: "content-field__label", "{key}" }
                                                        if is_text {
                                                            div { class: "prose-block", "{val_str}" }
                                                        } else {
                                                            span { class: "content-field__scalar", "{val_str}" }
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
