use crate::contexts::UserContext;
use crate::models::Session;
use crate::routes::Route;
use dioxus::prelude::*;

/// Navigation bar component with links to different pages
#[component]
pub fn Navbar() -> Element {
    let user_context: UserContext = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    // Load saved session on mount if not already present
    use_effect(move || {
        if session().is_none()
            && UserContext::has_valid_saved_session()
            && let Ok(Some(saved_session)) = UserContext::load_saved_session()
        {
            session.set(Some(saved_session));
        }
    });

    rsx! {
        // Fixed navbar at the top
        nav {
            class: "bg-gray-100 border-b-2 border-gray-200 shadow-md fixed w-full top-0 z-50 h-16",
            div {
                class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",

                div {
                    class: "flex justify-between h-16",

                    // Left side - Logo and navigation links
                    div {
                        class: "flex items-center",

                        div {
                            class: "flex-shrink-0 flex items-center",

                            Link {
                                to: Route::Home {},
                                class: "text-2xl font-bold text-indigo-600 hover:text-indigo-800 transition-colors",
                                "Content CMS"
                            }
                        }

                        div {
                            class: "hidden sm:ml-8 sm:flex sm:space-x-8 ml-6",

                            Link {
                                to: Route::Home {},
                                class: "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium transition-colors",
                                "Home"
                            }

                            Link {
                                to: Route::Dashboard {},
                                class: "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium transition-colors",
                                "Dashboard"
                            }
                            Link {
                                to: Route::TagsList {},
                                class: "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium transition-colors",
                                "Tags"
                            }

                        }
                    }

                    // Right side - User info or login button
                    div {
                        class: "flex items-center space-x-4",

                        if session().is_some() {
                            // User info and logout when authenticated
                            div {
                                class: "flex items-center space-x-4",

                                // Show user email
                                span {
                                    class: "text-sm text-gray-700",
                                    "{session().as_ref().map(|s| s.user.email.clone()).unwrap_or_default()}"
                                }

                                // Create content button
                                Link {
                                    to: Route::ContentEdit { id: 0 },
                                    class: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors shadow-sm",

                                    svg {
                                        class: "-ml-1 mr-2 h-5 w-5",
                                        fill: "none",
                                        view_box: "0 0 24 24",
                                        stroke: "currentColor",

                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            "stroke-width": 2,
                                            d: "M12 6v6m0 0v6m0-6h6m-6 0H6"
                                        }
                                    }

                                    "Create Content"
                                }

                                // Logout button
                                button {
                                    onclick: move |_| {
                                        let user_context = user_context.clone();
                                        let mut session = session;
                                        let navigator = use_navigator();
                                        spawn(async move {
                                            let _ = user_context.logout().await;
                                            session.write().take();
                                            navigator.push(Route::Login {});
                                        });
                                    },
                                    class: "inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors shadow-sm",
                                    "Logout"
                                }
                            }
                        } else {
                            // Login button when not authenticated
                            Link {
                                to: Route::Login {},
                                class: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors shadow-sm",
                                "Login"
                            }
                        }
                    }
                }
            }
        }
    }
}
