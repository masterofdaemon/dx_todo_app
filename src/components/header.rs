use dioxus::prelude::*;
use crate::models::Project;

#[derive(Clone, Copy)]
pub struct HeaderState {
    pub active_project: Signal<Option<Project>>, // a snapshot for display
}

#[component]
pub fn Header(count: usize, on_switch: EventHandler<()>, on_export: EventHandler<()>) -> Element {
    let state = use_context::<HeaderState>();
    let name = state
        .active_project
        .read()
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Select a project".to_string());
    rsx! {
        div { class: "header",
            h1 { class: "title", "To-Do" }
            span { class: "meta", "{count} items" }
            div { class: "actions", style: "margin-left:auto; display:flex; gap:8px;",
                span { class: "meta", "Project: {name}" }
                button { class: "btn btn-ghost", onclick: move |_| on_switch.call(()), "Switch" }
                button { class: "btn btn-primary", onclick: move |_| on_export.call(()), "Export to PDF" }
            }
        }
    }
}
