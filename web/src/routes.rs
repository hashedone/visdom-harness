use dioxus::prelude::*;
use uuid::Uuid;

use crate::layout::Layout;
use crate::views::entities::{Entities, EntityDetail};
use crate::views::projects::{ProjectDetail, Projects};

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[redirect("/", || Route::Projects {})]
    #[route("/projects")]
    Projects {},
    #[route("/projects/:project_id")]
    ProjectDetail { project_id: Uuid },
    #[route("/entities")]
    Entities {},
    #[route("/entities/:entity_id")]
    EntityDetail { entity_id: Uuid },
}
