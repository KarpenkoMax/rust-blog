use leptos::prelude::*;

use crate::models::{Post, User};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) token: RwSignal<Option<String>>,
    pub(crate) user: RwSignal<Option<User>>,
    pub(crate) posts: RwSignal<Vec<Post>>,
    pub(crate) error: RwSignal<Option<String>>,
    pub(crate) loading: RwSignal<bool>,
}

impl AppState {
    pub(crate) fn new() -> Self {
        Self {
            token: RwSignal::new(None),
            user: RwSignal::new(None),
            posts: RwSignal::new(Vec::new()),
            error: RwSignal::new(None),
            loading: RwSignal::new(false),
        }
    }

    pub(crate) fn set_error(&self, message: impl Into<String>) {
        self.error.set(Some(message.into()));
    }

    pub(crate) fn clear_error(&self) {
        self.error.set(None);
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.token.get().is_some()
    }
}
