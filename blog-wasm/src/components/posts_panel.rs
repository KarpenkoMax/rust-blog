use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::models::Post;
use crate::state::AppState;

fn validate_non_empty_fields(title: &str, content: &str, error_message: &'static str) -> Result<(), &'static str> {
    if title.trim().is_empty() || content.trim().is_empty() {
        return Err(error_message);
    }
    Ok(())
}

fn find_post_for_edit(posts: &[Post], post_id: i64) -> Option<(String, String)> {
    posts
        .iter()
        .find(|post| post.id == post_id)
        .map(|post| (post.title.clone(), post.content.clone()))
}

#[component]
pub(crate) fn PostsPanel(
    state: AppState,
    on_refresh: Callback<()>,
) -> impl IntoView {
    let create_title = RwSignal::new(String::new());
    let create_content = RwSignal::new(String::new());

    let editing_post_id = RwSignal::new(None::<i64>);
    let edit_title = RwSignal::new(String::new());
    let edit_content = RwSignal::new(String::new());

    let on_create_post = Callback::new({
        let state = state.clone();
        move |ev: SubmitEvent| {
            ev.prevent_default();
            state.clear_error();

            let title = create_title.get().trim().to_string();
            let content = create_content.get().trim().to_string();

            if let Err(message) = validate_non_empty_fields(&title, &content, "Заполните title и content") {
                state.set_error(message);
                return;
            }

            let Some(token) = state.token.get() else {
                state.set_error("Нужна авторизация для создания поста");
                return;
            };

            state.loading.set(true);
            let state2 = state.clone();
            leptos::task::spawn_local(async move {
                match api::create_post(&token, &title, &content).await {
                    Ok(created) => {
                        state2.posts.update(|posts| posts.insert(0, created));
                        create_title.set(String::new());
                        create_content.set(String::new());
                        state2.clear_error();
                    }
                    Err(err) => state2.set_error(err.to_string()),
                }
                state2.loading.set(false);
            });
        }
    });

    let on_delete_post = Callback::new({
        let state = state.clone();
        move |post_id: i64| {
            state.clear_error();

            let Some(token) = state.token.get() else {
                state.set_error("Нужна авторизация для удаления поста");
                return;
            };

            state.loading.set(true);
            let state2 = state.clone();
            leptos::task::spawn_local(async move {
                match api::delete_post(&token, post_id).await {
                    Ok(()) => {
                        state2.posts.update(|posts| posts.retain(|p| p.id != post_id));
                        state2.clear_error();
                    }
                    Err(err) => state2.set_error(err.to_string()),
                }
                state2.loading.set(false);
            });
        }
    });

    let on_start_edit = Callback::new({
        let state = state.clone();
        move |post_id: i64| {
            let posts = state.posts.get();
            let Some((title, content)) = find_post_for_edit(&posts, post_id) else {
                state.set_error("Пост для редактирования не найден в текущем списке");
                return;
            };

            editing_post_id.set(Some(post_id));
            edit_title.set(title);
            edit_content.set(content);
        }
    });

    let on_cancel_edit = Callback::new(move |_| {
        editing_post_id.set(None);
        edit_title.set(String::new());
        edit_content.set(String::new());
    });

    let on_save_update = Callback::new({
        let state = state.clone();
        move |post_id: i64| {
            state.clear_error();

            let Some(token) = state.token.get() else {
                state.set_error("Нужна авторизация для обновления поста");
                return;
            };

            let title = edit_title.get().trim().to_string();
            let content = edit_content.get().trim().to_string();

            if let Err(message) =
                validate_non_empty_fields(&title, &content, "Заполните title и content для обновления")
            {
                state.set_error(message);
                return;
            }

            state.loading.set(true);
            let state2 = state.clone();
            spawn_local(async move {
                match api::update_post(&token, post_id, &title, &content).await {
                    Ok(updated) => {
                        state2.posts.update(|posts| {
                            if let Some(post) = posts.iter_mut().find(|p| p.id == post_id) {
                                *post = updated;
                            }
                        });
                        editing_post_id.set(None);
                        edit_title.set(String::new());
                        edit_content.set(String::new());
                        state2.clear_error();
                    }
                    Err(err) => state2.set_error(err.to_string()),
                }
                state2.loading.set(false);
            });
        }
    });

    let state_for_create_show = state.clone();
    let state_for_posts_each = state.clone();
    let state_for_post_actions_show = state.clone();

    view! {
        <h2>"Posts"</h2>
        <button on:click=move |_| on_refresh.run(()) disabled=move || state.loading.get()>
            "Refresh posts"
        </button>

        <p style="margin-top: 0.5rem;">
            "Всего в state: "
            {move || state.posts.get().len()}
        </p>

        <Show when=move || state_for_create_show.is_authenticated()>
            <h3 style="margin-top: 1rem;">"Create post"</h3>
            <form on:submit=move |ev| on_create_post.run(ev)>
                <input
                    placeholder="title"
                    prop:value=move || create_title.get()
                    on:input=move |ev| create_title.set(event_target_value(&ev))
                />
                <input
                    placeholder="content"
                    prop:value=move || create_content.get()
                    on:input=move |ev| create_content.set(event_target_value(&ev))
                />
                <button type="submit" disabled=move || state.loading.get()>
                    "Create"
                </button>
            </form>
        </Show>

        <ul>
            <For
                each=move || state_for_posts_each.posts.get()
                key=|post| (post.id, post.updated_at.clone())
                children=move |post| {
                    let state_for_post_actions_show = state_for_post_actions_show.clone();
                    let post_id = post.id;
                    let post_author_id = post.author_id;
                    let post_title = post.title.clone();
                    let post_content = post.content.clone();

                    let is_editing_this = {
                        let id = post_id;
                        move || editing_post_id.get() == Some(id)
                    };
                    view! {
                        <li style="margin-bottom: 0.5rem;">
                            <strong>{post_title.clone()}</strong>
                            <div>{post_content.clone()}</div>
                            <small>{format!("id={}, author_id={}", post_id, post_author_id)}</small>

                            <Show when=move || {
                                if !state_for_post_actions_show.is_authenticated() {
                                    return false;
                                }
                                let Some(user) = state_for_post_actions_show.user.get() else {
                                    return false;
                                };
                                user.id == post_author_id
                            }>
                                <div style="margin-top: 0.25rem;">
                                    <Show when=move || !is_editing_this()>
                                        <button
                                        on:click={
                                                let on_start_edit = on_start_edit.clone();
                                                let id = post_id;
                                                move |_| on_start_edit.run(id)
                                            }
                                            disabled=move || state.loading.get()
                                        >
                                            "Edit"
                                        </button>
                                    </Show>

                                    <Show when=move || is_editing_this()>
                                        <div style="margin-top: 0.5rem;">
                                            <input
                                                placeholder="new title"
                                                prop:value=move || edit_title.get()
                                                on:input=move |ev| edit_title.set(event_target_value(&ev))
                                            />
                                            <input
                                                placeholder="new content"
                                                prop:value=move || edit_content.get()
                                                on:input=move |ev| edit_content.set(event_target_value(&ev))
                                            />
                                            <button
                                                on:click={
                                                    let on_save_update = on_save_update.clone();
                                                    let id = post_id;
                                                    move |_| on_save_update.run(id)
                                                }
                                                disabled=move || state.loading.get()
                                            >
                                                "Save"
                                            </button>
                                            <button
                                                style="margin-left: 0.5rem;"
                                                on:click={
                                                    let on_cancel_edit = on_cancel_edit.clone();
                                                    move |_| on_cancel_edit.run(())
                                                }
                                                disabled=move || state.loading.get()
                                            >
                                                "Cancel"
                                            </button>
                                        </div>
                                    </Show>

                                    <button
                                        style="margin-left: 0.5rem;"
                                        on:click={
                                            let on_delete_post = on_delete_post.clone();
                                            let id = post_id;
                                            move |_| on_delete_post.run(id)
                                        }
                                        disabled=move || state.loading.get()
                                    >
                                        "Delete"
                                    </button>
                                </div>
                            </Show>
                        </li>
                    }
                }
            />
        </ul>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_post(id: i64, title: &str, content: &str) -> Post {
        Post {
            id,
            title: title.to_string(),
            content: content.to_string(),
            author_id: 1,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn validate_non_empty_fields_accepts_non_blank_values() {
        let result = validate_non_empty_fields("title", "content", "err");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_non_empty_fields_rejects_blank_values() {
        let result = validate_non_empty_fields("  ", "content", "err");
        assert_eq!(result, Err("err"));
    }

    #[test]
    fn find_post_for_edit_returns_title_and_content() {
        let posts = vec![sample_post(1, "A", "X"), sample_post(2, "B", "Y")];
        let result = find_post_for_edit(&posts, 2);
        assert_eq!(result, Some(("B".to_string(), "Y".to_string())));
    }

    #[test]
    fn find_post_for_edit_returns_none_for_missing_post() {
        let posts = vec![sample_post(1, "A", "X")];
        let result = find_post_for_edit(&posts, 999);
        assert!(result.is_none());
    }
}
