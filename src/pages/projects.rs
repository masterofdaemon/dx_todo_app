use dioxus::prelude::*;

use crate::components::projects::ProjectsState;

#[component]
pub fn Projects() -> Element {
    let _ = use_context::<ProjectsState>();
    rsx! { div { class: "app", crate::components::projects::Projects {} } }
}
