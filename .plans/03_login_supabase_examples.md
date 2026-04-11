# Supabase Login Implementation Guide and Examples (844 lines)

## Overview

This document provides comprehensive examples and patterns for implementing Supabase authentication in Content Profile application. It reviews the current infrastructure and demonstrates how to use the authentication system effectively.

## Current Infrastructure

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        App Component                         │
│                   (provides UserContext)                     │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ├─────────────────────────────────────┐
                     │                                     │
                     ▼                                     ▼
              ┌──────────────┐                     ┌──────────────┐
              │ Login Page   │                     │ Dashboard    │
              │              │                     │              │
              │ UserContext  │                     │ UserContext  │
              │ Session      │◀──localStorage─────▶│ Session      │
              └──────┬───────┘                     └──────┬───────┘
                     │                                     │
                     ▼                                     ▼
              ┌──────────────┐                     ┌──────────────┐
              │ AuthService  │                     │ AuthService  │
              │              │                     │              │
              │ login()      │◀─────Supabase──────▶│ get_user()   │
              │ signup()     │                     │ refresh()    │
              │ logout()     │                     │ logout()     │
              └──────────────┘                     └──────────────┘
```

### Core Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `AuthService` | `src/services/auth.rs` | Handles all Supabase auth API calls |
| `UserContext` | `src/contexts/user_context.rs` | Manages auth state and session persistence |
| `SessionStorage` | `src/services/session.rs` | Handles localStorage session storage |
| `Login` | `src/pages/login.rs` | Login/signup page component |
| Models | `src/models/auth.rs` | Session, User, LoginRequest, AuthResponse, AuthError |

## Usage Examples

### Example 1: Basic Login Flow

```rust
// Component: Basic login usage
use crate::contexts::UserContext;
use dioxus::prelude::*;

#[component]
fn SimpleLoginExample() -> Element {
    let user_context = use_context::<UserContext>();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);

    let handle_login = move |_| {
        let email_val = email.read().clone();
        let password_val = password.read().clone();
        let user_context = user_context.clone();

        if email_val.is_empty() || password_val.is_empty() {
            *error.write() = Some("Email and password are required".to_string());
            return;
        }

        *loading.write() = true;

        spawn(async move {
            match user_context.login(email_val, password_val).await {
                Ok(session) => {
                    tracing::info!("Logged in as: {}", session.user.email);
                    // Navigation is handled by Login component
                }
                Err(e) => {
                    *error.write() = Some(e);
                    *loading.write() = false;
                }
            }
        });
    };

    rsx! {
        div { class: "p-4",
            input {
                r#type: "email",
                placeholder: "Email",
                value: "{email}",
                oninput: move |e| *email.write() = e.value()
            }
            input {
                r#type: "password",
                placeholder: "Password",
                value: "{password}",
                oninput: move |e| *password.write() = e.value()
            }
            button {
                onclick: handle_login,
                disabled: *loading.read(),
                if *loading.read() { "Loading..." } else { "Login" }
            }
            if let Some(err) = error.read().as_ref() {
                div { class: "text-red-500", "{err}" }
            }
        }
    }
}
```

### Example 2: Protected Route Wrapper

```rust
// Component: Protected route that requires authentication
use crate::contexts::UserContext;
use crate::routes::Route;
use dioxus::prelude::*;

#[component]
fn ProtectedRoute(children: Element) -> Element {
    let user_context = use_context::<UserContext>();
    let mut session = use_signal(|| user_context.load_saved_session().ok().flatten());
    let navigate = use_navigator();

    use_effect(move || {
        if session.read().is_none() {
            navigate.push(Route::Login {});
        }
    });

    rsx! {
        if let Some(sess) = session.read().as_ref() {
            div {
                p { "Welcome, {sess.user.email}" }
                {children}
            }
        } else {
            div { class: "p-4", "Loading authentication..." }
        }
    }
}
```

### Example 3: User Profile Display

```rust
// Component: Display user profile with logout
use crate::contexts::UserContext;
use crate::models::Session;
use dioxus::prelude::*;

#[component]
fn UserProfile(session: Session) -> Element {
    let user_context = use_context::<UserContext>();
    let mut loading = use_signal(|| false);

    let handle_logout = move |_| {
        let user_context = user_context.clone();
        let access_token = session.access_token.clone();
        
        *loading.write() = true;

        spawn(async move {
            let _ = user_context.logout(Some(access_token)).await;
            *loading.write() = false;
        });
    };

    rsx! {
        div { class: "bg-white shadow rounded-lg p-6",
            div { class: "flex items-center space-x-4",
                div { class: "flex-1",
                    h3 { class: "text-lg font-medium text-gray-900",
                        "{session.user.email}"
                    }
                    p { class: "text-sm text-gray-500",
                        "ID: {session.user.id}"
                    }
                    if let Some(confirmed_at) = &session.user.email_confirmed_at {
                        p { class: "text-sm text-green-600",
                            "Email confirmed at: {confirmed_at}"
                        }
                    }
                }
                button {
                    onclick: handle_logout,
                    disabled: *loading.read(),
                    class: "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700",
                    if *loading.read() { "Logging out..." } else { "Logout" }
                }
            }
        }
    }
}
```

### Example 4: Auto-Refresh Token Provider

```rust
// Component: AuthProvider with automatic token refresh
use crate::contexts::UserContext;
use crate::models::Session;
use dioxus::prelude::*;

/// Provides authentication state with auto-refresh
#[component]
fn AuthProvider(children: Element) -> Element {
    let user_context = use_context::<UserContext>();
    let mut session = use_signal(|| user_context.load_saved_session().ok().flatten());
    let mut is_refreshing = use_signal(|| false);

    let refresh_token = move || {
        let user_context = user_context.clone();
        let mut session = session.clone();
        let mut is_refreshing = is_refreshing.clone();

        spawn(async move {
            if *is_refreshing.read() {
                return;
            }

            if let Some(sess) = session.read().as_ref() {
                let now = chrono::Utc::now().timestamp();
                let expires_in = sess.expires_at - now;

                if expires_in < 300 && !*is_refreshing.read() {
                    *is_refreshing.write() = true;

                    match user_context.refresh_token(&sess.refresh_token).await {
                        Ok(new_session) => {
                            *session.write() = Some(new_session);
                            tracing::info!("Token refreshed successfully");
                        }
                        Err(e) => {
                            tracing::error!("Failed to refresh token: {}", e);
                            *session.write() = None;
                            let _ = user_context.clear_saved_session();
                        }
                    }

                    *is_refreshing.write() = false;
                }
            }
        });
    };

    use_resource(move || async move {
        refresh_token();
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    });

    rsx! {
        {children}
    }
}
```

### Example 5: Session State Hook

```rust
// Custom hook: use_auth_state
use crate::contexts::UserContext;
use crate::models::Session;
use dioxus::prelude::*;

/// Custom hook for managing authentication state
pub fn use_auth_state() -> (
    ReadOnlySignal<Option<Session>>,
    impl Fn(String, String) -> dioxus_core::Task + Clone,
    impl Fn() -> dioxus_core::Task + Clone,
) {
    let user_context = use_context::<UserContext>();
    let mut session = use_signal(|| user_context.load_saved_session().ok().flatten());

    let login = {
        let user_context = user_context.clone();
        let mut session = session.clone();

        move |email: String, password: String| {
            let user_context = user_context.clone();
            let mut session = session.clone();

            spawn(async move {
                match user_context.login(email, password).await {
                    Ok(sess) => {
                        *session.write() = Some(sess);
                    }
                    Err(e) => {
                        tracing::error!("Login failed: {}", e);
                    }
                }
            })
        }
    };

    let logout = {
        let user_context = user_context.clone();
        let mut session = session.clone();

        move || {
            let user_context = user_context.clone();
            let mut session = session.clone();
            let access_token = session.read().as_ref().map(|s| s.access_token.clone());

            spawn(async move {
                let _ = user_context.logout(access_token).await;
                *session.write() = None;
            })
        }
    };

    (session.into(), login, logout)
}

// Usage example
#[component]
fn UsingAuthStateHook() -> Element {
    let (session, login, logout) = use_auth_state();

    rsx! {
        if let Some(sess) = session() {
            div {
                p { "Logged in as: {sess.user.email}" }
                button { onclick: move |_| { logout(); }, "Logout" }
            }
        } else {
            button {
                onclick: move |_| {
                    login("user@example.com".to_string(), "password".to_string());
                },
                "Login"
            }
        }
    }
}
```

## Component Patterns

### Pattern 1: Loading States

```rust
// Always show loading states during async operations
#[component]
fn LoadingStateExample() -> Element {
    let mut loading = use_signal(|| false);

    let handle_action = move |_| {
        *loading.write() = true;
        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            *loading.write() = false;
        });
    };

    rsx! {
        button {
            onclick: handle_action,
            disabled: *loading.read(),
            class: if *loading.read() {
                "opacity-50 cursor-not-allowed"
            } else {
                ""
            },
            if *loading.read() {
                "Processing..."
            } else {
                "Click Me"
            }
        }
    }
}
```

### Pattern 2: Error Boundaries

```rust
// Component: Error message display with auto-dismiss
#[component]
fn ErrorMessage(message: String) -> Element {
    let mut visible = use_signal(|| true);

    use_effect(move || {
        let mut visible = visible.clone();
        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            *visible.write() = false;
        });
    });

    if !*visible.read() {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded relative",
            button {
                class: "absolute top-0 right-0 px-4 py-3",
                onclick: move |_| *visible.write() = false,
                "×"
            }
            "{message}"
        }
    }
}
```

### Pattern 3: Form Validation

```rust
// Component: Validated login form
#[derive(PartialEq)]
struct FormErrors {
    email: Option<String>,
    password: Option<String>,
}

impl Default for FormErrors {
    fn default() -> Self {
        Self {
            email: None,
            password: None,
        }
    }
}

#[component]
fn ValidatedLoginForm() -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut errors = use_signal(FormErrors::default);

    let validate_email = |email: &str| -> Option<String> {
        if email.is_empty() {
            Some("Email is required".to_string())
        } else if !email.contains('@') {
            Some("Invalid email format".to_string())
        } else {
            None
        }
    };

    let validate_password = |password: &str| -> Option<String> {
        if password.is_empty() {
            Some("Password is required".to_string())
        } else if password.len() < 6 {
            Some("Password must be at least 6 characters".to_string())
        } else {
            None
        }
    };

    let handle_submit = move |_| {
        let email_val = email.read().clone();
        let password_val = password.read().clone();

        errors.write().email = validate_email(&email_val);
        errors.write().password = validate_password(&password_val);

        if errors.read().email.is_none() && errors.read().password.is_none() {
            // Form is valid, submit
        }
    };

    rsx! {
        form {
            class: "space-y-4",
            onsubmit: move |e| e.prevent_default(),
            div {
                input {
                    r#type: "email",
                    class: if errors.read().email.is_some() {
                        "border-red-500"
                    } else {
                        ""
                    },
                    value: "{email}",
                    oninput: move |e| {
                        *email.write() = e.value();
                        errors.write().email = None;
                    }
                }
                if let Some(err) = errors.read().email.as_ref() {
                    p { class: "text-red-500 text-sm", "{err}" }
                }
            }
            div {
                input {
                    r#type: "password",
                    class: if errors.read().password.is_some() {
                        "border-red-500"
                    } else {
                        ""
                    },
                    value: "{password}",
                    oninput: move |e| {
                        *password.write() = e.value();
                        errors.write().password = None;
                    }
                }
                if let Some(err) = errors.read().password.as_ref() {
                    p { class: "text-red-500 text-sm", "{err}" }
                }
            }
            button {
                onclick: handle_submit,
                "Submit"
            }
        }
    }
}
```

## Error Handling Patterns

### Error Response Types

```rust
// Supabase returns structured errors
match response.status() {
    400 => {
        // Invalid request (invalid email format, etc.)
        Err("Invalid email or password format".to_string())
    }
    401 => {
        // Unauthorized (wrong credentials)
        Err("Invalid email or password".to_string())
    }
    422 => {
        // Unprocessable entity (email already exists, etc.)
        Err("Email already registered".to_string())
    }
    _ => {
        // Other errors
        Err("An unexpected error occurred".to_string())
    }
}
```

### User-Friendly Error Messages

```rust
fn format_auth_error(error: String) -> String {
    if error.contains("Invalid login credentials") {
        "Invalid email or password".to_string()
    } else if error.contains("User already registered") {
        "An account with this email already exists".to_string()
    } else if error.contains("Email not confirmed") {
        "Please confirm your email address".to_string()
    } else if error.contains("Password should be at least") {
        "Password is too short".to_string()
    } else {
        "An error occurred. Please try again".to_string()
    }
}
```

## Security Best Practices

### 1. Session Validation

```rust
// Always validate session before making authenticated requests
fn validate_session(session: &Option<Session>) -> bool {
    match session {
        Some(sess) => {
            let now = chrono::Utc::now().timestamp();
            now < sess.expires_at
        }
        None => false,
    }
}
```

### 2. Input Sanitization

```rust
// Sanitize user inputs before using them
fn sanitize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn sanitize_password(password: &str) -> String {
    // Don't trim passwords (users might want leading/trailing spaces)
    password.to_string()
}
```

### 3. Token Expiration Handling

```rust
// Check token expiration before using it
fn is_token_valid(session: &Session) -> bool {
    let now = chrono::Utc::now().timestamp();
    let buffer = 60; // 1 minute buffer
    now < (session.expires_at - buffer)
}
```

## Common Use Cases

### Use Case 1: Check Authentication on App Load

```rust
#[component]
fn App() -> Element {
    let user_context = use_context::<UserContext>();
    let mut is_authenticated = use_signal(|| false);
    let mut is_loading = use_signal(|| true);

    use_resource(move || async move {
        match user_context.load_saved_session() {
            Ok(Some(session)) => {
                let now = chrono::Utc::now().timestamp();
                if now < session.expires_at {
                    *is_authenticated.write() = true;
                }
            }
            _ => {}
        }
        *is_loading.write() = false;
    });

    rsx! {
        if *is_loading.read() {
            div { "Loading..." }
        } else {
            if *is_authenticated.read() {
                Dashboard {}
            } else {
                Login {}
            }
        }
    }
}
```

### Use Case 2: Protected API Call with Auto-Refresh

```rust
// Make authenticated API calls with automatic token refresh
async fn protected_api_call<F, T>(
    session: &Signal<Option<Session>>,
    user_context: &UserContext,
    api_call: F,
) -> Result<T, String>
where
    F: FnOnce(&str) -> dioxus_core::Task,
{
    let sess = session.read();
    let access_token = sess
        .as_ref()
        .map(|s| s.access_token.clone())
        .ok_or_else(|| "Not authenticated".to_string())?;

    // Check if token needs refresh
    let now = chrono::Utc::now().timestamp();
    if sess.as_ref().map_or(true, |s| now >= s.expires_at - 300) {
        if let Some(refresh_token) = sess.as_ref().map(|s| s.refresh_token.clone()) {
            match user_context.refresh_token(&refresh_token).await {
                Ok(new_session) => {
                    *session.write() = Some(new_session);
                }
                Err(e) => {
                    return Err(format!("Token refresh failed: {}", e));
                }
            }
        }
    }

    // TODO: Make the actual API call
    // This is a placeholder - actual implementation depends on your API
    Ok(()) as Result<T, String>
}
```

### Use Case 3: Auth State Persistence

```rust
// Persist auth state across page reloads
#[component]
fn AuthPersistenceProvider(children: Element) -> Element {
    let user_context = use_context::<UserContext>();
    let mut session = use_signal(|| user_context.load_saved_session().ok().flatten());

    // Save session to localStorage whenever it changes
    use_effect(move || {
        if let Some(ref sess) = *session.read() {
            let _ = user_context.load_saved_session().ok().flatten();
        }
    });

    rsx! {
        {children}
    }
}
```

## Testing Examples

### Unit Test: AuthService

```rust
#[cfg(test)]
mod auth_tests {
    use super::*;

    #[tokio::test]
    async fn test_login_request_serialization() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("password123"));
    }

    #[tokio::test]
    async fn test_session_serialization() {
        let session = Session {
            access_token: "access_token".to_string(),
            refresh_token: "refresh_token".to_string(),
            expires_at: chrono::Utc::now().timestamp() + 3600,
            token_type: "bearer".to_string(),
            user: User {
                id: "123".to_string(),
                email: "test@example.com".to_string(),
                email_confirmed_at: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
                last_sign_in_at: None,
            },
        };

        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("access_token"));
        assert!(json.contains("test@example.com"));
    }
}
```

## Integration Examples

### Example: Integrating with Content Service

```rust
// Example of using auth with content API
use crate::contexts::UserContext;
use crate::models::Session;
use dioxus::prelude::*;

#[component]
fn ContentListWithAuth() -> Element {
    let user_context = use_context::<UserContext>();
    let mut session = use_signal(|| user_context.load_saved_session().ok().flatten());
    let mut contents = use_signal(|| Vec::<Content>::new);

    let load_contents = move |_| {
        let sess = session.read().clone();
        let mut contents = contents.clone();

        spawn(async move {
            if let Some(s) = sess {
                // Use access_token to fetch content
                // let result = fetch_content_with_auth(&s.access_token).await;
                // match result {
                //     Ok(data) => *contents.write() = data,
                //     Err(e) => tracing::error!("Failed to load content: {}", e),
                // }
            }
        });
    };

    rsx! {
        button { onclick: load_contents, "Load Content" }
        for content in contents.read().iter() {
            div { key: "{content.id}", "{content.title}" }
        }
    }
}
```

## Troubleshooting

### Common Issues and Solutions

1. **"Authentication is not configured"**
   - Ensure `SUPABASE_URL` and `SUPABASE_ANON_KEY` are set in your `.env` file
   - Verify environment variables are loaded correctly in `build.rs`

2. **"Login failed: Invalid login credentials"**
   - Verify the user exists in Supabase
   - Check email confirmation status
   - Ensure password is correct

3. **Token expiration errors**
   - Implement automatic token refresh
   - Check `expires_at` timestamp before using access token
   - Handle 401 errors by refreshing tokens

4. **Session not persisting**
   - Check browser localStorage is enabled
   - Verify `SessionStorage::save_session()` is called
   - Check for JavaScript errors in browser console

## Next Steps

Based on the planning documents, consider implementing:

1. **Phase 1 Security Enhancements** (from `02_token_cookie_storage.md`)
   - Add Content Security Policy headers
   - Implement input sanitization
   - Shorten access token expiration to 10 minutes

2. **Hybrid Storage Approach** (Recommended)
   - Store only refresh token in localStorage
   - Keep access token in memory
   - Implement automatic token refresh

3. **Protected Routes**
   - Create a `ProtectedRoute` wrapper component
   - Add authentication checks to all protected pages
   - Handle unauthorized redirects

4. **Additional Features**
   - Password reset flow
   - Email verification UI
   - OAuth providers (Google, GitHub, etc.)

## Resources

- [Supabase Auth Documentation](https://supabase.com/docs/guides/auth)
- [Dioxus Documentation](https://dioxuslabs.com/learn/0.7)
- [Dioxus Hooks Guide](https://dioxuslabs.com/learn/0.7/guides/hooks)
- [Project Planning Documents](./01_planning_auth_supabase.md)
- [Token Storage Strategy](./02_token_cookie_storage.md)