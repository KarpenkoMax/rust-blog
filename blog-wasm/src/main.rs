#[cfg(target_arch = "wasm32")]
fn main() {
    use leptos::prelude::*;
    leptos::mount::mount_to_body(|| view! { <p>"Hello from Leptos CSR!"</p> });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Пустой main нужен только чтобы `cargo build --workspace` на хосте проходил.
}
