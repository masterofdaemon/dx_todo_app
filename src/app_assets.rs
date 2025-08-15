use dioxus::prelude::*;

// Use relative asset paths on Android (no leading slash), absolute on others
#[cfg(target_os = "android")]
pub const FAVICON: Asset = asset!("assets/favicon.ico");
#[cfg(not(target_os = "android"))]
pub const FAVICON: Asset = asset!("/assets/favicon.ico");

#[cfg(target_os = "android")]
pub const MAIN_CSS: Asset = asset!("assets/main.css");
#[cfg(not(target_os = "android"))]
pub const MAIN_CSS: Asset = asset!("/assets/main.css");

// Head nodes helper: on Android, inline the CSS; on others, include meta, favicon, and stylesheet
#[cfg(target_os = "android")]
pub fn head_nodes() -> Element {
    const INLINE_CSS: &str = include_str!("../assets/main.css");
    rsx! { style { "{INLINE_CSS}" } }
}

#[cfg(not(target_os = "android"))]
pub fn head_nodes() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no" }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
    }
}

