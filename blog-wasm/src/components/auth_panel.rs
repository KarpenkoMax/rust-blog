use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::state::AppState;
use crate::storage;

#[component]
pub(crate) fn AuthPanel(state: AppState) -> impl IntoView {
    let reg_username = RwSignal::new(String::new());
    let reg_email = RwSignal::new(String::new());
    let reg_password = RwSignal::new(String::new());

    let login_username = RwSignal::new(String::new());
    let login_password = RwSignal::new(String::new());

    let on_register = {
        let state = state.clone();
        move |ev: SubmitEvent| {
            ev.prevent_default();
            state.clear_error();

            let username = reg_username.get().trim().to_string();
            let email = reg_email.get().trim().to_string();
            let password = reg_password.get().trim().to_string();

            if username.is_empty() || email.is_empty() || password.is_empty() {
                state.set_error("Заполните все поля регистрации");
                return;
            }

            state.loading.set(true);
            let state2 = state.clone();
            spawn_local(async move {
                match api::register(&username, &email, &password).await {
                    Ok(auth) => {
                        if let Err(err) = storage::save_token(&auth.access_token) {
                            state2.set_error(err);
                        } else if let Err(err) = storage::save_user(&auth.user) {
                            state2.set_error(err);
                        } else {
                            state2.token.set(Some(auth.access_token));
                            state2.user.set(Some(auth.user));
                            state2.clear_error();
                        }
                    }
                    Err(err) => state2.set_error(err.to_string()),
                }
                state2.loading.set(false);
            });
        }
    };

    let on_login = {
        let state = state.clone();
        move |ev: SubmitEvent| {
            ev.prevent_default();
            state.clear_error();

            let username = login_username.get().trim().to_string();
            let password = login_password.get().trim().to_string();

            if username.is_empty() || password.is_empty() {
                state.set_error("Заполните все поля входа");
                return;
            }

            state.loading.set(true);
            let state2 = state.clone();
            spawn_local(async move {
                match api::login(&username, &password).await {
                    Ok(auth) => {
                        if let Err(err) = storage::save_token(&auth.access_token) {
                            state2.set_error(err);
                        } else if let Err(err) = storage::save_user(&auth.user) {
                            state2.set_error(err);
                        } else {
                            state2.token.set(Some(auth.access_token));
                            state2.user.set(Some(auth.user));
                            state2.clear_error();
                        }
                    }
                    Err(err) => state2.set_error(err.to_string()),
                }
                state2.loading.set(false);
            });
        }
    };

    let on_logout = {
        let state = state.clone();
        move |_| {
            if let Err(err) = storage::clear_token() {
                state.set_error(err);
                return;
            }
            if let Err(err) = storage::clear_user() {
                state.set_error(err);
                return;
            }
            state.token.set(None);
            state.user.set(None);
            state.clear_error();
        }
    };

    view! {
        <button on:click=on_logout disabled=move || state.loading.get()>
            "Logout"
        </button>
        <h2>"Register"</h2>
        <form on:submit=on_register>
            <input
                placeholder="username"
                on:input=move |ev| reg_username.set(event_target_value(&ev))
            />
            <input
                placeholder="email"
                on:input=move |ev| reg_email.set(event_target_value(&ev))
            />
            <input
                placeholder="password"
                type="password"
                on:input=move |ev| reg_password.set(event_target_value(&ev))
            />
            <button type="submit" disabled=move || state.loading.get()>"Register"</button>
        </form>

        <h2 style="margin-top: 1rem;">"Login"</h2>
        <form on:submit=on_login>
            <input
                placeholder="username"
                on:input=move |ev| login_username.set(event_target_value(&ev))
            />
            <input
                placeholder="password"
                type="password"
                on:input=move |ev| login_password.set(event_target_value(&ev))
            />
            <button type="submit" disabled=move || state.loading.get()>"Login"</button>
        </form>

        <hr style="margin: 1rem 0;" />
    }
}
