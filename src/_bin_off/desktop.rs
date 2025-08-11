use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus::events::Key;
use std::cmp::max;
use printpdf::{PdfDocument, PdfDocumentReference, Mm, BuiltinFont};
#[cfg(not(target_os = "android"))]
use rfd::FileDialog;

mod models;
mod storage;
mod components;
use models::{Filter, Todo, Subtask, Project};
use storage::{load_or_migrate_projects, save_projects};
use components::{
    header::Header,
    add_form::AddForm,
    filter_bar::FilterBar,
    todo_item::TodoItem,
};
use components::projects::ProjectsState;
use components::header::HeaderState;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[cfg(target_os = "android")]
fn main() { dioxus_mobile::launch(App); }

#[cfg(not(target_os = "android"))]
fn main() { dioxus::launch(App); }

// Android: export without a file dialog – write into app temp directory
#[cfg(target_os = "android")]
fn export_active_project_pdf(projects: &Vec<Project>, active_id: Option<u64>) -> Result<(), String> {
    let active_id = active_id.ok_or_else(|| "No active project selected".to_string())?;
    let project = projects.iter().find(|p| p.id == active_id).ok_or_else(|| "Active project not found".to_string())?;

    let (doc, page1, layer1) = PdfDocument::new(&format!("Project: {}", project.name), Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| format!("font error: {e}"))?;

    let mut current_page = page1;
    let mut current_layer = doc.get_page(current_page).get_layer(layer1);
    let margin_left = Mm(15.0);
    let margin_top = Mm(15.0);
    let line_height = Mm(6.0);
    let mut cursor_y = Mm(297.0) - margin_top;

    let mut write_line = |doc: &PdfDocumentReference, text: &str, size_pt: f64| {
        if cursor_y.0 < 20.0 {
            let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            current_page = p;
            current_layer = doc.get_page(current_page).get_layer(l);
            cursor_y = Mm(297.0) - margin_top;
        }
        current_layer.use_text(text, size_pt, margin_left, cursor_y, &font);
        cursor_y = Mm(cursor_y.0 - line_height.0);
    };

    write_line(&doc, &format!("Project: {}", project.name), 16.0);
    write_line(&doc, "", 10.0);
    for t in &project.todos {
        let mark = if t.completed { "[x]" } else { "[ ]" };
        write_line(&doc, &format!("{} {}", mark, t.title), 12.0);
        for s in &t.subtasks {
            let mark = if s.completed { "[x]" } else { "[ ]" };
            write_line(&doc, &format!("    {} {}", mark, s.title), 11.0);
        }
        if !t.description.trim().is_empty() {
            write_line(&doc, &format!("    — {}", t.description.trim()), 10.0);
        }
    }

    use std::fs::File;
    use std::io::BufWriter;
    let mut out_path = std::env::temp_dir();
    let ts = chrono::Utc::now().timestamp();
    let fname = format!("{}_{}.pdf", project.name.replace('/', "-"), ts);
    out_path.push(fname);
    let mut out = BufWriter::new(File::create(&out_path).map_err(|e| format!("create error: {e}"))?);
    doc.save(&mut out).map_err(|e| format!("save error: {e}"))?;
    println!("[Export][Android] Saved to {}", out_path.display());
    Ok(())
}

// Export active project to a simple PDF using printpdf's built-in Helvetica font (Desktop)
#[cfg(not(target_os = "android"))]
fn export_active_project_pdf(projects: &Vec<Project>, active_id: Option<u64>) -> Result<(), String> {
    let active_id = active_id.ok_or_else(|| "No active project selected".to_string())?;
    let project = projects.iter().find(|p| p.id == active_id).ok_or_else(|| "Active project not found".to_string())?;

    // Ask for save path
    let Some(path) = FileDialog::new()
        .set_title("Export Project to PDF")
        .set_file_name(&format!("{}.pdf", project.name))
        .save_file() else {
        return Err("Save canceled".into());
    };

    // Document A4
    let (doc, page1, layer1) = PdfDocument::new(&format!("Project: {}", project.name), Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| format!("font error: {e}"))?;

    // Page layout
    let mut current_page = page1;
    let mut current_layer = doc.get_page(current_page).get_layer(layer1);
    let margin_left = Mm(15.0);
    let margin_top = Mm(15.0);
    let line_height = Mm(6.0);
    let mut cursor_y = Mm(297.0) - margin_top;

    // Helper to write line and handle pagination
    let mut write_line = |doc: &PdfDocumentReference, text: &str, size_pt: f64| {
        if cursor_y.0 < 20.0 { // new page if near bottom
            let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            current_page = p;
            current_layer = doc.get_page(current_page).get_layer(l);
            cursor_y = Mm(297.0) - margin_top;
        }
        current_layer.use_text(text, size_pt, margin_left, cursor_y, &font);
        cursor_y = Mm(cursor_y.0 - line_height.0);
    };

    // Header
    write_line(&doc, &format!("Project: {}", project.name), 16.0);
    write_line(&doc, "", 10.0);

    // Tasks
    for t in &project.todos {
        let mark = if t.completed { "[x]" } else { "[ ]" };
        write_line(&doc, &format!("{} {}", mark, t.title), 12.0);
        // Subtasks
        for s in &t.subtasks {
            let mark = if s.completed { "[x]" } else { "[ ]" };
            // indent by adding spaces
            write_line(&doc, &format!("    {} {}", mark, s.title), 11.0);
        }
        if !t.description.trim().is_empty() {
            write_line(&doc, &format!("    — {}", t.description.trim()), 10.0);
        }
    }

    use std::fs::File;
    use std::io::BufWriter;
    let mut out = BufWriter::new(File::create(&path).map_err(|e| format!("create error: {e}"))?);
    doc.save(&mut out).map_err(|e| format!("save error: {e}"))?;
    Ok(())
}
// Types and persistence are defined in `models.rs` and `storage.rs`.

#[derive(Clone, Copy)]
struct AppState {
    projects: Signal<Vec<Project>>,
    active_project_id: Signal<Option<u64>>,
    new_title: Signal<String>,
    editing_id: Signal<Option<u64>>,
    editing_text: Signal<String>,
    next_id: Signal<u64>,
    filter: Signal<Filter>,
}

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")] Projects {},
    #[route("/list")] List {},
    #[route("/todo/:id")] Details { id: u64 },
}

#[component]
fn App() -> Element {
    // State
    let mut projects = use_signal(Vec::<Project>::new);
    let mut active_project_id = use_signal(|| Option::<u64>::None);
    let new_title = use_signal(String::new);
    let editing_id = use_signal(|| Option::<u64>::None);
    let editing_text = use_signal(String::new);
    let mut next_id = use_signal(|| 1u64);
    let filter = use_signal(|| Filter::All);

    // Provide context for screens
    use_context_provider(|| AppState {
        projects: projects.clone(),
        active_project_id: active_project_id.clone(),
        new_title: new_title.clone(),
        editing_id: editing_id.clone(),
        editing_text: editing_text.clone(),
        next_id: next_id.clone(),
        filter: filter.clone(),
    });
    // Provide Projects and Header contexts
    use_context_provider(|| ProjectsState { projects: projects.clone(), active_project_id: active_project_id.clone() });
    let active_project_snap = use_signal(|| Option::<Project>::None);
    use_context_provider(|| HeaderState { active_project: active_project_snap.clone() });

    // One-time load from disk after first render
    use_effect(move || {
        let loaded = load_or_migrate_projects();
        println!("[App] Loaded {} project(s)", loaded.len());
        if !loaded.is_empty() {
            // choose first project by default if not selected
            if active_project_id.read().is_none() {
                println!("[App] No active project set. Selecting first: id={} name={} ", loaded[0].id, loaded[0].name);
                active_project_id.set(Some(loaded[0].id));
            }
            // compute next id across all todos
            let max_id_val = loaded
                .iter()
                .flat_map(|p| p.todos.iter())
                .fold(0u64, |acc, t| max(acc, t.id));
            next_id.set(max_id_val + 1);
        }
        projects.set(loaded);
    });
    // keep active project snapshot updated for header
    {
        let projects = projects.clone();
        let mut active_project_id = active_project_id.clone();
        let mut active_project_snap = active_project_snap.clone();
        use_effect(move || {
            let opt = active_project_id.read().and_then(|id| projects.read().iter().find(|p| p.id == id).cloned());
            if let Some(ref p) = opt { println!("[HeaderState] Active project snapshot updated: id={} name={}", p.id, p.name); } else { println!("[HeaderState] Active project snapshot updated: None"); }
            active_project_snap.set(opt);
        });
    }
    rsx! {
        // Inject global assets once so all routes (including Projects) are styled on first load
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

// Home list screen
#[component]
fn List() -> Element {
    let state = use_context::<AppState>();
    let mut projects = state.projects;
    let mut active_project_id = state.active_project_id;
    let mut new_title = state.new_title;
    let mut editing_id = state.editing_id;
    let mut editing_text = state.editing_text;
    let mut next_id = state.next_id;
    let mut filter = state.filter;
    let nav = use_navigator();

    // Guard: require active project
    let active_id_opt = *active_project_id.read();
    let active_id = match active_id_opt { Some(id) => id, None => { println!("[List] No active project. Redirecting to Projects."); nav.push(Route::Projects {}); return rsx!{ div { class: "app", div { class: "card", "Select a project" } } }; } };

    // Add todo
    let mut on_add = move |title: String| {
        if title.trim().is_empty() { return; }
        let id = *next_id.read();
        next_id.set(id + 1);
        if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
            p.todos.push(Todo { id, title, completed: false, subtasks: Vec::new(), description: String::new() });
        }
        save_projects(&projects.read());
    };

    // Item handlers
    let mut toggle = move |id: u64| {
        if let Some(t) = projects.write().iter_mut().find(|p| p.id == active_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) {
            let target = !t.completed;
            t.completed = target;
            // propagate to subtasks
            for s in &mut t.subtasks { s.completed = target; }
        }
        save_projects(&projects.read());
    };
    let mut start_edit = move |id: u64, text: String| { editing_id.set(Some(id)); editing_text.set(text); };
    let mut cancel_edit = move || { editing_id.set(None); editing_text.set(String::new()); };
    let mut save_edit = move |id: u64| { let text = editing_text.read().clone(); if let Some(t) = projects.write().iter_mut().find(|p| p.id == active_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) { t.title = text.clone(); } save_projects(&projects.read()); editing_id.set(None); editing_text.set(String::new()); };
    let mut remove_item = move |id: u64| { if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) { p.todos.retain(|t| t.id != id); } save_projects(&projects.read()); };
    let mut clear_completed = move || { if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) { p.todos.retain(|t| !t.completed); } save_projects(&projects.read()); };
    let mut confirming_clear = use_signal(|| false);

    // Drag & drop reordering state and handlers
    let mut dragging_from = use_signal(|| Option::<u64>::None);
    let mut drag_over = use_signal(|| Option::<u64>::None);
    let mut on_drag_start_item = move |id: u64| { dragging_from.set(Some(id)); };
    let mut on_drag_over_item = move |id: u64| { drag_over.set(Some(id)); };
    let mut on_drag_leave_item = move |id: u64| {
        if drag_over.read().as_ref() == Some(id).as_ref() { drag_over.set(None); }
    };
    let mut on_drag_end_item = move |_id: u64| {
        dragging_from.set(None);
        drag_over.set(None);
    };
    let mut on_drop_on_item = move |target_id: u64| {
        let src_opt = *dragging_from.read();
        if let Some(src_id) = src_opt {
            if src_id == target_id { dragging_from.set(None); return; }
            if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                if let (Some(src_idx), Some(dst_idx)) = (
                    p.todos.iter().position(|t| t.id == src_id),
                    p.todos.iter().position(|t| t.id == target_id),
                ) {
                    let item = p.todos.remove(src_idx);
                    let insert_idx = if src_idx < dst_idx { dst_idx - 1 } else { dst_idx };
                    p.todos.insert(insert_idx, item);
                }
            }
            dragging_from.set(None);
            drag_over.set(None);
            save_projects(&projects.read());
        }
    };

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { class: "app",
            div { class: "card",
                // header actions
                Header { 
                    count: projects.read().iter().find(|p| p.id == active_id).map(|p| p.todos.len()).unwrap_or(0),
                    on_switch: move |_| { println!("[Header] Switch clicked"); nav.push(Route::Projects {}); },
                    on_export: move |_| {
                        println!("[Header] Export clicked");
                        let res = export_active_project_pdf(&projects.read(), *active_project_id.read());
                        match res {
                            Ok(()) => println!("[Export] Success"),
                            Err(e) => println!("[Export] Error: {}", e),
                        }
                    }
                }
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
                    {
                        let items: Vec<Todo> = projects.read().iter().find(|p| p.id == active_id).map(|p| p.todos.clone()).unwrap_or_else(|| Vec::new());
                        rsx! {
                            for t in items.into_iter().filter(|t| match *filter.read() { Filter::All => true, Filter::Active => !t.completed, Filter::Completed => t.completed }) {
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
                            on_drag_leave: move |id| on_drag_leave_item(id),
                            on_drag_end: move |id| on_drag_end_item(id),
                            on_drop: move |id| on_drop_on_item(id),
                            is_dragging: dragging_from.read().as_ref() == Some(t.id).as_ref(),
                            is_drag_over: drag_over.read().as_ref() == Some(t.id).as_ref(),
                        }
                            }
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

// Projects screen renders list/create projects and sets active_project_id
#[component]
fn Projects() -> Element {
    let state = use_context::<AppState>();
    let _projects = state.projects;
    let _active_project_id = state.active_project_id;

    rsx! { components::projects::Projects {} }
}

// Details screen
#[component]
fn Details(id: u64) -> Element {
    let state = use_context::<AppState>();
    let mut projects = state.projects;
    let active_project_id = state.active_project_id;
    let nav = use_navigator();

    // Guard: require active project
    let active_id = match *active_project_id.read() { Some(id) => id, None => { nav.push(Route::Projects {}); return rsx!{ div { class: "app", div { class: "card", "Select a project" } } }; } };

    let todo_opt = projects.read().iter().find(|p| p.id == active_id).and_then(|p| p.todos.iter().cloned().find(|t| t.id == id));
    let Some(todo) = todo_opt else { return rsx!{ div { class: "app", div { class: "card", "Not found" } } }; };

    // Subtasks handlers
    let mut add_sub = move |title: String| {
        if title.trim().is_empty() { return; }
        if let Some(it) = projects.write().iter_mut().find(|p| p.id == active_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) {
            let next_sid = it.subtasks.iter().map(|s| s.id).max().unwrap_or(0) + 1;
            it.subtasks.push(Subtask { id: next_sid, title, completed: false });
            // New subtask means parent can't be completed
            it.completed = false;
        }
        save_projects(&projects.read());
    };
    let mut toggle_sub = move |sid: u64| {
        if let Some(it) = projects.write().iter_mut().find(|p| p.id == active_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) {
            if let Some(st) = it.subtasks.iter_mut().find(|s| s.id == sid) {
                st.completed = !st.completed;
            }
            // If there is at least one subtask and all are completed, mark parent done
            if !it.subtasks.is_empty() {
                it.completed = it.subtasks.iter().all(|s| s.completed);
            }
        }
        save_projects(&projects.read());
    };
    let mut remove_sub = move |sid: u64| {
        if let Some(it) = projects.write().iter_mut().find(|p| p.id == active_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) {
            it.subtasks.retain(|s| s.id != sid);
            if !it.subtasks.is_empty() {
                it.completed = it.subtasks.iter().all(|s| s.completed);
            } else {
                // No subtasks: do not auto-complete; leave as-is
            }
        }
        save_projects(&projects.read());
    };
    let mut update_desc = move |v: String| { if let Some(it) = projects.write().iter_mut().find(|p| p.id == active_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) { it.description = v; } save_projects(&projects.read()); };

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
