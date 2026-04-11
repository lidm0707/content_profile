# Supabase Authentication System - Quick Reference

## Overview

This document provides a quick reference guide for the Supabase authentication system implemented in the Content Profile application.

## Document Structure

| Document | Purpose |
|----------|---------|
| [`01_planning_auth_supabase.md`](./01_planning_auth_supabase.md) | Complete implementation plan with 10 phases |
| [`02_token_cookie_storage.md`](./02_token_cookie_storage.md) | Token/cookie storage strategy and security analysis |
| [`03_login_supabase_examples.md`](./03_login_supabase_examples.md) | Code examples and patterns for authentication |
| `README_AUTH.md` (this file) | Quick reference guide |

## Quick Start

### 1. Configuration

Set up environment variables in `.env`:

```env
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
```

### 2. Basic Login

```rust
use crate::contexts::UserContext;
use dioxus::prelude::*;

#[component]
fn MyComponent() -> Element {
    let user_context = use_context::<UserContext>();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);

    let handle_login = move |_| {
        let email_val = email.read().clone();
        let password_val = password.read().clone();
        let user_context = user_context.clone();

        spawn(async move {
            match user_context.login(email_val, password_val).await {
                Ok(session) => {
                    // Logged in successfully
                    tracing::info!("Logged in as: {}", session.user.email);
                }
                Err(e) => {
                    tracing::error!("Login failed: {}", e);
                }
            }
        });
    };

    rsx! {
        input {
            r#type: "email",
            value: "{email}",
            oninput: move |e| *email.write() = e.value()
        }
        input {
            r#type: "password",
            value: "{password}",
            oninput: move |e| *password.write() = e.value()
        }
        button { onclick: handle_login, "Login" }
    }
}
```

## Core Components

### AuthService
**Location:** `src/services/auth.rs`

Manages all Supabase API calls:
- `login(email, password)` - Authenticate user
- `signup(email, password)` - Create new user
- `logout(access_token)` - End session
- `refresh_token(refresh_token)` - Get new access token
- `get_user(access_token)` - Fetch user details

### UserContext
**Location:** `src/contexts/user_context.rs`

Provides authentication state management:
- Wraps AuthService with session storage
- Handles session persistence via localStorage
- Provides methods for login, signup, logout, token refresh

### SessionStorage
**Location:** `src/services/session.rs`

Manages localStorage session persistence:
- `save_session(session)` - Save session to localStorage
- `load_session()` - Load session from localStorage
- `clear_session()` - Remove session from localStorage
- `has_valid_session()` - Check if session exists and is valid

### Models
**Location:** `src/models/auth.rs`

Data structures:
- `Session` - Complete session with tokens and user info
- `User` - User information from Supabase
- `LoginRequest` - Login/signup request payload
- `AuthResponse` - Response from Supabase auth endpoints
- `AuthError` - Error structure from Supabase

## Architecture

```
User Action (Login/Signup)
    ↓
Login Page Component
    ↓
UserContext (auth state management)
    ↓
AuthService (Supabase API calls)
    ↓
SessionStorage (localStorage persistence)
    ↓
App State (context provider)
```

## Authentication Flow

### Login Flow
```
User enters credentials → UserContext.login()
    ↓
AuthService.login() → Supabase API
    ↓
Receive Session (access_token, refresh_token, user)
    ↓
SessionStorage.save_session()
    ↓
Navigate to Dashboard
```

### Protected Route Flow
```
User accesses protected route
    ↓
Check UserContext.load_saved_session()
    ↓
If valid session → Show content
If invalid/missing → Redirect to Login
```

### Token Refresh Flow
```
API call with access_token fails (401)
    ↓
AuthService.refresh_token(refresh_token)
    ↓
Receive new session
    ↓
SessionStorage.save_session()
    ↓
Retry original API call
```

## Key Patterns

### 1. Check Authentication State

```rust
let user_context = use_context::<UserContext>();
let session = user_context.load_saved_session().ok().flatten();

if let Some(sess) = session {
    // User is authenticated
    rsx! { div { "Welcome, {sess.user.email}" } }
} else {
    // User is not authenticated
    rsx! { div { "Please log in" } }
}
```

### 2. Protected Route Wrapper

```rust
#[component]
fn ProtectedRoute(children: Element) -> Element {
    let user_context = use_context::<UserContext>();
    let navigate = use_navigator();
    
    let is_authenticated = user_context.has_valid_saved_session();
    
    use_effect(move || {
        if !is_authenticated {
            navigate.push(Route::Login {});
        }
    });
    
    if is_authenticated {
        rsx! { {children} }
    } else {
        rsx! { div { "Redirecting..." } }
    }
}
```

### 3. Logout

```rust
let user_context = use_context::<UserContext>();
let session = user_context.load_saved_session().ok().flatten();

spawn(async move {
    if let Some(sess) = session {
        let _ = user_context.logout(Some(sess.access_token)).await;
    }
});
```

### 4. Auto-Refresh Token

```rust
let user_context = use_context::<UserContext>();
let session = user_context.load_saved_session().ok().flatten();

if let Some(sess) = session {
    let now = chrono::Utc::now().timestamp();
    let expires_in = sess.expires_at - now;
    
    // Refresh if token expires in less than 5 minutes
    if expires_in < 300 {
        match user_context.refresh_token(&sess.refresh_token).await {
            Ok(new_session) => {
                // Token refreshed successfully
            }
            Err(e) => {
                // Handle refresh failure
            }
        }
    }
}
```

## Security Considerations

### Current State
- **Storage:** localStorage (medium security)
- **Token Type:** JWT (access + refresh)
- **Expiration:** Configurable via Supabase project settings

### Recommended Enhancements

1. **Immediate Priority:**
   - Add Content Security Policy (CSP) headers
   - Implement input sanitization
   - Shorten access token expiration to 10 minutes

2. **Short-term:**
   - Implement hybrid storage approach (refresh in localStorage, access in memory)
   - Add automatic token refresh
   - Implement token encryption

3. **Long-term:**
   - Migrate to HttpOnly cookies via proxy server
   - Add CSRF protection
   - Implement 2FA support

See [`02_token_cookie_storage.md`](./02_token_cookie_storage.md) for detailed security analysis and migration guides.

## Common Use Cases

### 1. Protecting a Page
```rust
use crate::contexts::UserContext;

#[component]
fn Dashboard() -> Element {
    let user_context = use_context::<UserContext>();
    
    if !user_context.has_valid_saved_session() {
        return rsx! { div { "Please log in" } };
    }
    
    // Render dashboard content
    rsx! { div { "Dashboard content" } }
}
```

### 2. Displaying User Profile
```rust
#[component]
fn UserProfile() -> Element {
    let user_context = use_context::<UserContext>();
    let mut session = use_signal(|| user_context.load_saved_session().ok().flatten());
    
    rsx! {
        if let Some(sess) = session.read().as_ref() {
            div {
                p { "Email: {sess.user.email}" }
                p { "ID: {sess.user.id}" }
            }
        }
    }
}
```

### 3. Making Authenticated API Calls
```rust
async fn fetch_protected_data(
    user_context: &UserContext,
    session: &Session,
) -> Result<String, String> {
    let access_token = &session.access_token;
    
    // Use access_token in Authorization header
    // headers.set("Authorization", &format!("Bearer {}", access_token));
    
    // Make API call...
    Ok("data".to_string())
}
```

## Error Handling

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Authentication is not configured" | Missing Supabase credentials | Set `SUPABASE_URL` and `SUPABASE_ANON_KEY` |
| "Invalid login credentials" | Wrong email or password | Verify credentials exist in Supabase |
| "Email not confirmed" | User hasn't verified email | Confirm email via Supabase |
| "Token refresh failed" | Refresh token expired | User must log in again |

### Error Display Pattern

```rust
let mut error = use_signal(|| Option::<String>::None);

// In your RSX
if let Some(err) = error.read().as_ref() {
    div {
        class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded",
        "{err}"
    }
}
```

## Routes

| Route | Component | Auth Required |
|-------|-----------|---------------|
| `/` | `Home` | No |
| `/login` | `Login` | No |
| `/dashboard` | `Dashboard` | Yes (should be protected) |
| `/content/edit/:id` | `ContentEdit` | Yes (should be protected) |

## Testing

### Running Tests
```bash
cargo test
```

### Test Coverage
- AuthService unit tests
- Session serialization tests
- Model serialization tests

See [`01_planning_auth_supabase.md`](./01_planning_auth_supabase.md) Phase 8 for detailed testing strategy.

## Implementation Status

### ✅ Completed
- [x] AuthService with all Supabase endpoints
- [x] UserContext for auth state management
- [x] SessionStorage with localStorage
- [x] Complete data models
- [x] Login page with signup mode
- [x] Route integration
- [x] Error handling

### 🚧 In Progress
- [ ] Protected route wrapper
- [ ] Auto-refresh token functionality
- [ ] Input sanitization
- [ ] CSP headers

### 📋 Planned
- [ ] Hybrid storage approach
- [ ] Token encryption
- [ ] Password reset flow
- [ ] Email verification UI
- [ ] OAuth providers
- [ ] 2FA support

## Code Examples

For detailed code examples and patterns, see [`03_login_supabase_examples.md`](./03_login_supabase_examples.md).

Examples include:
- Basic login flow
- Protected route wrapper
- User profile display
- Auto-refresh token provider
- Session state hook
- Form validation
- Error boundaries
- Loading states

## Troubleshooting

### Issues & Solutions

1. **Configuration not loaded**
   - Check `.env` file exists
   - Verify environment variables in `build.rs`
   - Restart development server

2. **Session not persisting**
   - Check browser localStorage is enabled
   - Verify `SessionStorage::save_session()` is called
   - Check for JavaScript errors

3. **Token expiration errors**
   - Implement auto-refresh mechanism
   - Check `expires_at` before using token
   - Handle 401 responses gracefully

4. **Login failures**
   - Verify Supabase project is active
   - Check email confirmation status
   - Ensure user exists in Supabase auth

## Related Files

### Source Files
- `src/services/auth.rs` - Supabase API client
- `src/services/session.rs` - localStorage management
- `src/contexts/user_context.rs` - Auth state context
- `src/models/auth.rs` - Auth data models
- `src/pages/login.rs` - Login/signup page
- `src/routes.rs` - Route definitions

### Documentation
- [`01_planning_auth_supabase.md`](./01_planning_auth_supabase.md) - Full implementation plan
- [`02_token_cookie_storage.md`](./02_token_cookie_storage.md) - Security and storage strategy
- [`03_login_supabase_examples.md`](./03_login_supabase_examples.md) - Code examples

## Resources

- [Supabase Auth Documentation](https://supabase.com/docs/guides/auth)
- [Dioxus Documentation](https://dioxuslabs.com/learn/0.7)
- [Supabase JS Client Reference](https://supabase.com/docs/reference/javascript)

## Next Steps

1. **Immediate:**
   - Review [`01_planning_auth_supabase.md`](./01_planning_auth_supabase.md) for implementation details
   - Check [`02_token_cookie_storage.md`](./02_token_cookie_storage.md) for security recommendations
   - Explore code examples in [`03_login_supabase_examples.md`](./03_login_supabase_examples.md)

2. **Implementation:**
   - Add protected route wrapper to `/dashboard` and `/content/edit/*`
   - Implement auto-refresh token functionality
   - Add security enhancements (CSP, input sanitization)

3. **Testing:**
   - Test login/logout flow
   - Verify session persistence
   - Test token refresh mechanism

---

**Last Updated:** Based on current project state

**Maintainer:** Content Profile Team