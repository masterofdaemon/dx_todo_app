#![cfg(target_os = "android")]
use dioxusmain::App;

fn main() {
    // Launch the Dioxus app on Android using the library entry point.
    dioxus_mobile::launch(App);
}

