use dioxus::prelude::*;

#[component]
pub fn Header(count: usize) -> Element {
    rsx! {
        div { class: "header",
            h1 { class: "title", "To-Do" }
            span { class: "meta", "{count} items" }
        }
    }
}
