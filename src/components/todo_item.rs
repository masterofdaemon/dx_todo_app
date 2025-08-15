use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use crate::Route;

use crate::models::Todo;

#[component]
pub fn TodoItem(
    todo: Todo,
    is_editing: bool,
    editing_text: String,
    on_toggle: EventHandler<MouseEvent>,
    on_start_edit: EventHandler<MouseEvent>,
    on_remove: EventHandler<MouseEvent>,
    on_save_click: EventHandler<MouseEvent>,
    on_save_key: EventHandler<KeyboardEvent>,
    on_edit_input: EventHandler<FormEvent>,
    on_cancel: EventHandler<MouseEvent>,
    // Drag & drop reordering
    on_drag_start: EventHandler<u64>,
    on_drag_over: EventHandler<u64>,
    on_drag_leave: EventHandler<u64>,
    on_drag_end: EventHandler<u64>,
    on_drop: EventHandler<u64>,
    // Visual flags
    is_dragging: bool,
    is_drag_over: bool,
) -> Element {
    let nav = use_navigator();
    rsx! {
        li {
            ondragover: move |e: dioxus::events::DragEvent| { e.prevent_default(); on_drag_over.call(todo.id); },
            ondragleave: move |_| on_drag_leave.call(todo.id),
            ondrop: move |_| on_drop.call(todo.id),
            class: {
                let mut cls = if is_editing { "list-item editing".to_string() } else { "list-item".to_string() };
                if is_dragging { cls.push_str(" dragging"); }
                if is_drag_over { cls.push_str(" drag-over"); }
                cls
            },
            // complete toggle
            if !is_editing {
                div { class: "row between",
                    div { class: "left",
                        span { 
                            class: "drag-handle", 
                            draggable: "true", 
                            ondragstart: move |_| on_drag_start.call(todo.id),
                            ondragend: move |_| on_drag_end.call(todo.id),
                            svg { 
                                view_box: "0 0 24 24", 
                                fill: "currentColor",
                                circle { cx: "7", cy: "7", r: "1.5" }
                                circle { cx: "7", cy: "12", r: "1.5" }
                                circle { cx: "7", cy: "17", r: "1.5" }
                                circle { cx: "12", cy: "7", r: "1.5" }
                                circle { cx: "12", cy: "12", r: "1.5" }
                                circle { cx: "12", cy: "17", r: "1.5" }
                            }
                        }
                        input {
                            r#type: "checkbox",
                            checked: todo.completed,
                            onclick: move |e| on_toggle.call(e),
                        }
                    }
                }
            } else {
                div { class: "checkbox-spacer" }
            }

            // title or editor
            div { class: "content",
                if !is_editing {
                    span { class: if todo.completed { "item-title completed" } else { "item-title" }, "{todo.title}" }
                } else {
                    input {
                        class: "text edit",
                        r#type: "text",
                        value: "{editing_text}",
                        autofocus: "true",
                        oninput: move |e| on_edit_input.call(e),
                        onkeydown: move |e| on_save_key.call(e),
                    }
                }
            }

            // actions
            div { class: "actions",
                if !is_editing {
                    button { class: "btn btn-primary btn-icon", onclick: move |_| { nav.push(Route::Details { id: todo.id }); }, "🔍" }
                    button { class: "btn btn-edit btn-icon", onclick: move |e| on_start_edit.call(e), "✎" }
                    button { class: "btn btn-danger btn-icon", onclick: move |e| on_remove.call(e), "🗑" }
                } else {
                    button { class: "btn btn-success btn-icon", onclick: move |e| on_save_click.call(e), "💾" }
                    button { class: "btn btn-ghost btn-icon", onclick: move |e| on_cancel.call(e), "✖" }
                }
            }
        }
    }
}
