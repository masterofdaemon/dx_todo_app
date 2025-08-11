use dioxus::prelude::*;

use crate::models::Filter;

#[component]
pub fn FilterBar(
    active: Filter,
    on_all: EventHandler<MouseEvent>,
    on_active: EventHandler<MouseEvent>,
    on_completed: EventHandler<MouseEvent>,
    on_clear_completed: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "filters",
            div { class: "tabs",
                button { class: if matches!(active, Filter::All) { "tab active" } else { "tab" }, onclick: move |e| on_all.call(e), "All" }
                button { class: if matches!(active, Filter::Active) { "tab active" } else { "tab" }, onclick: move |e| on_active.call(e), "Active" }
                button { class: if matches!(active, Filter::Completed) { "tab active" } else { "tab" }, onclick: move |e| on_completed.call(e), "Completed" }
            }
            button { class: "btn btn-link danger", onclick: move |e| on_clear_completed.call(e), "Clear completed" }
        }
    }
}
