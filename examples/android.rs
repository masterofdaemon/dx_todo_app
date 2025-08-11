#[cfg(target_os = "android")]
fn main() {
    // Launch the Dioxus app via dioxus-mobile when built for Android
    // The library crate name is `dioxusmain` (see Cargo.toml [lib] name)
    dioxus_mobile::launch(dioxusmain::App);
}

#[cfg(not(target_os = "android"))]
fn main() {
    // This example is only meaningful for Android targets.
    // `dx serve --platform android --example android` will cross-compile it.
    println!("Android example: build with --platform android");
}
