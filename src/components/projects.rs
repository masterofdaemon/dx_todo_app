use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use crate::models::Project;
use crate::storage::save_projects;
use crate::Route;

#[derive(Clone, Copy)]
pub struct ProjectsState {
    pub projects: Signal<Vec<Project>>,
    pub active_project_id: Signal<Option<u64>>,
}

#[component]
pub fn Projects() -> Element {
    let state = use_context::<ProjectsState>();
    let mut projects = state.projects;
    let mut active = state.active_project_id;
    let nav = use_navigator();

    let mut new_name = use_signal(String::new);

    let mut add_project = move |name: String| {
        let name = name.trim().to_string();
        if name.is_empty() { return; }
        let next_id = projects.read().iter().map(|p| p.id).max().unwrap_or(0) + 1;
        projects.write().push(Project { id: next_id, name: name.clone(), todos: Vec::new() });
        active.set(Some(next_id));
        save_projects(&projects.read());
        println!("[Projects] Navigating to List after add");
        nav.push(Route::List {});
    };

    rsx! {
        div { class: "app",
            div { class: "card",
                h2 { class: "title", "Projects" }
                ul { class: "list",
                    for p in projects.read().iter().cloned() {
                        li { class: "list-item",
                            div { class: "content",
                                div { class: "item-title", "{p.name}" }
                            }
                            div { class: "actions",
                                button { class: "btn btn-primary", onclick: move |_| { println!("[Projects] Open clicked for id={} name={}", p.id, p.name); active.set(Some(p.id)); println!("[Projects] Navigating to List after open"); nav.push(Route::List {}); }, "Open" }
                            }
                        }
                    }
                }
                div { class: "row", style: "gap:8px; margin-top: 12px;",
                    input { class: "text", placeholder: "New project name", value: "{new_name.read()}", oninput: move |e| new_name.set(e.value()) }
                    button { class: "btn btn-primary", onclick: move |_| { let n = new_name.read().clone(); if !n.trim().is_empty() { println!("[Projects] Add clicked with name={}", n); add_project(n); new_name.set(String::new()); } }, "Add" }
                }
            }
        }
    }
}
