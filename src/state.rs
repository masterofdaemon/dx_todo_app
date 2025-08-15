use dioxus::prelude::*;

use crate::models::Project;

#[derive(Clone, Copy)]
pub struct AppState {
    pub projects: Signal<Vec<Project>>,
    pub active_project_id: Signal<Option<u64>>,
    pub new_title: Signal<String>,
    pub editing_id: Signal<Option<u64>>,
    pub editing_text: Signal<String>,
    pub next_id: Signal<u64>,
    pub filter: Signal<crate::models::Filter>,
}

#[derive(Clone)]
pub struct FlashState {
    pub msg: Signal<Option<String>>,
}
