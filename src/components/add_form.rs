use dioxus::prelude::*;

#[component]
pub fn AddForm(
    value: String,
    on_input: EventHandler<FormEvent>,
    on_enter: EventHandler<KeyboardEvent>,
    on_add: EventHandler<MouseEvent>,
) -> Element {
    // Labels: English default, Russian on Android
    #[cfg(target_os = "android")]
    let (ph_lbl, add_lbl) = ("Добавить задачу...", "Добавить");
    #[cfg(not(target_os = "android"))]
    let (ph_lbl, add_lbl) = ("Add a task...", "Add");

    rsx! {
        div { class: "add",
            input {
                class: "text",
                r#type: "text",
                placeholder: "{ph_lbl}",
                value: "{value}",
                oninput: move |e| on_input.call(e),
                onkeydown: move |e| on_enter.call(e),
            }
            button { class: "btn btn-primary", onclick: move |e| on_add.call(e), "{add_lbl}" }
        }
    }
}
