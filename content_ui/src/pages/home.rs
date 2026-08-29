use crate::routes::Route;
use dioxus::prelude::*;

const MASCOT: Asset = asset!("/assets/mascot.png");

/// Hero section component — 和風 landing: dark purple, orange sun accent,
/// vertical Japanese caption, mascot artwork.
#[component]
fn HeroSection() -> Element {
    rsx! {
        div {
            class: "relative overflow-hidden",
            style: "background: linear-gradient(160deg, #14101f 0%, #1e1730 55%, #2a1a33 100%);",

            // Orange sun disc behind the content, like a rising 日の丸.
            div {
                class: "pointer-events-none absolute -top-24 -right-24 h-96 w-96 rounded-full opacity-20",
                style: "background: radial-gradient(circle, #ff8c3b 0%, transparent 70%);",
            }

            div {
                class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16 md:py-24 grid grid-cols-1 lg:grid-cols-2 gap-12 items-center",

                // Left — copy.
                div {
                    class: "flex gap-6",

                    // Vertical Japanese caption, classic 和風 side-title.
                    div {
                        class: "hidden sm:flex flex-col items-center gap-4",
                        div {
                            class: "jp-vertical text-lg text-gray-400 select-none",
                            "コンテンツ管理"
                        }
                        div {
                            class: "jp-accent-bar w-1 h-24 rounded-full",
                        }
                    }

                    div {
                        p {
                            class: "text-sm font-semibold tracking-widest uppercase text-orange-400",
                            "ようこそ · Welcome"
                        }

                        h1 {
                            class: "mt-3 text-4xl tracking-tight font-extrabold text-gray-900 sm:text-5xl md:text-6xl",

                            span {
                                class: "block",
                                "Content Management System"
                            }

                            span {
                                class: "block text-orange-400",
                                "Powered by Dioxus & Supabase"
                            }
                        }

                        p {
                            class: "mt-6 text-base text-gray-500 sm:text-lg max-w-xl",
                            "Rust の Dioxus フレームワークと Supabase で作られた、モダンで高速なコンテンツ管理。Create, edit, and manage your content with a calm, minimal interface."
                        }

                        div {
                            class: "mt-8 flex flex-wrap gap-4",

                            Link {
                                to: Route::Dashboard {},
                                class: "flex items-center justify-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-white bg-orange-500 hover:bg-orange-600 md:px-10",
                                "Go to Dashboard"
                            }

                            button {
                                class: "flex items-center justify-center px-8 py-3 border border-purple-400/40 text-base font-medium rounded-md text-purple-200 bg-purple-500/10 hover:bg-purple-500/20 md:px-10",
                                "Learn More"
                            }
                        }
                    }
                }

                // Right — mascot artwork in a framed card.
                div {
                    class: "relative flex justify-center",

                    div {
                        class: "relative rounded-2xl p-2",
                        style: "background: linear-gradient(160deg, #ff8c3b, #7c5cff);",

                        div {
                            class: "rounded-xl overflow-hidden",
                            style: "background: #1e1730;",

                            img {
                                class: "h-80 w-80 md:h-96 md:w-96 object-cover",
                                src: "{MASCOT}",
                                alt: "マスコット — crab mascot",
                            }
                        }
                    }

                    // Vertical signature beside the frame.
                    div {
                        class: "jp-vertical absolute -right-8 top-4 text-sm text-gray-500 select-none",
                        "かにさま"
                    }
                }
            }
        }
    }
}

/// Features section component
#[component]
fn FeaturesSection() -> Element {
    const FEATURE_TITLE: &str = "Features · 機能";
    const FEATURE_SUBTITLE: &str = "Everything you need to manage your content";

    rsx! {
        div {
            class: "py-16",
            style: "background: var(--tp-bg);",

            div {
                class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",

                div {
                    class: "lg:text-center",

                    h2 {
                        class: "text-base text-orange-400 font-semibold tracking-widest uppercase",
                        "{FEATURE_TITLE}"
                    }

                    p {
                        class: "mt-2 text-3xl leading-8 font-extrabold tracking-tight text-gray-900 sm:text-4xl",
                        "{FEATURE_SUBTITLE}"
                    }
                }

                div {
                    class: "mt-12",

                    dl {
                        class: "space-y-10 md:space-y-0 md:grid md:grid-cols-3 md:gap-x-8 md:gap-y-10",

                        FeatureItem {
                            icon: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
                            title: "Easy Content Creation · 作成",
                            description: "Create and edit content with an intuitive interface. Supports rich text editing and automatic slug generation."
                        }

                        FeatureItem {
                            icon: "M13 10V3L4 14h7v7l9-11h-7z",
                            title: "Fast Performance · 高速",
                            description: "Built with Rust and Dioxus for lightning-fast performance. Supabase provides instant database access."
                        }

                        FeatureItem {
                            icon: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
                            title: "Content Management · 管理",
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
                    class: "flex items-center justify-center h-12 w-12 rounded-md text-white",
                    style: "background: linear-gradient(135deg, var(--tp-accent), var(--tp-purple));",

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
