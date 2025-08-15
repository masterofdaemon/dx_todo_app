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
    let _ = use_context::<ProjectsState>();
    rsx! { div { class: "app", crate::components::projects::Projects {} } }
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
