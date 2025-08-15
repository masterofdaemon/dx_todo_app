use dioxus::prelude::*;
#[cfg(feature = "i18n")]
use dioxus_i18n::{prelude::*, t};
#[cfg(feature = "i18n")]
use unic_langid::langid;
use crate::models::Project;

#[derive(Clone, Copy)]
pub struct HeaderState {
    pub active_project: Signal<Option<Project>>, // a snapshot for display
}

#[component]
pub fn Header(count: usize, on_switch: EventHandler<()>, on_export: EventHandler<()>) -> Element {
    let state = use_context::<HeaderState>();

    // Select project label and language handlers
    // Use i18n only when feature is on AND not on Android
    #[cfg(all(feature = "i18n", not(target_os = "android")))]
    let (select_project_label, _change_en_btn, _change_ru_btn) = {
        let mut i = i18n();
        let label = t!("header.select_project").to_string();
        let change_en = move |_| i.set_language(langid!("en-US"));
        let change_ru = move |_| i.set_language(langid!("ru-RU"));
        (label, Some(change_en), Some(change_ru))
    };

    // Fallback: literals (Russian on Android, English elsewhere without i18n)
    #[cfg(any(not(feature = "i18n"), target_os = "android"))]
    let (select_project_label, _change_en_btn, _change_ru_btn) = {
        #[cfg(target_os = "android")]
        { ("Выберите проект".to_string(), None::<EventHandler<MouseEvent>>, None::<EventHandler<MouseEvent>>) }
        #[cfg(not(target_os = "android"))]
        { ("Select project".to_string(), None::<EventHandler<MouseEvent>>, None::<EventHandler<MouseEvent>>) }
    };
    let name = state
        .active_project
        .read()
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| select_project_label.clone());

    let items_text = format!("Items: {}", count);
    let project_text = format!("Project: {}", name.clone());

    rsx! {
        div { class: "header",
            // Left: compact info
            div { class: "meta", style: "display:flex; align-items:center; gap:10px;",
                span { { items_text } }
                span { { project_text } }
            }
            // Right: actions
            div { class: "actions", style: "display:flex; gap:8px; align-items:center;",
                // Language toggle (only when i18n feature is enabled and not Android)
                {
                    #[cfg(all(feature = "i18n", not(target_os = "android")))]
                    { rsx!{
                        button { class: "btn btn-ghost", onclick: _change_en_btn.unwrap(), "EN" }
                        button { class: "btn btn-ghost", onclick: _change_ru_btn.unwrap(), "RU" }
                    }}
                }
                // App actions
                {
                    #[cfg(target_os = "android")]
                    { rsx!{
                        button { class: "btn btn-ghost", onclick: move |_| on_switch.call(()), "Сменить" }
                        button { class: "btn btn-primary", onclick: move |_| on_export.call(()), "Экспорт" }
                    } }
                    #[cfg(not(target_os = "android"))]
                    { rsx!{
                        button { class: "btn btn-ghost", onclick: move |_| on_switch.call(()), "Switch" }
                        button { class: "btn btn-primary", onclick: move |_| on_export.call(()), "Export" }
                    } }
                }
            }
        }
    }
}
