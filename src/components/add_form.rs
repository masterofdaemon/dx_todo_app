use dioxus::prelude::*;

#[component]
pub fn AddForm(
    value: String,
    on_input: EventHandler<FormEvent>,
    on_enter: EventHandler<KeyboardEvent>,
    on_add: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "add",
            input {
                class: "text",
                r#type: "text",
                placeholder: "Add a task...",
                value: "{value}",
                oninput: move |e| on_input.call(e),
                onkeydown: move |e| on_enter.call(e),
            }
            button { class: "btn btn-primary", onclick: move |e| on_add.call(e), "Add" }
        }
    }
}
