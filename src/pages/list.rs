use dioxus::events::Key;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

use crate::components::{add_form::AddForm, filter_bar::FilterBar, header::Header};
use crate::export::export_active_project_pdf;
use crate::models::{Filter, Todo};
use crate::state::{AppState, FlashState};
use crate::storage::save_projects;
use crate::app::Route;

#[component]
pub fn List() -> Element {
    let state = use_context::<AppState>();
    let mut projects = state.projects;
    let active_project_id = state.active_project_id.read();
    let mut new_title = state.new_title;
    let mut editing_id = state.editing_id;
    let mut editing_text = state.editing_text;
    let mut filter = state.filter;
    let mut next_id = state.next_id;
    let nav = use_navigator();
    let mut flash = use_context::<FlashState>();

    #[cfg(target_os = "android")]
    let no_active_lbl = "Активный проект не выбран. Откройте раздел Проекты.";
    #[cfg(not(target_os = "android"))]
    let no_active_lbl = "No active project selected. Go to Projects.";

    let Some(active_id) = *active_project_id else {
        return rsx! { div { class: "app", div { class: "card", "{no_active_lbl}" } } };
    };

    let project = projects.read().iter().find(|p| p.id == active_id).cloned();
    #[cfg(target_os = "android")]
    let not_found_project_lbl = "Проект не найден";
    #[cfg(not(target_os = "android"))]
    let not_found_project_lbl = "Project not found";

    let Some(proj) = project else {
        return rsx! { div { class: "app", div { class: "card", "{not_found_project_lbl}" } } };
    };

    let active_filter = *filter.read();
    let count = match active_filter {
        Filter::All => proj.todos.len(),
        Filter::Active => proj.todos.iter().filter(|t| !t.completed).count(),
        Filter::Completed => proj.todos.iter().filter(|t| t.completed).count(),
    };
    let visible: Vec<Todo> = match active_filter {
        Filter::All => proj.todos.clone(),
        Filter::Active => proj
            .todos
            .iter()
            .cloned()
            .filter(|t| !t.completed)
            .collect(),
        Filter::Completed => proj.todos.iter().cloned().filter(|t| t.completed).collect(),
    };

    let on_switch = move |_| {
        nav.push(Route::Projects {});
    };
    let on_export = move |_| {
        println!("[Export] Clicked");
        match export_active_project_pdf(&projects.read(), Some(active_id)) {
            Ok(()) => {
                println!("[Export] Success");
                #[cfg(target_os = "android")]
                flash.msg.set(Some("Экспортировано в Загрузки".to_string()));
                #[cfg(not(target_os = "android"))]
                flash.msg.set(Some("Exported to Downloads".to_string()));
            }
            Err(e) => {
                eprintln!("[Export] Error: {e}");
                flash.msg.set(Some(format!("Export failed: {e}")));
            }
        }
    };
    let on_input = move |e: FormEvent| {
        new_title.set(e.value());
    };
    let on_enter = move |e: KeyboardEvent| {
        if e.key() == Key::Enter {
            let title = new_title.read().trim().to_string();
            if !title.is_empty() {
                if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                    let id = *next_id.read();
                    *next_id.write() = id + 1;
                    p.todos.push(Todo {
                        id,
                        title: title.clone(),
                        completed: false,
                        description: String::new(),
                        subtasks: Vec::new(),
                    });
                }
                save_projects(&projects.read());
                new_title.set(String::new());
            }
        }
    };
    let on_add = move |_| {
        let title = new_title.read().trim().to_string();
        if !title.is_empty() {
            if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                let id = *next_id.read();
                *next_id.write() = id + 1;
                p.todos.push(Todo {
                    id,
                    title: title.clone(),
                    completed: false,
                    description: String::new(),
                    subtasks: Vec::new(),
                });
            }
            save_projects(&projects.read());
            new_title.set(String::new());
        }
    };

    let on_all = move |_| {
        filter.set(Filter::All);
    };
    let on_active_f = move |_| {
        filter.set(Filter::Active);
    };
    let on_completed_f = move |_| {
        filter.set(Filter::Completed);
    };
    let on_clear_completed = move |_| {
        if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
            p.todos.retain(|t| !t.completed);
        }
        save_projects(&projects.read());
    };

    rsx! {
        div { class: "app",
            Header { count: count, on_switch: on_switch, on_export: on_export }
            div { class: "card", style: "background:#ffffff; color:#0f172a; border:1px solid rgba(15,23,42,0.12); border-radius:16px; margin:12px auto; padding:18px; box-shadow: 0 10px 25px rgba(0,0,0,0.08);",
                AddForm { value: new_title.read().clone(), on_input: on_input, on_enter: on_enter, on_add: on_add }
                FilterBar { active: active_filter, on_all: on_all, on_active: on_active_f, on_completed: on_completed_f, on_clear_completed: on_clear_completed }
                { if visible.is_empty() {
                    #[cfg(target_os = "android")]
                    let empty_lbl = "Задач пока нет. Добавьте новую выше.";
                    #[cfg(not(target_os = "android"))]
                    let empty_lbl = "No tasks yet. Add one above.";
                    rsx!{ div { style: "opacity:.8; color:var(--muted); padding: 16px 4px;", "{empty_lbl}" } }
                } else {
                    rsx!{ ul { class: "list",
                        for todo in visible.into_iter() {
                            crate::components::todo_item::TodoItem {
                                key: "todo-{todo.id}",
                                todo: todo.clone(),
                                is_editing: matches!(*editing_id.read(), Some(id) if id == todo.id),
                                editing_text: editing_text.read().clone(),
                                on_toggle: move |_| {
                                    if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                                        if let Some(t) = p.todos.iter_mut().find(|t| t.id == todo.id) { t.completed = !t.completed; }
                                    }
                                    save_projects(&projects.read());
                                },
                                on_start_edit: move |_| { editing_id.set(Some(todo.id)); editing_text.set(todo.title.clone()); },
                                on_remove: move |_| {
                                    if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) { p.todos.retain(|t| t.id != todo.id); }
                                    save_projects(&projects.read());
                                },
                                on_save_click: move |_| {
                                    if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                                        if let Some(t) = p.todos.iter_mut().find(|t| t.id == todo.id) { t.title = editing_text.read().clone(); }
                                    }
                                    editing_id.set(None);
                                    editing_text.set(String::new());
                                    save_projects(&projects.read());
                                },
                                on_save_key: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                                            if let Some(t) = p.todos.iter_mut().find(|t| t.id == todo.id) { t.title = editing_text.read().clone(); }
                                        }
                                        editing_id.set(None);
                                        editing_text.set(String::new());
                                        save_projects(&projects.read());
                                    } else if e.key() == Key::Escape {
                                        editing_id.set(None);
                                        editing_text.set(String::new());
                                    }
                                },
                                on_edit_input: move |e: FormEvent| { editing_text.set(e.value()); },
                                on_cancel: move |_| { editing_id.set(None); editing_text.set(String::new()); },
                                on_drag_start: move |_id: u64| {},
                                on_drag_over: move |_id: u64| {},
                                on_drag_leave: move |_id: u64| {},
                                on_drag_end: move |_id: u64| {},
                                on_drop: move |_id: u64| {},
                                is_dragging: false,
                                is_drag_over: false,
                            }
                        }
                    } }
                } }
            }
        }
    }
}

