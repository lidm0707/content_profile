use dioxus::prelude::*;

/// Props for the [`Pagination`] component.
///
/// Single visual style used by both the Dashboard (server-side paging) and
/// the ContentTable (client-side paging) so the controls look the same
/// regardless of where they appear.
#[derive(Clone, PartialEq, Props)]
pub struct PaginationProps {
    /// 1-indexed current page number.
    pub current_page: usize,
    /// Total number of pages.
    pub total_pages: usize,
    /// Total number of items across all pages (shown as "of N results").
    pub total_items: usize,
    /// Page size (used to compute the "Showing X to Y of N results" label).
    pub page_size: usize,
    /// Called when the user clicks Previous or a page number.
    pub on_prev: EventHandler<()>,
    /// Called when the user clicks Next or a page number.
    pub on_next: EventHandler<()>,
}

/// Consistent pagination bar: Prev chevron + "Page X of Y" + Next chevron,
/// with a "Showing a to b of N results" label on the left.
///
/// Hidden entirely when `total_pages <= 1`.
#[component]
pub fn Pagination(props: PaginationProps) -> Element {
    if props.total_pages <= 1 {
        return rsx! {};
    }

    let current = props.current_page;
    let max_page = props.total_pages;
    let total = props.total_items;
    let page_size = props.page_size;

    // 1-indexed inclusive range shown on the current page.
    let from = ((current - 1) * page_size) + 1;
    let to = (current * page_size).min(total);

    rsx! {
        div {
            class: "flex items-center justify-between border-t border-gray-200 bg-gray-50 px-6 py-3",

            // Left: "Showing X to Y of N results"
            p {
                class: "hidden sm:block text-sm text-gray-700",
                "Showing ",
                span { class: "font-medium", "{from}" },
                " to ",
                span { class: "font-medium", "{to}" },
                " of ",
                span { class: "font-medium", "{total}" },
                " results"
            }

            // Right: Prev / "Page X of Y" / Next
            div {
                class: "inline-flex rounded-md shadow-sm -space-x-px",

                button {
                    disabled: current == 1,
                    onclick: move |_| props.on_prev.call(()),
                    class: if current == 1 {
                        "relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-gray-50 text-sm font-medium text-gray-300 cursor-not-allowed"
                    } else {
                        "relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                    },
                    svg {
                        class: "h-5 w-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            "stroke-width": 2,
                            d: "M15 19l-7-7 7-7"
                        }
                    }
                }

                span {
                    class: "relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium text-gray-700",
                    "Page {current} of {max_page}"
                }

                button {
                    disabled: current == max_page,
                    onclick: move |_| props.on_next.call(()),
                    class: if current == max_page {
                        "relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-gray-50 text-sm font-medium text-gray-300 cursor-not-allowed"
                    } else {
                        "relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                    },
                    svg {
                        class: "h-5 w-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            "stroke-width": 2,
                            d: "M9 5l7 7-7 7"
                        }
                    }
                }
            }
        }
    }
}
