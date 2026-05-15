mod api;
mod layout;
mod routes;
mod views;

use dioxus::prelude::*;

use routes::Route;

fn main() {
    dioxus::launch(App);
}

#[allow(non_snake_case)]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/style.css") }
        Router::<Route> {}
    }
}
