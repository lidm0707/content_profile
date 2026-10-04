use content_sdk::contexts::UserContext;
use content_sdk::models::Session;
use content_sdk::services::BackendApiService;
use content_sdk::utils::config::Config;
use dioxus::prelude::*;

use crate::routes::Route;

const TITLE: &str = "Settings";
const SUBTITLE: &str = "Authentication settings stored for your account";
const SAVE_BTN_TEXT: &str = "Save Settings";
const SAVED_MSG: &str = "Settings saved";
const SIGN_IN_BTN_TEXT: &str = "Sign In";
const SIGNING_IN_TEXT: &str = "Signing in...";
const SIGNED_IN_MSG: &str = "Signed in to Supabase";
const SIGNED_IN_AS: &str = "Signed in to Supabase as";
const TAB_SUPABASE: &str = "Supabase";
const TAB_GOOGLE: &str = "Google";
const TAB_SYNC: &str = "Sync";
const TAB_ANON_KEY: &str = "Anon Key";
const TAB_LOGIN: &str = "Login";

const KEY_EMAIL: &str = "supabase_email";
// Password is intentionally never saved — auth persists via refresh token.
const KEY_SUPABASE_URL: &str = "supabase_url";
const KEY_SUPABASE_ANON_KEY: &str = "supabase_anon_key";
use crate::app::KEY_GOOGLE_FOLDER_ID;

const LABEL_EMAIL: &str = "Email";
const LABEL_PASSWORD: &str = "Password";
const LABEL_SUPABASE_URL: &str = "Supabase URL";
const LABEL_ANON_KEY: &str = "Supabase Anon Key";
const LABEL_GOOGLE_FOLDER_ID: &str = "Google Drive Folder ID";
const GOOGLE_CONNECT_BTN: &str = "Connect Google Drive";
const GOOGLE_CONNECTING: &str = "Connecting...";
const GOOGLE_CONNECTED: &str = "Google Drive connected ✓";
const GOOGLE_NOT_CONNECTED: &str = "Not connected";
const GOOGLE_RECONNECT_BTN: &str = "Reconnect";
const STATUS_POLL_INTERVAL_MS: u32 = 3000;
const STATUS_TEXT_CONNECTED_CLASS: &str = "text-sm text-green-600";
const STATUS_TEXT_CLASS: &str = "text-sm text-gray-700";
const BTN_IDLE_CLASS: &str = "inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors";
#[cfg(target_arch = "wasm32")]
const GOOGLE_OPEN_FAILED: &str = "Failed to open Google consent page — allow popups and retry";
const ERR_ANON_KEY_REQUIRED: &str = "Enter a Supabase anon key (Anon Key tab) first";
const SYNC_BTN_TEXT: &str = "Sync from Supabase";
const SYNC_RUNNING_TEXT: &str = "Syncing...";
const SYNC_DONE_FMT: &str = "Synced {n} row(s) from Supabase";
const PLACEHOLDER_SUPABASE_EMAIL: &str = "you@example.com";

const TAB_ACTIVE_CLASS: &str =
    "px-4 py-2 rounded-md text-sm font-medium bg-indigo-600 text-white transition-colors";
const TAB_IDLE_CLASS: &str = "px-4 py-2 rounded-md text-sm font-medium text-gray-400 hover:text-white hover:bg-gray-700 transition-colors";
const TABS_ROW_CLASS: &str = "flex gap-2 border-b border-gray-200 pb-3";
const TAB_SUB_ACTIVE_CLASS: &str = "block w-full text-left px-4 py-2 rounded-md text-sm font-medium bg-indigo-600 text-white transition-colors";
const TAB_SUB_IDLE_CLASS: &str = "block w-full text-left pl-8 pr-4 py-2 rounded-md text-sm text-gray-400 hover:text-white hover:bg-gray-700 transition-colors";
const NAV_CLASS: &str = "w-48 flex-shrink-0";
const NAV_BOX_CLASS: &str = "bg-white shadow-md rounded-lg p-3 space-y-1";
const CONTENT_CLASS: &str = "flex-1 bg-white shadow-md rounded-lg p-6 space-y-4";
const INPUT_CLASS: &str = "appearance-none rounded-md relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm";
const BTN_CLASS: &str = "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsSection {
    Supabase,
    Google,
    Sync,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Login,
    AnonKey,
}

fn field_from(settings: &serde_json::Value, key: &str) -> String {
    settings
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Opens the Google consent page in a new tab.
#[cfg(target_arch = "wasm32")]
fn open_consent_page(url: &str) -> Result<(), wasm_bindgen::JsValue> {
    web_sys::window()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no window"))?
        .open_with_url_and_target(url, "_blank")?
        .map(|_| ())
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("popup blocked"))
}

#[component]
pub fn Settings() -> Element {
    let user_context: UserContext = use_context();
    let session: Signal<Option<Session>> = use_context();
    let mut section = use_signal(|| SettingsSection::Supabase);
    let mut tab = use_signal(|| SettingsTab::Login);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut supabase_url = use_signal(String::new);
    let mut anon_key = use_signal(String::new);
    let mut google_folder_id = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut saved = use_signal(|| false);
    let mut signed_in = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let mut signing_in = use_signal(|| false);
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
    let mut google_connected = use_signal(|| Option::<bool>::None);
    let mut google_connecting = use_signal(|| false);
    let mut syncing = use_signal(|| false);
    let mut sync_result = use_signal(|| Option::<u32>::None);
    let config = use_context::<Memo<Config>>();

    // Google connection status (backend stores the token per user).
    // Polls so the status flips to connected as soon as the user finishes
    // consent in the popup tab, without a manual page refresh.
    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        let api = BackendApiService::new(Some(config.read().clone()));
        spawn(async move {
            loop {
                if let Ok(connected) = api.get_google_status().await
                    && google_connected.read().as_ref() != Some(&connected)
                {
                    google_connected.set(Some(connected));
                }
                gloo_timers::future::TimeoutFuture::new(STATUS_POLL_INTERVAL_MS).await;
            }
        });
    });

    let handle_google_connect = move |_| {
        let api = BackendApiService::new(Some(config.read().clone()));
        *google_connecting.write() = true;
        *error.write() = None;
        spawn(async move {
            match api.get_google_auth_url().await {
                Ok(url) => {
                    #[cfg(target_arch = "wasm32")]
                    if let Err(e) = open_consent_page(&url) {
                        error!("failed to open Google consent page: {e:?}");
                        *error.write() = Some(GOOGLE_OPEN_FAILED.to_string());
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = url;
                }
                Err(e) => *error.write() = Some(e),
            }
            *google_connecting.write() = false;
        });
    };

    let handle_sync_pull = move |_| {
        let api = BackendApiService::new(Some(config.read().clone()));
        *syncing.write() = true;
        *error.write() = None;
        *sync_result.write() = None;
        spawn(async move {
            match api.sync_pull().await {
                Ok(pulled) => *sync_result.write() = Some(pulled),
                Err(e) => *error.write() = Some(e),
            }
            *syncing.write() = false;
        });
    };

    // Load stored settings on mount
    let load_context = user_context.clone();
    use_effect(move || {
        let user_context = load_context.clone();
        spawn(async move {
            match user_context.load_settings().await {
                Ok(settings) => {
                    email.set(field_from(&settings, KEY_EMAIL));
                    supabase_url.set(field_from(&settings, KEY_SUPABASE_URL));
                    anon_key.set(field_from(&settings, KEY_SUPABASE_ANON_KEY));
                    google_folder_id.set(field_from(&settings, KEY_GOOGLE_FOLDER_ID));
                }
                Err(e) => *error.write() = Some(e),
            }
        });
    });

    let sign_in_context = user_context.clone();
    let handle_sign_in = move |_| {
        let user_context = sign_in_context.clone();
        let login_email = email.read().trim().to_string();
        let login_password = password.read().trim().to_string();
        let login_anon_key = anon_key.read().trim().to_string();
        let login_url = supabase_url.read().trim().to_string();
        *signing_in.write() = true;
        *error.write() = None;
        *signed_in.write() = false;
        spawn(async move {
            if login_anon_key.is_empty() {
                *error.write() = Some(ERR_ANON_KEY_REQUIRED.to_string());
                *signing_in.write() = false;
                return;
            }
            match user_context
                .login_supabase(login_url, login_email, login_password, login_anon_key)
                .await
            {
                Ok(_) => *signed_in.write() = true,
                Err(e) => *error.write() = Some(e),
            }
            *signing_in.write() = false;
        });
    };

    let save_context = user_context.clone();
    let handle_save = move |_| {
        let user_context = save_context.clone();
        // Only send the fields owned by the visible tab — the anon key and
        // the login credentials never travel in the same request.
        let settings = match (*section.read(), *tab.read()) {
            (SettingsSection::Supabase, SettingsTab::AnonKey) => {
                serde_json::json!({
                    KEY_SUPABASE_URL: supabase_url.read().trim(),
                    KEY_SUPABASE_ANON_KEY: anon_key.read().trim(),
                })
            }
            (SettingsSection::Supabase, SettingsTab::Login) => serde_json::json!({
                KEY_EMAIL: email.read().clone(),
            }),
            (SettingsSection::Google, _) => serde_json::json!({
                KEY_GOOGLE_FOLDER_ID: google_folder_id.read().trim(),
            }),
            (SettingsSection::Sync, _) => serde_json::json!({}),
        };
        *loading.write() = true;
        *error.write() = None;
        *saved.write() = false;
        spawn(async move {
            match user_context.save_settings(settings).await {
                Ok(()) => *saved.write() = true,
                Err(e) => *error.write() = Some(e),
            }
            *loading.write() = false;
        });
    };

    rsx! {
        div {
            class: "max-w-4xl mx-auto py-8 px-4 sm:px-6 lg:px-8",

            div {
                class: "mb-6",
                h1 {
                    class: "text-2xl font-bold text-gray-900",
                    "{TITLE}"
                }
                p {
                    class: "mt-1 text-sm text-gray-600",
                    "{SUBTITLE}"
                }
            }

            div {
                class: "bg-white shadow-md rounded-lg p-6 space-y-4",

                div {
                    class: TABS_ROW_CLASS,
                    button {
                        class: if *section.read() == SettingsSection::Supabase {
                            TAB_ACTIVE_CLASS
                        } else {
                            TAB_IDLE_CLASS
                        },
                        onclick: move |_| *section.write() = SettingsSection::Supabase,
                        "{TAB_SUPABASE}"
                    }
                    button {
                        class: if *section.read() == SettingsSection::Google {
                            TAB_ACTIVE_CLASS
                        } else {
                            TAB_IDLE_CLASS
                        },
                        onclick: move |_| *section.write() = SettingsSection::Google,
                        "{TAB_GOOGLE}"
                    }
                    button {
                        class: if *section.read() == SettingsSection::Sync {
                            TAB_ACTIVE_CLASS
                        } else {
                            TAB_IDLE_CLASS
                        },
                        onclick: move |_| *section.write() = SettingsSection::Sync,
                        "{TAB_SYNC}"
                    }
                }

                div {
                    class: "flex gap-6",

                    if *section.read() == SettingsSection::Supabase {
                        nav {
                            class: NAV_CLASS,
                            div {
                                class: NAV_BOX_CLASS,
                                button {
                                    class: if *tab.read() == SettingsTab::Login {
                                        TAB_SUB_ACTIVE_CLASS
                                    } else {
                                        TAB_SUB_IDLE_CLASS
                                    },
                                    onclick: move |_| *tab.write() = SettingsTab::Login,
                                    "{TAB_LOGIN}"
                                }
                                button {
                                    class: if *tab.read() == SettingsTab::AnonKey {
                                        TAB_SUB_ACTIVE_CLASS
                                    } else {
                                        TAB_SUB_IDLE_CLASS
                                    },
                                    onclick: move |_| *tab.write() = SettingsTab::AnonKey,
                                    "{TAB_ANON_KEY}"
                                }
                            }
                        }
                    }

                    div {
                        class: CONTENT_CLASS,

                {match *section.read() {
                    SettingsSection::Supabase => rsx! {
                        {match *tab.read() {
                            SettingsTab::Login => rsx! {
                                div {
                                    div {
                                        label {
                                            class: "block text-sm font-medium text-gray-700 mb-1",
                                            r#for: "supabase-email",
                                            "{LABEL_EMAIL}"
                                        }
                                        input {
                                            id: "supabase-email",
                                            r#type: "email",
                                            class: INPUT_CLASS,
                                            placeholder: PLACEHOLDER_SUPABASE_EMAIL,
                                            value: "{email}",
                                            oninput: move |e| {
                                                *email.write() = e.value();
                                                *saved.write() = false;
                                            }
                                        }
                                    }

                                    div {
                                        label {
                                            class: "block text-sm font-medium text-gray-700 mb-1",
                                            r#for: "supabase-password",
                                            "{LABEL_PASSWORD}"
                                        }
                                        input {
                                            id: "supabase-password",
                                            r#type: "password",
                                            class: INPUT_CLASS,
                                            value: "{password}",
                                            oninput: move |e| {
                                                *password.write() = e.value();
                                                *saved.write() = false;
                                            }
                                        }
                                    }
                                }
                            },
                            SettingsTab::AnonKey => rsx! {
                                div {
                                    label {
                                        class: "block text-sm font-medium text-gray-700 mb-1",
                                        r#for: "supabase-url",
                                        "{LABEL_SUPABASE_URL}"
                                    }
                                    input {
                                        id: "supabase-url",
                                        r#type: "url",
                                        class: INPUT_CLASS,
                                        placeholder: "https://your-project.supabase.co",
                                        value: "{supabase_url}",
                                        oninput: move |e| {
                                            *supabase_url.write() = e.value();
                                            *saved.write() = false;
                                        }
                                    }
                                }

                                div {
                                    label {
                                        class: "block text-sm font-medium text-gray-700 mb-1",
                                        r#for: "supabase-anon-key",
                                        "{LABEL_ANON_KEY}"
                                    }
                                    input {
                                        id: "supabase-anon-key",
                                        r#type: "password",
                                        class: INPUT_CLASS,
                                        value: "{anon_key}",
                                        oninput: move |e| {
                                            *anon_key.write() = e.value();
                                            *saved.write() = false;
                                        }
                                    }
                                }
                            },
                        }}
                    },
                    SettingsSection::Sync => rsx! {
                        div {
                            class: "flex items-center gap-3",
                            button {
                                class: if *syncing.read() {
                                    format!("{BTN_CLASS} opacity-50 cursor-not-allowed")
                                } else {
                                    BTN_CLASS.to_string()
                                },
                                disabled: *syncing.read(),
                                onclick: handle_sync_pull,
                                if *syncing.read() { "{SYNC_RUNNING_TEXT}" } else { "{SYNC_BTN_TEXT}" }
                            }
                            if let Some(n) = sync_result.read().as_ref() {
                                span {
                                    class: "text-green-600 text-sm",
                                    {SYNC_DONE_FMT.replace("{n}", &n.to_string())}
                                }
                            }
                        }
                    },
                    SettingsSection::Google => rsx! {
                        div {
                            class: "flex items-center gap-3",
                            span {
                                class: if google_connected.read().as_ref() == Some(&true) {
                                    STATUS_TEXT_CONNECTED_CLASS
                                } else {
                                    STATUS_TEXT_CLASS
                                },
                                if let Some(connected) = google_connected.read().as_ref() {
                                    if *connected { "{GOOGLE_CONNECTED}" } else { "{GOOGLE_NOT_CONNECTED}" }
                                } else {
                                    { "..." }
                                }
                            }
                            button {
                                class: if *google_connecting.read() {
                                    format!("{BTN_CLASS} opacity-50 cursor-not-allowed")
                                } else if google_connected.read().as_ref() == Some(&true) {
                                    BTN_IDLE_CLASS.to_string()
                                } else {
                                    BTN_CLASS.to_string()
                                },
                                disabled: *google_connecting.read(),
                                onclick: handle_google_connect,
                                if *google_connecting.read() {
                                    "{GOOGLE_CONNECTING}"
                                } else if google_connected.read().as_ref() == Some(&true) {
                                    "{GOOGLE_RECONNECT_BTN}"
                                } else {
                                    "{GOOGLE_CONNECT_BTN}"
                                }
                            }
                        }

                        div {
                            label {
                                class: "block text-sm font-medium text-gray-700 mb-1",
                                r#for: "google-folder-id",
                                "{LABEL_GOOGLE_FOLDER_ID}"
                            }
                            input {
                                id: "google-folder-id",
                                class: INPUT_CLASS,
                                value: "{google_folder_id}",
                                oninput: move |e| {
                                    *google_folder_id.write() = e.value();
                                    *saved.write() = false;
                                }
                            }
                        }
                    },
                }}

                if *section.read() == SettingsSection::Supabase
                    && let Some(current) = session().as_ref()
                {
                    div {
                        class: "text-green-600 text-sm",
                        "{SIGNED_IN_AS} {current.user.email}"
                    }
                }

                if let Some(err) = error.read().as_ref() {
                    div {
                        class: "text-red-500 text-sm",
                        "{err}"
                    }
                }

                if *saved.read() {
                    div {
                        class: "text-green-600 text-sm",
                        "{SAVED_MSG}"
                    }
                }

                if *signed_in.read() {
                    div {
                        class: "text-green-600 text-sm",
                        "{SIGNED_IN_MSG}"
                    }
                }

                div {
                    class: "flex items-center gap-3",
                    if *section.read() == SettingsSection::Supabase
                        && *tab.read() == SettingsTab::Login
                    {
                        button {
                            class: if *signing_in.read() {
                                format!("{BTN_CLASS} opacity-50 cursor-not-allowed")
                            } else {
                                BTN_CLASS.to_string()
                            },
                            disabled: *signing_in.read(),
                            onclick: handle_sign_in,
                            if *signing_in.read() { "{SIGNING_IN_TEXT}" } else { "{SIGN_IN_BTN_TEXT}" }
                        }
                    }
                    if *section.read() != SettingsSection::Sync {
                        button {
                            class: if *loading.read() {
                                format!("{BTN_CLASS} opacity-50 cursor-not-allowed")
                            } else {
                                BTN_CLASS.to_string()
                            },
                            disabled: *loading.read(),
                            onclick: handle_save,
                            if *loading.read() { "Saving..." } else { "{SAVE_BTN_TEXT}" }
                        }
                    }
                }
                }
            }
        }
        }

        div {
            class: "mt-4",
            Link {
                to: Route::Dashboard {},
                class: "text-indigo-600 hover:text-indigo-500 text-sm font-medium",
                "Back to Dashboard"
            }
        }
    }
}
