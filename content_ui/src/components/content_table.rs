use crate::components::Pagination;
use std::rc::Rc;

use content_sdk::models::Content;
use dioxus::prelude::*;
use reslt_core::prelude::*;

/// Page size for the table. reslt_core's `PageState` defaults to 10; we keep
/// the same value here as a single source of truth.
const PAGE_SIZE: usize = 10;

/// Row model for the reslt table.
///
/// `Content` lives in `content_sdk` and `FieldAccessible` in `reslt_core`,
/// so neither is local to this crate — the orphan rule forbids implementing
/// the trait directly on `Content`. This lightweight view model mirrors the
/// display fields as plain `String`s (keeps `Eq` trivial and sorting stable).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, FieldAccessible)]
pub struct ContentRow {
    pub id: i32,
    pub title: String,
    pub status: String,
    pub sync_status: String,
    pub created_at: String,
}

impl ContentRow {
    /// Project a [`Content`] item into a table row, normalising optionals to
    /// display strings so every cell has a value.
    fn from_content(c: &Content) -> Self {
        ContentRow {
            id: c.id.unwrap_or(0),
            title: c.title.clone(),
            status: c.status.clone(),
            sync_status: if c.synced_at.is_some() {
                "SYNCED".to_string()
            } else {
                "LOCAL".to_string()
            },
            created_at: c
                .created_at
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "N/A".to_string()),
        }
    }
}

/// In-place sort helper — mirrors the field names exposed by `ContentRow`.
fn sort_rows(rows: &mut [ContentRow], field: &str, descending: bool) {
    rows.sort_by(|a, b| {
        let ord = match field {
            "id" => a.id.cmp(&b.id),
            "title" => a.title.cmp(&b.title),
            "status" => a.status.cmp(&b.status),
            "sync_status" => a.sync_status.cmp(&b.sync_status),
            "created_at" => a.created_at.cmp(&b.created_at),
            _ => std::cmp::Ordering::Equal,
        };
        if descending { ord.reverse() } else { ord }
    });
}

/// Tailwind classes for the status badge, keyed by status string.
fn status_badge_class(status: &str) -> &'static str {
    match status {
        "published" => "bg-green-100 text-green-800",
        "draft" => "bg-yellow-100 text-yellow-800",
        "archived" => "bg-gray-100 text-gray-800",
        _ => "bg-blue-100 text-blue-800",
    }
}

/// Tailwind classes for the sync badge.
fn sync_badge_class(sync: &str) -> &'static str {
    if sync == "SYNCED" {
        "bg-blue-100 text-blue-800"
    } else {
        "bg-gray-100 text-gray-600"
    }
}

/// Build the reslt column definitions, including custom cell renderers for the
/// status / sync badges and the Edit action.
///
/// Uses reslt's `Col`/`PropCol` types so the table structure is defined the
/// reslt way. The Edit cell calls `on_edit(row.id)` instead of navigating —
/// the parent shows a slide panel.
fn build_columns(on_edit: impl Fn(i32) + 'static) -> PropCol<ContentRow> {
    let status_action: Rc<dyn Fn(ContentRow) -> Element> = Rc::new(|row: ContentRow| {
        let cls = status_badge_class(&row.status);
        rsx! {
            span {
                class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {cls}",
                "{row.status.to_uppercase()}"
            }
        }
    });

    let sync_action: Rc<dyn Fn(ContentRow) -> Element> = Rc::new(|row: ContentRow| {
        let cls = sync_badge_class(&row.sync_status);
        rsx! {
            span {
                class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {cls}",
                "{row.sync_status}"
            }
        }
    });

    let edit_action: Rc<dyn Fn(ContentRow) -> Element> = {
        let on_edit = Rc::new(on_edit);
        Rc::new(move |row: ContentRow| {
            let on_edit = on_edit.clone();
            rsx! {
                button {
                    class: "text-indigo-600 hover:text-indigo-900 text-sm font-medium cursor-pointer",
                    onclick: move |_| on_edit(row.id),
                    "Edit \u{2192}"
                }
            }
        })
    };

    PropCol {
        cols: vec![
            Col {
                head: "Title".to_string(),
                index: "title".to_string(),
                class: None,
                action: None,
            },
            Col {
                head: "Status".to_string(),
                index: "status".to_string(),
                class: None,
                action: Some(status_action),
            },
            Col {
                head: "Sync".to_string(),
                index: "sync_status".to_string(),
                class: None,
                action: Some(sync_action),
            },
            Col {
                head: "Created".to_string(),
                index: "created_at".to_string(),
                class: None,
                action: None,
            },
            Col {
                head: "Actions".to_string(),
                index: "id".to_string(),
                class: Some("text-right".to_string()),
                action: Some(edit_action),
            },
        ],
    }
}

/// Props for [`ContentTable`].
///
/// `contents` is the data fetched by the parent via `content_sdk` — this
/// component does NO fetching of its own. State (sort + pagination) is
/// managed with reslt_core's `SortState` / `PageState` types so it stays
/// consistent with the rest of the reslt ecosystem.
#[derive(Clone, PartialEq, Props)]
pub struct ContentTableProps {
    pub contents: Vec<Content>,
    /// Optional active tag filter label shown in the table header.
    #[props(default = String::new())]
    pub active_filter: String,
    /// Called with a content id when the Edit action is clicked. The parent
    /// typically navigates to the edit route.
    pub on_edit: EventHandler<i32>,
}

/// Content table rendered with reslt's column types and state shape.
///
/// Data flow: **content_sdk fetches (parent) -> `Vec<Content>` -> this table**.
/// Sorting + pagination are handled locally with reslt_core state shapes;
/// no network calls happen here.
#[component]
pub fn ContentTable(props: ContentTableProps) -> Element {
    let on_edit = props.on_edit;
    let cols = build_columns(move |id| on_edit.call(id));

    // reslt_core state. We use its `SortState` / `PageState` shapes directly
    // so this component stays interoperable with reslt helpers if needed.
    let sort_state: Signal<SortState> = use_signal(SortState::default);
    let mut page_state: Signal<PageState> = use_signal(|| PageState {
        current_page: 0,
        items_per_page: PAGE_SIZE,
        total_items: 0,
    });

    // Project + sort the incoming content_sdk data into display rows.
    let mut rows: Vec<ContentRow> = props
        .contents
        .iter()
        .map(ContentRow::from_content)
        .collect();

    let sort = sort_state.read().clone();
    if let Some(field) = &sort.column
        && !field.is_empty()
    {
        sort_rows(&mut rows, field, sort.descending);
    }

    // Keep reslt_core's PageState in sync with the actual data length so
    // pagination controls render the correct number of pages.
    let total_items = rows.len();
    let (cur_page, cur_size, cur_total) = {
        let page = page_state.read();
        (page.current_page, page.items_per_page, page.total_items)
    };
    let items_per_page = cur_size.max(1);
    let total_pages = total_items.div_ceil(items_per_page);
    let current_page = cur_page.min(total_pages.saturating_sub(1));

    // Drop the write if nothing changed to avoid needless re-renders.
    if cur_total != total_items || cur_page != current_page {
        let mut w = page_state.write();
        w.total_items = total_items;
        w.current_page = current_page;
    }

    let start = current_page * items_per_page;
    let end = (start + items_per_page).min(total_items);
    let visible_rows: Vec<ContentRow> =
        rows.get(start..end).map(|s| s.to_vec()).unwrap_or_default();

    if rows.is_empty() {
        let filter_note = if props.active_filter.is_empty() {
            "No content found. Create your first content item!"
        } else {
            "No content matches this tag."
        };
        return rsx! {
            div {
                class: "text-center py-16 bg-white rounded-xl shadow-sm border border-gray-100",
                svg {
                    class: "mx-auto h-12 w-12 text-gray-300",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        "stroke-width": 1.5,
                        d: "M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                    }
                }
                p {
                    class: "mt-4 text-gray-500 text-lg",
                    "{filter_note}"
                }
            }
        };
    }

    let active_field = sort.column.clone().unwrap_or_default();
    let sort_desc = sort.descending;

    rsx! {
        div {
            class: "mt-8",

            div {
                class: "shadow-sm overflow-hidden border border-gray-200 rounded-xl bg-white",

                // Table header bar — count + active filter indicator.
                div {
                    class: "px-6 py-3 border-b border-gray-200 bg-gray-50 flex items-center justify-between",
                    p {
                        class: "text-sm text-gray-600",
                        span { class: "font-semibold text-gray-900", "{total_items}" },
                        " {row_count_label(total_items)}"
                    }
                    if !props.active_filter.is_empty() {
                        span {
                            class: "inline-flex items-center gap-1 text-xs font-medium text-indigo-700 bg-indigo-50 px-2 py-1 rounded-full",
                            svg {
                                class: "h-3 w-3",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    "stroke-width": 2,
                                    d: "M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                                }
                            }
                            "{props.active_filter}"
                        }
                    }
                }

                div {
                    class: "overflow-x-auto",
                    table {
                        // `w-full` forces the table to fill its container.
                        // Without it, `min-w-full` only kicks in when content
                        // is wider than the container — narrow content leaves
                        // the table collapsed to its intrinsic width.
                        class: "min-w-full w-full divide-y divide-gray-200",
                        thead {
                            tr { class: "bg-gray-50",
                                for col in cols.cols.iter() {
                                    {render_header(col, &active_field, sort_desc, sort_state)}
                                }
                            }
                        }
                        tbody {
                            class: "bg-white divide-y divide-gray-100",
                            for (i, row) in visible_rows.iter().enumerate() {
                                tr {
                                    class: if i % 2 == 0 { "hover:bg-indigo-50/40 transition-colors" } else { "bg-gray-50/40 hover:bg-indigo-50/40 transition-colors" },
                                    for col in cols.cols.iter() {
                                        {render_cell(col, row.clone())}
                                    }
                                }
                            }
                        }
                    }
                }

                {render_pagination(total_pages, current_page, items_per_page, total_items, page_state)}
            }
        }
    }
}

/// Pluralise "item"/"items" for the header count.
fn row_count_label(count: usize) -> &'static str {
    if count == 1 { "item" } else { "items" }
}

/// Clickable, sortable column header. Renders an arrow when active.
///
/// Takes the sort signal by value (`Signal` is `Copy`) so each header in the
/// loop gets its own copy to capture in its `onclick` handler.
fn render_header(
    col: &Col<ContentRow>,
    active_field: &str,
    sort_desc: bool,
    mut sort_state: Signal<SortState>,
) -> Element {
    let field = col.index.clone();
    let is_active = active_field == col.index;
    let arrow = if is_active {
        if sort_desc { " \u{25BC}" } else { " \u{25B2}" }
    } else {
        ""
    };
    let align = col.class.as_deref().unwrap_or("");

    rsx! {
        th {
            class: "px-6 py-3 text-left text-xs font-semibold text-gray-500 uppercase tracking-wider cursor-pointer select-none hover:text-gray-700 hover:bg-gray-100 transition-colors {align}",
            onclick: move |_| {
                let cur = sort_state.read().clone();
                let next = if cur.column.as_deref() == Some(field.as_str()) {
                    SortState {
                        column: Some(field.clone()),
                        descending: !cur.descending,
                    }
                } else {
                    SortState {
                        column: Some(field.clone()),
                        descending: false,
                    }
                };
                *sort_state.write() = next;
            },
            "{col.head}{arrow}"
        }
    }
}

/// Body cell — delegates to the column's custom `action` renderer when present,
/// otherwise falls back to the field value via `FieldAccessible`.
fn render_cell(col: &Col<ContentRow>, row: ContentRow) -> Element {
    if let Some(action) = &col.action {
        let action = action.clone();
        return rsx! {
            td {
                class: "px-6 py-4 whitespace-nowrap text-sm",
                {action(row)}
            }
        };
    }

    // Title column gets heavier weight for visual hierarchy.
    let is_title = col.index == "title";
    let value = row.get_field(&col.index).unwrap_or_default();
    rsx! {
        td {
            class: if is_title {
                "px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 max-w-xs truncate"
            } else {
                "px-6 py-4 whitespace-nowrap text-sm text-gray-500"
            },
            "{value}"
        }
    }
}

/// Footer pagination bar — uses the shared [`Pagination`] component so the
/// look matches the Dashboard's server-side paging. Reads/writes
/// reslt_core's `PageState` (0-indexed internally; converts to 1-indexed
/// for display).
fn render_pagination(
    total_pages: usize,
    current_page: usize,
    items_per_page: usize,
    total_items: usize,
    mut page_state: Signal<PageState>,
) -> Element {
    rsx! {
        Pagination {
            current_page: current_page + 1,
            total_pages,
            total_items,
            page_size: items_per_page,
            on_prev: move |_| {
                if current_page > 0 {
                    page_state.write().current_page = current_page - 1;
                }
            },
            on_next: move |_| {
                if current_page + 1 < total_pages {
                    page_state.write().current_page = current_page + 1;
                }
            },
        }
    }
}
