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
            // Left: compact info
            div { class: "meta", style: "display:flex; align-items:center; gap:10px;",
                span { "{count} items" }
                span { "Project: {name}" }
            }
            // Right: actions
            div { class: "actions", style: "display:flex; gap:8px;",
                button { class: "btn btn-ghost", onclick: move |_| on_switch.call(()), "Switch" }
                button { class: "btn btn-primary", onclick: move |_| on_export.call(()), "Export to PDF" }
            }
        }
    }
}
