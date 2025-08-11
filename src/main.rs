use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus::events::Key;
use std::cmp::max;

mod models;
mod storage;
mod components;
use models::{Filter, Todo, Subtask};
use storage::{load_todos, save_todos};
use components::{
    header::Header,
    add_form::AddForm,
    filter_bar::FilterBar,
    todo_item::TodoItem,
};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() { dioxus::launch(App); }

// Types and persistence are defined in `models.rs` and `storage.rs`.

#[derive(Clone, Copy)]
struct AppState {
    todos: Signal<Vec<Todo>>,
    new_title: Signal<String>,
    editing_id: Signal<Option<u64>>,
    editing_text: Signal<String>,
    next_id: Signal<u64>,
    filter: Signal<Filter>,
}

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")] List {},
    #[route("/todo/:id")] Details { id: u64 },
}

#[component]
fn App() -> Element {
    // State
    let mut todos = use_signal(Vec::<Todo>::new);
    let new_title = use_signal(String::new);
    let editing_id = use_signal(|| Option::<u64>::None);
    let editing_text = use_signal(String::new);
    let mut next_id = use_signal(|| 1u64);
    let filter = use_signal(|| Filter::All);

    // Provide context for screens
    use_context_provider(|| AppState {
        todos: todos.clone(),
        new_title: new_title.clone(),
        editing_id: editing_id.clone(),
        editing_text: editing_text.clone(),
        next_id: next_id.clone(),
        filter: filter.clone(),
    });

    // One-time load from disk after first render
    use_effect(move || {
        let list = load_todos();
        if !list.is_empty() {
            let max_id_val = list.iter().fold(0u64, |acc, t| max(acc, t.id));
            next_id.set(max_id_val + 1);
            todos.set(list);
        }
    });
    rsx! { Router::<Route> {} }
}

// Home list screen
#[component]
fn List() -> Element {
    let state = use_context::<AppState>();
    let mut todos = state.todos;
    let mut new_title = state.new_title;
    let mut editing_id = state.editing_id;
    let mut editing_text = state.editing_text;
    let mut next_id = state.next_id;
    let mut filter = state.filter;

    // Add todo
    let mut on_add = move |title: String| {
        if title.trim().is_empty() { return; }
        let id = *next_id.read();
        next_id.set(id + 1);
        todos.write().push(Todo { id, title, completed: false, subtasks: Vec::new(), description: String::new() });
        save_todos(&todos.read());
    };

    // Item handlers
    let mut toggle = move |id: u64| {
        if let Some(t) = todos.write().iter_mut().find(|t| t.id == id) {
            let target = !t.completed;
            t.completed = target;
            // propagate to subtasks
            for s in &mut t.subtasks { s.completed = target; }
        }
        save_todos(&todos.read());
    };
    let mut start_edit = move |id: u64, text: String| { editing_id.set(Some(id)); editing_text.set(text); };
    let mut cancel_edit = move || { editing_id.set(None); editing_text.set(String::new()); };
    let mut save_edit = move |id: u64| { let text = editing_text.read().clone(); if let Some(t) = todos.write().iter_mut().find(|t| t.id == id) { t.title = text.clone(); } save_todos(&todos.read()); editing_id.set(None); editing_text.set(String::new()); };
    let mut remove_item = move |id: u64| { todos.write().retain(|t| t.id != id); save_todos(&todos.read()); };
    let mut clear_completed = move || { todos.write().retain(|t| !t.completed); save_todos(&todos.read()); };
    let mut confirming_clear = use_signal(|| false);

    // Drag & drop reordering state and handlers
    let mut dragging_from = use_signal(|| Option::<u64>::None);
    let mut on_drag_start_item = move |id: u64| { dragging_from.set(Some(id)); };
    let mut on_drag_over_item = move |_id: u64| { /* no-op: needed to allow drop */ };
    let mut on_drop_on_item = move |target_id: u64| {
        let src_opt = *dragging_from.read();
        if let Some(src_id) = src_opt {
            if src_id == target_id { dragging_from.set(None); return; }
            let mut vec = todos.write();
            if let (Some(src_idx), Some(dst_idx)) = (
                vec.iter().position(|t| t.id == src_id),
                vec.iter().position(|t| t.id == target_id),
            ) {
                let item = vec.remove(src_idx);
                let insert_idx = if src_idx < dst_idx { dst_idx - 1 } else { dst_idx };
                vec.insert(insert_idx, item);
            }
            drop(vec);
            dragging_from.set(None);
            save_todos(&todos.read());
        }
    };

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { class: "app",
            div { class: "card",
                Header { count: todos.read().len() }
                AddForm { value: new_title.read().clone(),
                    on_input: move |e: dioxus::events::FormEvent| new_title.set(e.value()),
                    on_enter: move |e: dioxus::events::KeyboardEvent| if e.key() == Key::Enter {
                        let title = new_title.read().clone();
                        on_add(title);
                        new_title.set(String::new());
                    },
                    on_add: move |_| {
                        let title = new_title.read().clone();
                        on_add(title);
                        new_title.set(String::new());
                    }
                }
                FilterBar { active: *filter.read(), on_all: move |_| filter.set(Filter::All), on_active: move |_| filter.set(Filter::Active), on_completed: move |_| filter.set(Filter::Completed), on_clear_completed: move |_| confirming_clear.set(true) }
                ul { class: "list",
                    for t in todos.read().iter().cloned().filter(|t| match *filter.read() { Filter::All => true, Filter::Active => !t.completed, Filter::Completed => t.completed }) {
                        TodoItem {
                            todo: t.clone(),
                            is_editing: editing_id.read().as_ref().is_some_and(|eid| *eid == t.id),
                            editing_text: editing_text.read().clone(),
                            on_toggle: move |_| toggle(t.id),
                            on_start_edit: move |_| start_edit(t.id, t.title.clone()),
                            on_remove: move |_| remove_item(t.id),
                            on_save_click: move |_| save_edit(t.id),
                            on_save_key: move |e: dioxus::events::KeyboardEvent| { if e.key() == Key::Enter { save_edit(t.id); } },
                            on_edit_input: move |e: dioxus::events::FormEvent| editing_text.set(e.value()),
                            on_cancel: move |_| cancel_edit(),
                            on_drag_start: move |id| on_drag_start_item(id),
                            on_drag_over: move |id| on_drag_over_item(id),
                            on_drop: move |id| on_drop_on_item(id),
                        }
                    }
                }
                if *confirming_clear.read() {
                    // modal overlay captures keys; Escape closes
                    div { class: "modal-overlay", tabindex: 0, onkeydown: move |e: dioxus::events::KeyboardEvent| if e.key() == Key::Escape { confirming_clear.set(false) },
                        div { class: "modal",
                            h3 { class: "title", "Clear completed tasks?" }
                            p { class: "meta", "This cannot be undone." }
                            div { class: "actions",
                                button { class: "btn btn-danger", autofocus: "true", onclick: move |_| { clear_completed(); confirming_clear.set(false); }, "Confirm" }
                                button { class: "btn btn-ghost", onclick: move |_| { confirming_clear.set(false); }, "Cancel" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Details screen
#[component]
fn Details(id: u64) -> Element {
    let state = use_context::<AppState>();
    let mut todos = state.todos;
    let nav = use_navigator();

    let todo_opt = todos.read().iter().cloned().find(|t| t.id == id);
    let Some(todo) = todo_opt else { return rsx!{ div { class: "app", div { class: "card", "Not found" } } }; };

    // Subtasks handlers
    let mut add_sub = move |title: String| {
        if title.trim().is_empty() { return; }
        if let Some(it) = todos.write().iter_mut().find(|t| t.id == id) {
            let next_sid = it.subtasks.iter().map(|s| s.id).max().unwrap_or(0) + 1;
            it.subtasks.push(Subtask { id: next_sid, title, completed: false });
            // New subtask means parent can't be completed
            it.completed = false;
        }
        save_todos(&todos.read());
    };
    let mut toggle_sub = move |sid: u64| {
        if let Some(it) = todos.write().iter_mut().find(|t| t.id == id) {
            if let Some(st) = it.subtasks.iter_mut().find(|s| s.id == sid) {
                st.completed = !st.completed;
            }
            // If there is at least one subtask and all are completed, mark parent done
            if !it.subtasks.is_empty() {
                it.completed = it.subtasks.iter().all(|s| s.completed);
            }
        }
        save_todos(&todos.read());
    };
    let mut remove_sub = move |sid: u64| {
        if let Some(it) = todos.write().iter_mut().find(|t| t.id == id) {
            it.subtasks.retain(|s| s.id != sid);
            if !it.subtasks.is_empty() {
                it.completed = it.subtasks.iter().all(|s| s.completed);
            } else {
                // No subtasks: do not auto-complete; leave as-is
            }
        }
        save_todos(&todos.read());
    };
    let mut update_desc = move |v: String| { if let Some(it) = todos.write().iter_mut().find(|t| t.id == id) { it.description = v; } save_todos(&todos.read()); };

    let mut sub_input = use_signal(String::new);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { class: "app",
            div { class: "card",
                div { class: "row between", style: "margin-bottom:12px;",
                    button { class: "btn btn-ghost", onclick: move |_| { nav.push(Route::List {}); }, "← Back" }
                }
                h2 { class: "title", "{todo.title}" }
                textarea { class: "text desc", rows: "3", placeholder: "Description...", value: "{todo.description}", oninput: move |e| update_desc(e.value()) }
                h3 { style: "margin-top:16px;", "Subtasks" }
                ul { class: "subtasks",
                    for st in todo.subtasks.clone().into_iter() {
                        li { key: "sub-{st.id}", class: "sub-item",
                            input { r#type: "checkbox", checked: st.completed, onclick: move |_| toggle_sub(st.id) }
                            span { class: if st.completed { "sub-title completed" } else { "sub-title" }, "{st.title}" }
                            button { class: "btn btn-ghost sub-remove", onclick: move |_| remove_sub(st.id), "✕" }
                        }
                    }
                    li { class: "sub-add",
                        input { class: "text sub-input", r#type: "text", placeholder: "Add a subtask…", value: "{sub_input.read()}", oninput: move |e| sub_input.set(e.value()), onkeydown: move |e| { if e.key() == Key::Enter { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } } } }
                        button { class: "btn btn-primary sub-add-btn", onclick: move |_| { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } }, "Add" }
                    }
                }
            }
        }
    }
}
