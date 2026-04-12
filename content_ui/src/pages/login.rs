use crate::contexts::UserContext;
use crate::models::Session;
use crate::routes::Route;
use dioxus::prelude::*;

const TITLE: &str = "Login";
const EMAIL_PLACEHOLDER: &str = "you@example.com";
const PASSWORD_PLACEHOLDER: &str = "••••••••";
const LOGIN_BTN_TEXT: &str = "Login";
const SIGNUP_BTN_TEXT: &str = "Sign Up";
const SWITCH_TO_SIGNUP: &str = "Don't have an account? Sign up";
const SWITCH_TO_LOGIN: &str = "Already have an account? Login";
const ERROR_CLASS: &str = "text-red-500 text-sm mt-2";
const INPUT_CLASS: &str = "appearance-none rounded-md relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm";
const BTN_CLASS: &str = "group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors";

/// Login page component with authentication
#[component]
pub fn Login() -> Element {
    let mut is_signup = use_signal(|| false);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);

    let user_context: UserContext = use_context();
    let mut session_signal: Signal<Option<Session>> = use_context();
    let is_configured = use_signal(|| user_context.is_configured());
    let navigator = use_navigator();

    let handle_auth = move |_| {
        let email_val = email.read().clone();
        let password_val = password.read().clone();
        let is_signup_val = *is_signup.read();
        let user_context = user_context.clone();

        if email_val.is_empty() || password_val.is_empty() {
            *error.write() = Some("Email and password are required".to_string());
            return;
        }

        *loading.write() = true;
        *error.write() = None;

        spawn(async move {
            let result = if is_signup_val {
                user_context.signup(email_val, password_val).await
            } else {
                user_context.login(email_val, password_val).await
            };

            *loading.write() = false;

            match result {
                Ok(session) => {
                    *session_signal.write() = Some(session);
                    navigator.push(Route::Dashboard {});
                }
                Err(e) => {
                    *error.write() = Some(e);
                }
            }
        });
    };

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8",

            if !*is_configured.read() {
                div {
                    class: "max-w-md w-full space-y-8",

                    div {
                        class: "text-center",

                        h2 {
                            class: "mt-6 text-3xl font-extrabold text-gray-900",
                            "{TITLE}"
                        }

                        p {
                            class: "mt-2 text-sm text-gray-600",
                            "Authentication is not configured"
                        }

                        div {
                            class: "mt-4 bg-red-50 border border-red-200 rounded-md p-4",

                            p {
                                class: "text-sm text-red-800",
                                "This application requires Supabase authentication. Please set up Supabase credentials in your configuration."
                            }
                        }
                    }
                }
            } else {
                div {
                    class: "max-w-md w-full space-y-8",

                    div {
                        class: "text-center",

                        h2 {
                            class: "mt-6 text-3xl font-extrabold text-gray-900",
                            "{TITLE}"
                        }

                        p {
                            class: "mt-2 text-sm text-gray-600",
                            if *is_signup.read() {
                                "Create your account to get started"
                            } else {
                                "Sign in to your account"
                            }
                        }
                    }

                    form {
                    class: "mt-8 space-y-6",
                    onsubmit: move |e| {
                        e.prevent_default();
                    },
                    action: "#",
                    method: "POST",

                    div {
                        class: "rounded-md shadow-sm -space-y-px",

                        div {
                            class: "mb-4",

                            label {
                                class: "sr-only",
                                r#for: "email-address",
                                "Email address"
                            }

                            input {
                                id: "email-address",
                                name: "email",
                                r#type: "email",
                                autocomplete: "email",
                                required: "true",
                                class: INPUT_CLASS,
                                placeholder: EMAIL_PLACEHOLDER,
                                value: "{email}",
                                oninput: move |e| {
                                    *email.write() = e.value();
                                    error.write().take();
                                }
                            }
                        }

                        div {

                            label {
                                class: "sr-only",
                                r#for: "password",
                                "Password"
                            }

                            input {
                                id: "password",
                                name: "password",
                                r#type: "password",
                                autocomplete: if *is_signup.read() { "new-password" } else { "current-password" },
                                required: "true",
                                class: INPUT_CLASS,
                                placeholder: PASSWORD_PLACEHOLDER,
                                value: "{password}",
                                oninput: move |e| {
                                    *password.write() = e.value();
                                    error.write().take();
                                }
                            }
                        }
                    }

                    if let Some(err) = error.read().as_ref() {
                        div {
                            class: ERROR_CLASS,
                            "{err}"
                        }
                    }

                    div {
                        button {
                            onclick: handle_auth,
                            disabled: *loading.read(),
                            class: if *loading.read() {
                                format!("{} opacity-50 cursor-not-allowed", BTN_CLASS)
                            } else {
                                BTN_CLASS.to_string()
                            },

                            if *loading.read() {
                                "Loading..."
                            } else {
                                if *is_signup.read() {
                                    "{SIGNUP_BTN_TEXT}"
                                } else {
                                    "{LOGIN_BTN_TEXT}"
                                }
                            }
                        }
                    }

                    div {
                        class: "mt-4 text-center",

                        button {
                            r#type: "button",
                            class: "text-indigo-600 hover:text-indigo-500 text-sm font-medium transition-colors",
                            onclick: move |_| {
                                is_signup.with_mut(|val| *val = !*val);
                                error.write().take();
                            },

                            if *is_signup.read() {
                                "{SWITCH_TO_LOGIN}"
                            } else {
                                "{SWITCH_TO_SIGNUP}"
                            }
                        }
                    }
                    }
                }
            }
        }
    }
}
