use dioxus::prelude::*;
#[cfg(not(target_os = "android"))]
use dioxus_i18n::prelude::Locale;
#[cfg(not(target_os = "android"))]
use dioxus_i18n::prelude::*;
use dioxus_router::prelude::Routable;
use dioxus_router::prelude::Router;
#[cfg(not(target_os = "android"))]
use unic_langid::langid;

use crate::components::header::HeaderState;
use crate::components::projects::ProjectsState;
use crate::models::Project;
use crate::state::{AppState, FlashState};
use crate::storage::load_or_migrate_projects;

// assets and head tag helper
use crate::app_assets::head_nodes;

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
    let filter = use_signal(|| crate::models::Filter::All);

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
                .fold(0u64, |acc, t| std::cmp::max(acc, t.id));
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

// Re-export pages
pub use crate::pages::{
    list::List,
    projects::Projects,
    details::Details,
};
