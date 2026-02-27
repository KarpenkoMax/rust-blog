#[cfg(target_arch = "wasm32")]
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use crate::api;
#[cfg(target_arch = "wasm32")]
use crate::state::AppState;
#[cfg(target_arch = "wasm32")]
use crate::storage;

use crate::components::auth_panel::AuthPanel;
use crate::components::posts_panel::PostsPanel;

#[cfg(target_arch = "wasm32")]
fn load_posts(state: AppState, limit: u32, offset: u32) {
    state.loading.set(true);
    state.clear_error();

    leptos::task::spawn_local(async move {
        match api::list_posts(limit, offset).await {
            Ok(resp) => state.posts.set(resp.posts),
            Err(err) => state.set_error(err.to_string()),
        }
        state.loading.set(false);
    });
}

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();

    if let Some(token) = storage::load_token() {
        state.token.set(Some(token));
    }
    if let Some(user) = storage::load_user() {
        state.user.set(Some(user));
    }

    load_posts(state.clone(), 10, 0);

    let auth_text = {
        let state = state.clone();
        move || {
            if state.is_authenticated() {
                "yes".to_string()
            } else {
                "no".to_string()
            }
        }
    };

    let user_text = {
        let state = state.clone();
        move || {
            state
                .user
                .get()
                .map(|u| format!("{} ({})", u.username, u.email))
                .unwrap_or_else(|| "anonymous".to_string())
        }
    };

    let error_text = {
        let state = state.clone();
        move || state.error.get().unwrap_or_default()
    };

    let on_refresh = Callback::new({
        let state = state.clone();
        move |_| load_posts(state.clone(), 10, 0)
    });

    view! {
        <main class="page">
            <section class="container">
                <h1>"Rust Blog (Leptos)"</h1>
                <p>"Auth: " {auth_text}</p>
                <p>"Current user: " {user_text}</p>

                <AuthPanel state=state.clone() />

                <Show when=move || !state.error.get().unwrap_or_default().is_empty()>
                    <div class="error-banner">
                        <strong>"Ошибка: "</strong>
                        {error_text}
                    </div>
                </Show>
                
                <PostsPanel state=state.clone() on_refresh=on_refresh />

            </section>
        </main>
    }
}
