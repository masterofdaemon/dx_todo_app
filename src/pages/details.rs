use dioxus::events::Key;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

use crate::app::Route;
use crate::app_assets::head_nodes;
use crate::models::Subtask;
use crate::state::AppState;
use crate::storage::save_projects;

#[component]
pub fn Details(id: u64) -> Element {
    let state = use_context::<AppState>();
    let mut projects = state.projects;
    let active_project_id = state.active_project_id.read().unwrap_or(0);
    let nav = use_navigator();

    let todo_opt = projects
        .read()
        .iter()
        .find(|p| p.id == active_project_id)
        .and_then(|p| p.todos.iter().find(|t| t.id == id))
        .cloned();
    #[cfg(target_os = "android")]
    let not_found_lbl = "Не найдено";
    #[cfg(not(target_os = "android"))]
    let not_found_lbl = "Not found";
    let Some(todo) = todo_opt else {
        return rsx! { div { class: "app", div { class: "card", "{not_found_lbl}" } } };
    };

    let mut add_sub = move |title: String| {
        if title.trim().is_empty() {
            return;
        }
        if let Some(it) = projects
            .write()
            .iter_mut()
            .find(|p| p.id == active_project_id)
            .and_then(|p| p.todos.iter_mut().find(|t| t.id == id))
        {
            let next_sid = it.subtasks.iter().map(|s| s.id).max().unwrap_or(0) + 1;
            it.subtasks.push(Subtask {
                id: next_sid,
                title,
                completed: false,
            });
            it.completed = false;
        }
        save_projects(&projects.read());
    };
    let mut toggle_sub = move |sid: u64| {
        if let Some(it) = projects
            .write()
            .iter_mut()
            .find(|p| p.id == active_project_id)
            .and_then(|p| p.todos.iter_mut().find(|t| t.id == id))
        {
            if let Some(st) = it.subtasks.iter_mut().find(|s| s.id == sid) {
                st.completed = !st.completed;
            }
            if !it.subtasks.is_empty() {
                it.completed = it.subtasks.iter().all(|s| s.completed);
            }
        }
        save_projects(&projects.read());
    };
    let mut remove_sub = move |sid: u64| {
        if let Some(it) = projects
            .write()
            .iter_mut()
            .find(|p| p.id == active_project_id)
            .and_then(|p| p.todos.iter_mut().find(|t| t.id == id))
        {
            it.subtasks.retain(|s| s.id != sid);
            if !it.subtasks.is_empty() {
                it.completed = it.subtasks.iter().all(|s| s.completed);
            } else {
                // No subtasks: do not auto-complete; leave as-is
            }
        }
        save_projects(&projects.read());
    };
    let mut update_desc = move |v: String| {
        if let Some(it) = projects
            .write()
            .iter_mut()
            .find(|p| p.id == active_project_id)
            .and_then(|p| p.todos.iter_mut().find(|t| t.id == id))
        {
            it.description = v;
        }
        save_projects(&projects.read());
    };

    let mut sub_input = use_signal(String::new);

    rsx! {
        { head_nodes() }
        div { class: "app",
            div { class: "card",
                div { class: "row between", style: "margin-bottom:12px;",
                    {
                        #[cfg(target_os = "android")]
                        { rsx!{ button { class: "btn btn-ghost", onclick: move |_| { nav.push(Route::List {}); }, "← Назад" } } }
                        #[cfg(not(target_os = "android"))]
                        { rsx!{ button { class: "btn btn-ghost", onclick: move |_| { nav.push(Route::List {}); }, "← Back" } } }
                    }
                }
                h2 { class: "title", "{todo.title}" }
                {
                    #[cfg(target_os = "android")]
                    { rsx!{ textarea { class: "text desc", rows: "3", placeholder: "Описание...", value: "{todo.description}", oninput: move |e| update_desc(e.value()) } } }
                    #[cfg(not(target_os = "android"))]
                    { rsx!{ textarea { class: "text desc", rows: "3", placeholder: "Description...", value: "{todo.description}", oninput: move |e| update_desc(e.value()) } } }
                }
                {
                    #[cfg(target_os = "android")]
                    { rsx!{ h3 { style: "margin-top:16px;", "Подзадачи" } } }
                    #[cfg(not(target_os = "android"))]
                    { rsx!{ h3 { style: "margin-top:16px;", "Subtasks" } } }
                }
                ul { class: "subtasks",
                    for st in todo.subtasks.clone().into_iter() {
                        li { key: "sub-{st.id}", class: "sub-item",
                            input { r#type: "checkbox", checked: st.completed, onclick: move |_| toggle_sub(st.id) }
                            span { class: if st.completed { "sub-title completed" } else { "sub-title" }, "{st.title}" }
                            button { class: "btn btn-ghost sub-remove", onclick: move |_| remove_sub(st.id), "✕" }
                        }
                    }
                    li { class: "sub-add",
                        {
                            #[cfg(target_os = "android")]
                            { rsx!{
                                input { class: "text sub-input", r#type: "text", placeholder: "Добавить подзадачу…", value: "{sub_input.read()}", oninput: move |e| sub_input.set(e.value()), onkeydown: move |e| { if e.key() == Key::Enter { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } } } }
                                button { class: "btn btn-primary sub-add-btn", onclick: move |_| { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } }, "Добавить" }
                            } }
                            #[cfg(not(target_os = "android"))]
                            { rsx!{
                                input { class: "text sub-input", r#type: "text", placeholder: "Add a subtask…", value: "{sub_input.read()}", oninput: move |e| sub_input.set(e.value()), onkeydown: move |e| { if e.key() == Key::Enter { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } } } }
                                button { class: "btn btn-primary sub-add-btn", onclick: move |_| { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } }, "Add" }
                            } }
                        }
                    }
                }
            }
        }
    }
}
