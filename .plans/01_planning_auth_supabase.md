# Supabase Authentication Implementation Plan

## Overview

This document outlines the complete implementation plan for Supabase authentication in the Content Profile application.

### Current Infrastructure

The application already has authentication foundation:

- ✅ `AuthService` - Handles login, signup, logout, token refresh, and user retrieval
- ✅ `UserContext` - Provides authentication state management across the app
- ✅ `SessionStorage` - Manages session persistence in localStorage
- ✅ Auth Models - `Session`, `User`, `LoginRequest`, `AuthResponse`, `AuthError`
- ✅ Config System - Supports Supabase URL and anon key configuration

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    App Components                           │
│  (Dashboard, Content CRUD, Tag Management)              │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
              ┌─────────────────┐
              │  UserContext   │
              │  (Auth State)  │
              └────────┬────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
        ▼              ▼              ▼
  ┌──────────┐  ┌──────────┐  ┌──────────┐
  │SessionSto│  │AuthService│  │Protected │
  │rage      │  │(Supabase)│  │Routes    │
  └──────────┘  └──────────┘  └──────────┘
                       │
                       ▼
              ┌─────────────────┐
              │   Supabase     │
              │  (auth/v1/*)   │
              └─────────────────┘
```

---

## Phase 1: Supabase Project Setup

### 1.1 Create Supabase Project

1. Go to [supabase.com](https://supabase.com)
2. Click "New Project"
3. Choose organization (or create new)
4. Project name: `content-profile`
5. Database password: Generate strong password (save securely!)
6. Region: Choose closest to your users
7. Pricing plan: Free tier is sufficient for start
8. Click "Create new project" (takes ~2 minutes)

### 1.2 Enable Email Authentication

1. Navigate to **Authentication** → **Providers**
2. Select **Email** provider
3. Ensure "Confirm email" is toggled:
   - **Development**: OFF (for easier testing)
   - **Production**: ON (required for security)
4. Click "Save"

### 1.3 Get Project Credentials

1. Navigate to **Project Settings** → **API**
2. Copy these values:
   - **Project URL**: `https://your-project-id.supabase.co`
   - **anon public**: Long JWT key starting with `eyJhbGc...`

---

## Phase 2: Configuration

### 2.1 Set Environment Variables

Create `.env` file in project root (development):

```bash
# Application Mode
APP_MODE=supabase

# Supabase Configuration
SUPABASE_URL=https://your-project-id.supabase.co
SUPABASE_ANON_KEY=eyJhbGc...your-anon-key

# Sync Configuration
SYNC_ENABLED=true
```

**Important**: Never commit `.env` to version control!

### 2.2 Update Build Configuration

Ensure `build.rs` sets the environment variables for compilation:

```rust
// build.rs
fn main() {
    dotenvy::dotenv().ok();

    println!("cargo:rustc-env=APP_MODE={}", env::var("APP_MODE").unwrap_or("office".to_string()));
    println!("cargo:rustc-env=SUPABASE_URL={}", env::var("SUPABASE_URL").unwrap_or_default());
    println!("cargo:rustc-env=SUPABASE_ANON_KEY={}", env::var("SUPABASE_ANON_KEY").unwrap_or_default());
    println!("cargo:rustc-env=SYNC_ENABLED={}", env::var("SYNC_ENABLED").unwrap_or("false".to_string()));
}
```

### 2.3 Verify Configuration

Check that config loads correctly by looking for console logs:

```
INFO: APP_MODE set to 'supabase'
INFO: SUPABASE_URL found: https://***
INFO: SUPABASE_ANON_KEY found: eyJh***
INFO: Configuration loaded: mode=Supabase, sync_enabled=true, supabase_url=true
```

---

## Phase 3: Authentication Flow

### 3.1 Login Flow

```
User enters credentials → validate inputs → call AuthService.login()
  ↓
Supabase validates email/password
  ↓
Receive Session (access_token, refresh_token, user)
  ↓
Save session to localStorage via SessionStorage
  ↓
Navigate to Dashboard
```

### 3.2 Signup Flow

```
User enters email/password → validate inputs → call AuthService.signup()
  ↓
Supabase creates user account
  ↓
Receive Session (access_token, refresh_token, user)
  ↓
Save session to localStorage via SessionStorage
  ↓
Navigate to Dashboard
```

### 3.3 Logout Flow

```
User clicks logout → call AuthService.logout()
  ↓
Clear session from localStorage
  ↓
Navigate to Login page
```

### 3.4 Token Refresh Flow (Automatic)

```
Any API call fails with 401/403
  ↓
Call AuthService.refresh_token(refresh_token)
  ↓
Update session in localStorage
  ↓
Retry original API call
```

---

## Phase 4: UI Components

### 4.1 Create Login Page Component

**File**: `src/pages/login.rs`

```rust
use crate::components::LoginForm;
use crate::contexts::UserContext;
use crate::routes::Route;
use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    let user_context = use_context::<UserContext>();
    let navigate = use_navigator();
    
    // Redirect if already logged in
    let is_authenticated = use_resource(move || async move {
        user_context.load_saved_session().ok()
    });
    
    use_effect(move || {
        if let Some(session) = is_authenticated.read().as_ref() {
            if session.is_some() {
                navigate.push(Route::Dashboard {});
            }
        }
    });
    
    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8",
            LoginForm {
                on_login: move |email: String, password: String| {
                    // Handle login logic
                },
                on_signup_click: move |_| {
                    navigate.push(Route::Signup {});
                }
            }
        }
    }
}
```

### 4.2 Create Signup Page Component

**File**: `src/pages/signup.rs`

Similar structure to login, but uses signup endpoint.

### 4.3 Create Auth Form Components

**File**: `src/components/auth_form.rs`

**LoginForm Component**:
- Email input field (with validation)
- Password input field (with visibility toggle)
- "Login" button
- "Don't have an account? Sign up" link
- Error message display
- Loading state during API call

**SignupForm Component**:
- Same as LoginForm plus:
- Confirm password field
- "Sign up" button
- "Already have an account? Log in" link

### 4.4 Create Protected Route Wrapper

**File**: `src/components/protected_route.rs`

```rust
use dioxus::prelude::*;

#[component]
pub fn ProtectedRoute(children: Element) -> Element {
    let user_context = use_context::<UserContext>();
    let navigate = use_navigator();
    let is_authenticated = use_resource(move || async move {
        user_context.load_saved_session().ok()
    });
    
    match &*is_authenticated.read() {
        Some(Some(_)) => {
            // Authenticated - render children
            children
        }
        Some(None) | None => {
            // Loading or not authenticated
            rsx! {
                div { class: "flex items-center justify-center min-h-screen",
                    p { "Loading..." }
                }
            }
        }
    }
}
```

---

## Phase 5: Route Integration

### 5.1 Update Routes Enum

**File**: `src/routes.rs`

```rust
#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    // Public routes
    #[route("/")]
    Home {},
    
    #[route("/login")]
    Login {},
    
    #[route("/signup")]
    Signup {},
    
    // Protected routes (under layout)
    #[layout(ProtectedLayout)]
        #[route("/dashboard")]
        Dashboard {},
        
        #[route("/content")]
        ContentList {},
        
        #[route("/content/new")]
        ContentEdit { id: 0 },
        
        #[route("/content/:id/edit")]
        ContentEdit { id: i32 },
}
```

### 5.2 Create Protected Layout

**File**: `src/layouts/protected_layout.rs`

```rust
use crate::components::ProtectedRoute;
use crate::contexts::UserContext;
use crate::routes::Route;
use dioxus::prelude::*;

#[component]
pub fn ProtectedLayout() -> Element {
    let user_context = use_context::<UserContext>();
    let navigate = use_navigator();
    
    rsx! {
        ProtectedRoute {
            Navbar { 
                user_context: user_context.clone() 
            }
            Outlet::<Route> {}
        }
    }
}

#[component]
fn Navbar(user_context: UserContext) -> Element {
    // Navigation bar with logout functionality
    rsx! {
        // Navbar implementation
    }
}
```

---

## Phase 6: Session Management

### 6.1 Session Storage Implementation

Ensure `src/services/session.rs` implements:

- `save_session(session: &Session)` - Save to localStorage
- `load_session() -> Result<Option<Session>, String>` - Load from localStorage
- `clear_session() -> Result<(), String>` - Remove from localStorage
- `has_valid_session() -> bool` - Check if session exists and isn't expired

### 6.2 Auto-Refresh Token

Create a hook in `src/hooks/use_auth.rs`:

```rust
use dioxus::prelude::*;

pub fn use_auto_refresh() {
    let user_context = use_context::<UserContext>();
    
    use_effect(move || {
        async move {
            if let Ok(Some(session)) = user_context.load_saved_session() {
                // Check if token is expired or about to expire
                let now = chrono::Utc::now().timestamp();
                let expires_at = session.expires_at;
                let five_minutes = 300;
                
                if expires_at - now < five_minutes {
                    if let Ok(_) = user_context.refresh_token(&session.refresh_token).await {
                        // Token refreshed successfully
                    }
                }
            }
        }
    });
}
```

---

## Phase 7: Error Handling

### 7.1 Common Auth Errors

| Error | Description | Solution |
|-------|-------------|------------|
| `invalid_grant` | Invalid email/password | Show "Invalid credentials" |
| `User not found` | Email doesn't exist | Suggest signing up |
| `Email not confirmed` | Email verification required | Ask user to check email |
| `email_signup_disabled` | Email signup disabled | Enable in Supabase |
| `Invalid login credentials` | Invalid API key | Check configuration |
| `JWT expired` | Token expired | Auto-refresh token |

### 7.2 Error UI Components

**File**: `src/components/error_message.rs`

```rust
#[component]
pub fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: "rounded-md bg-red-50 p-4",
            div { class: "flex",
                div { class: "flex-shrink-0",
                    svg {
                        class: "h-5 w-5 text-red-400",
                        view_box: "0 0 20 20",
                        fill: "currentColor",
                        path {
                            fill_rule: "evenodd",
                            d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z",
                            clip_rule: "evenodd"
                        }
                    }
                }
                div { class: "ml-3",
                    h3 { class: "text-sm font-medium text-red-800", "Error" }
                    p { class: "mt-2 text-sm text-red-700", "{message}" }
                }
            }
        }
    }
}
```

---

## Phase 8: Testing

### 8.1 Manual Testing Checklist

**Login Flow**:
- [ ] Can navigate to /login
- [ ] Can enter email and password
- [ ] Shows validation errors for invalid inputs
- [ ] Successfully logs in with correct credentials
- [ ] Redirects to dashboard after login
- [ ] Shows error message with wrong credentials
- [ ] Can click link to go to signup page

**Signup Flow**:
- [ ] Can navigate to /signup
- [ ] Can enter email, password, and confirm password
- [ ] Shows validation errors for mismatched passwords
- [ ] Successfully creates account
- [ ] Redirects to dashboard after signup
- [ ] Shows error if email already exists
- [ ] Can click link to go to login page

**Session Persistence**:
- [ ] Session is saved to localStorage
- [ ] User remains logged in after page refresh
- [ ] Can access protected routes with valid session
- [ ] Redirected to login when logged out
- [ ] Session expires correctly

**Token Refresh**:
- [ ] Tokens auto-refresh before expiration
- [ ] API calls succeed after token refresh
- [ ] Logged out if refresh fails

**Logout**:
- [ ] Can click logout button
- [ ] Session is cleared from localStorage
- [ ] Redirected to login page
- [ ] Cannot access protected routes after logout

### 8.2 Automated Tests

**File**: `tests/integration_tests/auth_tests.rs`

```rust
#[cfg(test)]
mod auth_tests {
    use super::*;
    
    #[test]
    fn test_login_request_serialization() {
        // Test serialization of LoginRequest
    }
    
    #[test]
    fn test_session_serialization() {
        // Test serialization/deserialization of Session
    }
    
    // Add more tests...
}
```

---

## Phase 9: Security Considerations

### 9.1 Best Practices

1. **Never store passwords in localStorage** - Only store JWT tokens
2. **Use HTTPS in production** - Never send auth over HTTP
3. **Implement rate limiting** - Prevent brute force attacks (handled by Supabase)
4. **Validate inputs on client** - Add client-side validation for better UX
5. **Enable email confirmation** - Required in production
6. **Short token expiration** - Set appropriate expiration times
7. **Secure refresh tokens** - Store securely in Supabase
8. **Logout on all devices** - Option to revoke all sessions

### 9.2 Environment Variable Security

```bash
# .gitignore (already exists)
.env
.env.local
.env.production

# .env.example (commit this!)
APP_MODE=office
SUPABASE_URL=https://your-project-id.supabase.co
SUPABASE_ANON_KEY=your-anon-key-here
SYNC_ENABLED=true
```

---

## Phase 10: Deployment Considerations

### 10.1 Production Configuration

```bash
# Production .env (never commit!)
APP_MODE=supabase
SUPABASE_URL=https://your-project-id.supabase.co
SUPABASE_ANON_KEY=your-production-anon-key
SYNC_ENABLED=true
```

### 10.2 Supabase Production Setup

1. **Enable Email Confirmation**:
   - Go to Authentication → Providers → Email
   - Toggle "Confirm email" to ON
   
2. **Set Up Custom SMTP** (optional):
   - Go to Project Settings → Authentication
   - Configure SMTP settings for transactional emails

3. **Enable Row Level Security (RLS)**:
   - Go to Database → Policies
   - Enable RLS on user-related tables
   - Create policies that check `auth.uid()`

4. **Set Up Auth Hooks**:
   - Go to Database → Triggers → Auth Hooks
   - Create `auth.users` triggers for:
     - Creating user profiles
     - Logging user activity
     - Sending welcome emails

---

## Implementation Order

1. **Phase 1-2**: Supabase setup and configuration (30 minutes)
2. **Phase 3-4**: Auth flow and UI components (2-3 hours)
3. **Phase 5**: Route integration (30 minutes)
4. **Phase 6**: Session management (1 hour)
5. **Phase 7**: Error handling (30 minutes)
6. **Phase 8**: Testing (1-2 hours)
7. **Phase 9**: Security review (30 minutes)
8. **Phase 10**: Deployment prep (1 hour)

**Total Estimated Time**: 7-9 hours

---

## Next Steps

1. ✅ Review this plan
2. ⬜ Create Supabase project (Phase 1)
3. ⬜ Set up environment variables (Phase 2)
4. ⬜ Create auth form components (Phase 4)
5. ⬜ Integrate with routes (Phase 5)
6. ⬜ Test thoroughly (Phase 8)
7. ⬜ Deploy to production (Phase 10)

---

## Resources

- [Supabase Auth Documentation](https://supabase.com/docs/guides/auth)
- [Supabase JavaScript Client](https://supabase.com/docs/reference/javascript)
- [Dioxus Documentation](https://dioxuslabs.com/docs)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [JWT.io](https://jwt.io/) - Debug JWT tokens