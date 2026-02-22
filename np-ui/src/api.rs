//! API utilities for handling HTTP requests and responses

/// Check if a response indicates unauthorized (401) and logout if so.
/// Returns true if the user was logged out.
pub fn handle_unauthorized(status: u16) -> bool {
    if status == 401 {
        logout_and_redirect();
        true
    } else {
        false
    }
}

/// Check response status and handle 401 automatically.
/// Returns Err with appropriate message for error statuses.
pub fn check_response_status(status: u16, ok: bool) -> Result<(), String> {
    if status == 401 {
        logout_and_redirect();
        Err("Session expired. Please login again.".to_string())
    } else if !ok {
        Err(format!("HTTP error: {}", status))
    } else {
        Ok(())
    }
}

/// Clear the auth token and redirect to login page
pub fn logout_and_redirect() {
    if let Some(window) = web_sys::window() {
        // Clear the token
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("np_token");
        }
        // Redirect to login
        let _ = window.location().set_href("/login");
    }
}

/// Get the authentication token from localStorage
pub fn get_auth_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("np_token").ok().flatten())
}
