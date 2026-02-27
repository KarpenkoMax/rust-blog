#[cfg(target_arch = "wasm32")]
mod api;
#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod components;
#[cfg(target_arch = "wasm32")]
mod models;
#[cfg(target_arch = "wasm32")]
mod state;
#[cfg(target_arch = "wasm32")]
mod storage;

#[cfg(target_arch = "wasm32")]
fn main() {
    leptos::mount::mount_to_body(app::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Пустой main нужен только чтобы `cargo build --workspace` на хосте проходил.
}
