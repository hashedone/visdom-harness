use dioxus::prelude::*;

use crate::routes::Route;

#[allow(non_snake_case)]
pub fn Layout() -> Element {
    let mut nav_open = use_signal(|| false);

    rsx! {
        div { class: "app-shell",
            // Mobile overlay backdrop
            if *nav_open.read() {
                div {
                    class: "nav-backdrop",
                    onclick: move |_| nav_open.set(false),
                }
            }

            // Left nav
            nav {
                class: if *nav_open.read() { "sidebar sidebar--open" } else { "sidebar" },
                div { class: "sidebar-header",
                    span { class: "sidebar-brand", "Visdom" }
                    button {
                        class: "burger",
                        onclick: move |_| nav_open.set(false),
                        "✕"
                    }
                }
                ul { class: "nav-list",
                    li {
                        Link {
                            to: Route::Projects {},
                            class: "nav-link",
                            onclick: move |_| nav_open.set(false),
                            "Projects"
                        }
                    }
                    li {
                        Link {
                            to: Route::Entities {},
                            class: "nav-link",
                            onclick: move |_| nav_open.set(false),
                            "Entities"
                        }
                    }
                }
            }

            // Main area
            div { class: "main-area",
                header { class: "topbar",
                    button {
                        class: "burger burger--top",
                        onclick: move |_| {
                            let current = *nav_open.read();
                            nav_open.set(!current);
                        },
                        "☰"
                    }
                }
                div { class: "content-area",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
