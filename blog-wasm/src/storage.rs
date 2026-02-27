use crate::models::User;

const TOKEN_KEY: &str = "blog_token";
const USER_KEY: &str = "blog_user";

fn parse_token(raw: &str) -> Option<String> {
    let token = raw.trim().to_string();
    if token.is_empty() {
        return None;
    }
    Some(token)
}

fn parse_user(raw: &str) -> Option<User> {
    serde_json::from_str::<User>(raw).ok()
}

pub(crate) fn load_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let raw = storage.get_item(TOKEN_KEY).ok()??;
    parse_token(&raw)
}

pub(crate) fn save_token(token: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
    let storage = window
        .local_storage()
        .map_err(|_| "failed to access localStorage".to_string())?
        .ok_or_else(|| "localStorage is not available".to_string())?;

    storage
        .set_item(TOKEN_KEY, token)
        .map_err(|_| "failed to save token".to_string())
}

pub(crate) fn clear_token() -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
    let storage = window
        .local_storage()
        .map_err(|_| "failed to access localStorage".to_string())?
        .ok_or_else(|| "localStorage is not available".to_string())?;

    storage
        .remove_item(TOKEN_KEY)
        .map_err(|_| "failed to clear token".to_string())
}

pub(crate) fn load_user() -> Option<User> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let raw = storage.get_item(USER_KEY).ok()??;
    parse_user(&raw)
}

pub(crate) fn save_user(user: &User) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
    let storage = window
        .local_storage()
        .map_err(|_| "failed to access localStorage".to_string())?
        .ok_or_else(|| "localStorage is not available".to_string())?;

    let raw = serde_json::to_string(user).map_err(|_| "failed to serialize user".to_string())?;
    storage
        .set_item(USER_KEY, &raw)
        .map_err(|_| "failed to save user".to_string())
}

pub(crate) fn clear_user() -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
    let storage = window
        .local_storage()
        .map_err(|_| "failed to access localStorage".to_string())?
        .ok_or_else(|| "localStorage is not available".to_string())?;

    storage
        .remove_item(USER_KEY)
        .map_err(|_| "failed to clear user".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token_trims_and_returns_value() {
        let token = parse_token("  abc.def.ghi  ");
        assert_eq!(token.as_deref(), Some("abc.def.ghi"));
    }

    #[test]
    fn parse_token_rejects_blank() {
        assert!(parse_token("   ").is_none());
    }

    #[test]
    fn parse_user_returns_none_for_invalid_json() {
        let user = parse_user("{not-json}");
        assert!(user.is_none());
    }

    #[test]
    fn parse_user_returns_some_for_valid_json() {
        let raw = r#"{"id":1,"username":"u","email":"e@example.com","created_at":"2026-01-01T00:00:00Z"}"#;
        let user = parse_user(raw);
        assert!(user.is_some());
        let user = user.expect("user should parse");
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "u");
    }
}
