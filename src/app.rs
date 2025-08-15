use dioxus::events::Key;
use dioxus::prelude::*;
#[cfg(not(target_os = "android"))]
use dioxus_i18n::prelude::Locale;
#[cfg(not(target_os = "android"))]
use dioxus_i18n::prelude::*;
use dioxus_router::prelude::{Routable, Router, use_navigator};
use std::cmp::max;
#[cfg(not(target_os = "android"))]
use unic_langid::langid;

use crate::components::header::HeaderState;
use crate::components::projects::ProjectsState;
use crate::components::{add_form::AddForm, filter_bar::FilterBar, header::Header};
use crate::export::export_active_project_pdf;
use crate::models::{Filter, Project, Subtask, Todo};
use crate::state::{AppState, FlashState};
use crate::storage::{load_or_migrate_projects, save_projects};

// Use relative asset paths on Android (no leading slash), absolute on others
#[cfg(target_os = "android")]
pub const FAVICON: Asset = asset!("assets/favicon.ico");
#[cfg(not(target_os = "android"))]
pub const FAVICON: Asset = asset!("/assets/favicon.ico");

#[cfg(target_os = "android")]
pub const MAIN_CSS: Asset = asset!("assets/main.css");
#[cfg(not(target_os = "android"))]
pub const MAIN_CSS: Asset = asset!("/assets/main.css");

// Head nodes helper: on Android, skip document::* tags; on others, include meta, favicon, and stylesheet
#[cfg(target_os = "android")]
fn head_nodes() -> Element {
    const INLINE_CSS: &str = include_str!("../assets/main.css");
    rsx! { style { "{INLINE_CSS}" } }
}

#[cfg(not(target_os = "android"))]
fn head_nodes() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no" }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
    }
}

// Routing
#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    Projects {},
    #[route("/list")]
    List {},
    #[route("/todo/:id")]
    Details { id: u64 },
}

// Root component
#[component]
pub fn App() -> Element {
    let mut projects = use_signal(Vec::<Project>::new);
    let mut active_project_id = use_signal(|| Option::<u64>::None);
    let new_title = use_signal(String::new);
    let editing_id = use_signal(|| Option::<u64>::None);
    let editing_text = use_signal(String::new);
    let mut next_id = use_signal(|| 1u64);
    let filter = use_signal(|| Filter::All);

    use_context_provider(|| AppState {
        projects: projects.clone(),
        active_project_id: active_project_id.clone(),
        new_title: new_title.clone(),
        editing_id: editing_id.clone(),
        editing_text: editing_text.clone(),
        next_id: next_id.clone(),
        filter: filter.clone(),
    });
    use_context_provider(|| ProjectsState {
        projects: projects.clone(),
        active_project_id: active_project_id.clone(),
    });
    let active_project_snap = use_signal(|| Option::<Project>::None);
    use_context_provider(|| HeaderState {
        active_project: active_project_snap.clone(),
    });

    use_effect(move || {
        println!("[App] started");
    });

    use_effect(move || {
        let loaded = load_or_migrate_projects();
        println!("[App] Loaded {} project(s)", loaded.len());
        if !loaded.is_empty() {
            if active_project_id.read().is_none() {
                println!(
                    "[App] No active project set. Selecting first: id={} name={}",
                    loaded[0].id, loaded[0].name
                );
                active_project_id.set(Some(loaded[0].id));
            }
            let max_id_val = loaded
                .iter()
                .flat_map(|p| p.todos.iter())
                .fold(0u64, |acc, t| max(acc, t.id));
            next_id.set(max_id_val + 1);
        }
        projects.set(loaded);
    });

    {
        let projects = projects.clone();
        let active_project_id = active_project_id.clone();
        let mut active_project_snap = active_project_snap.clone();
        use_effect(move || {
            let opt = active_project_id
                .read()
                .and_then(|id| projects.read().iter().find(|p| p.id == id).cloned());
            if let Some(ref p) = opt {
                println!(
                    "[HeaderState] Active project snapshot updated: id={} name={}",
                    p.id, p.name
                );
            } else {
                println!("[HeaderState] Active project snapshot updated: None");
            }
            active_project_snap.set(opt);
        });
    }

    let flash = use_signal(|| None::<String>);
    use_context_provider(|| FlashState { msg: flash.clone() });

    {
        let flash = flash.clone();
        use_effect(move || {
            if flash.read().is_some() {
                let mut flash = flash.clone();
                dioxus::prelude::spawn(async move {
                    use std::time::Duration;
                    tokio::time::sleep(Duration::from_millis(2000)).await;
                    flash.set(None);
                });
            }
        });
    }

    #[cfg(not(target_os = "android"))]
    use_init_i18n(|| {
        I18nConfig::new(langid!("ru-RU"))
            .with_locale(Locale::new_static(
                langid!("en-US"),
                include_str!("./i18n/en-US.ftl"),
            ))
            .with_locale(Locale::new_static(
                langid!("ru-RU"),
                include_str!("./i18n/ru-RU.ftl"),
            ))
    });

    rsx! {
        { head_nodes() }
        { if let Some(m) = flash.read().clone() { rsx!{
            div {
                style: "position:fixed; top:12px; left:50%; transform:translateX(-50%); background:rgba(56,189,248,0.95); color:#0b1020; padding:10px 14px; border-radius:8px; box-shadow:0 8px 24px rgba(0,0,0,0.25); font-weight:600; z-index:9999;",
                {m}
            }
        } } else { rsx!{} } }
        { main_body() }
    }
}

#[cfg(target_os = "android")]
fn main_body() -> Element {
    rsx! {
        ErrorBoundary {
            handle_error: move |e| rsx!{ pre { style: "white-space:pre-wrap; padding:12px; color:#ef4444;", "{e:?}" } },
            Router::<Route> {}
        }
    }
}

#[cfg(not(target_os = "android"))]
fn main_body() -> Element {
    rsx! {
        ErrorBoundary {
            handle_error: move |e| rsx!{ pre { style: "white-space:pre-wrap; padding:12px; color:#ef4444;", "{e:?}" } },
            Router::<Route> {}
        }
    }
}

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

#[component]
pub fn Projects() -> Element {
    let _ = use_context::<ProjectsState>();
    rsx! { div { class: "app", crate::components::projects::Projects {} } }
}

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
