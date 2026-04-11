use crate::routes::Route;
use dioxus::prelude::*;

/// Hero section component
#[component]
fn HeroSection() -> Element {
    rsx! {
        div {
            class: "relative bg-white overflow-hidden",

            div {
                class: "max-w-7xl mx-auto",

                div {
                    class: "relative z-10 pb-8 bg-white sm:pb-16 md:pb-20 lg:max-w-3xl lg:w-full lg:pb-28 xl:pb-32",

                    main {
                        class: "mt-10 mx-auto max-w-7xl px-4 sm:mt-12 sm:px-6 md:mt-16 lg:mt-20 lg:px-8 xl:mt-28",

                        div {
                            class: "sm:text-center lg:text-left",

                            h1 {
                                class: "text-4xl tracking-tight font-extrabold text-gray-900 sm:text-5xl md:text-6xl",

                                span {
                                    class: "block xl:inline",
                                    "Content Management System"
                                }

                                span {
                                    class: "block text-indigo-600 xl:inline",
                                    "Powered by Dioxus & Supabase"
                                }
                            }

                            p {
                                class: "mt-3 text-base text-gray-500 sm:mt-5 sm:text-lg sm:max-w-xl sm:mx-auto md:mt-5 md:text-xl lg:mx-0 lg:max-w-2xl",
                                "Create, edit, and manage your content with a modern, responsive interface, built with Rust's Dioxus framework and Supabase's powerful backend."
                            }

                            div {
                                class: "mt-5 sm:mt-8 sm:flex sm:justify-center lg:justify-start",

                                div {
                                    class: "rounded-md shadow",

                                    Link {
                                        to: Route::Dashboard {},
                                        class: "w-full flex items-center justify-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 md:py-4 md:text-lg md:px-10",
                                        "Go to Dashboard"
                                    }
                                }

                                div {
                                    class: "mt-3 sm:mt-0 sm:ml-3",

                                    button {
                                        class: "w-full flex items-center justify-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 md:py-4 md:text-lg md:px-10",
                                        "Learn More"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div {
                class: "lg:absolute lg:inset-y-0 lg:right-0 lg:w-1/2",

                img {
                    class: "h-56 w-full object-cover sm:h-72 md:h-96 lg:w-full lg:h-full",
                    src: "https://images.unsplash.com/photo-1498050108023-c5249f4df085?ixlib=rb-1.2.1&auto=format&fit=crop&w=2012&q=80",
                    alt: "Content Management"
                }
            }
        }
    }
}

/// Features section component
#[component]
fn FeaturesSection() -> Element {
    const FEATURE_TITLE: &str = "Features";
    const FEATURE_SUBTITLE: &str = "Everything you need to manage your content";

    rsx! {
        div {
            class: "py-12 bg-white",

            div {
                class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",

                div {
                    class: "lg:text-center",

                    h2 {
                        class: "text-base text-indigo-600 font-semibold tracking-wide uppercase",
                        "{FEATURE_TITLE}"
                    }

                    p {
                        class: "mt-2 text-3xl leading-8 font-extrabold tracking-tight text-gray-900 sm:text-4xl",
                        "{FEATURE_SUBTITLE}"
                    }
                }

                div {
                    class: "mt-10",

                    dl {
                        class: "space-y-10 md:space-y-0 md:grid md:grid-cols-3 md:gap-x-8 md:gap-y-10",

                        FeatureItem {
                            icon: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
                            title: "Easy Content Creation",
                            description: "Create and edit content with an intuitive interface. Supports rich text editing and automatic slug generation."
                        }

                        FeatureItem {
                            icon: "M13 10V3L4 14h7v7l9-11h-7z",
                            title: "Fast Performance",
                            description: "Built with Rust and Dioxus for lightning-fast performance. Supabase provides instant database access."
                        }

                        FeatureItem {
                            icon: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
                            title: "Content Management",
                            description: "Organize your content with status tracking, version history, and comprehensive search capabilities."
                        }
                    }
                }
            }
        }
    }
}

/// Feature item component
#[component]
fn FeatureItem(icon: String, title: String, description: String) -> Element {
    rsx! {
        div {
            dt {
                div {
                    class: "flex items-center justify-center h-12 w-12 rounded-md bg-indigo-500 text-white",

                    svg {
                        class: "h-6 w-6",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",

                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            "stroke-width": 2,
                            d: "{icon}"
                        }
                    }
                }

                p {
                    class: "mt-5 text-lg leading-6 font-medium text-gray-900",
                    "{title}"
                }
            }

            dd {
                class: "mt-2 text-base text-gray-500",
                "{description}"
            }
        }
    }
}

/// Home page component - the landing page of the application
#[component]
pub fn Home() -> Element {
    rsx! {

        HeroSection {}

        FeaturesSection {}
    }
}
