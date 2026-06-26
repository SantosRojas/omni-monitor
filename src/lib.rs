pub mod models;

#[cfg(feature = "ssr")]
pub mod api;
#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod database;

#[cfg(all(feature = "frontend", target_arch = "wasm32"))]
pub mod app;
#[cfg(all(feature = "frontend", target_arch = "wasm32"))]
pub mod components;
#[cfg(all(feature = "frontend", target_arch = "wasm32"))]
pub mod pages;
#[cfg(all(feature = "frontend", target_arch = "wasm32"))]
pub mod utils;

#[cfg(all(feature = "frontend", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::mount_to_body(|| leptos::view! { <crate::app::App /> });
}
