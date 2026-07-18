# Plan: Convert Content Cards to reslt_core Table

## Goal
Replace the card-based content list on the `ContentList` page with a sortable,
paginated table managed by `reslt_core` types and state shape.

## Context
- `content_ui/Cargo.toml` already declared `reslt_core` as a git dep, but only
  the column types were used.
- `components/content_table.rs` existed but was orphaned (not declared in
  `components/mod.rs`, never compiled).
- `pages/content_list.rs` rendered `ContentListComponent` (the card grid in
  `components/content_list.rs`).
- `content_sdk` exposes `get_all_content`, `get_paginated_content`, and
  `count_content` for data fetching.

## Tasks
- [x] 0. CSS fix — `.md-render table` was collapsing because of
      `display: block` + `width: 100%`. Replaced with
      `min-width: 100%` + `width: max-content` so markdown tables fill the
      container and scroll horizontally when wide. (markdown.css)
- [x] 1. Activate `content_table` module in `components/mod.rs` and re-export
      `ContentTable`.
- [x] 2. Rewrite `components/content_table.rs` to:
      - Use `reslt_core::SortState` + `PageState` signals for table state.
      - Keep the reslt `Col` / `PropCol` / `FieldAccessible` types for columns.
      - Project `Vec<Content>` -> `Vec<ContentRow>` (orphan-rule safe).
      - Client-side sort + pagination with a Tailwind footer bar
        (prev / numbered pages / next).
- [x] 3. Update `pages/content_list.rs` to render `ContentTable` instead of
      the card grid. Edit action routes to `Route::ContentEdit { id }` via a
      thin wrapper component (`ContentEditOnRow`) so the `Navigator` type
      doesn't have to be named through helper functions.
- [x] 4. **Swap the Dashboard's card list for `ContentTable` too** — the user
      was looking at `/dashboard`, not `/content/list/:tag`. Dashboard already
      does server-side pagination via `get_paginated_content`; the table now
      renders each 9-item server page. Dashboard's Prev/Next stays for server
      paging; table's internal pagination footer is hidden (9 < 10).
- [x] 5. Delete `components/content_list.rs` (cards) entirely — no longer
      used anywhere. Remove the `pub mod` + `pub use` from `components/mod.rs`.
- [x] 6. `cargo check && cargo clippy` — clean across the workspace.
- [x] 7. **Filter bug fixes** (user feedback: "I saw when filter have problem"):
      - Made `ContentListProps.tag` reactive via `ReadSignal<String>` so
        `use_resource` re-runs when the route param changes. Previously it
        captured `props.tag` once at mount, so tag -> tag navigation would
        show stale data.
      - Simplified redundant filter UI on `/content/list/:tag` — was showing
        the filter four times (h2 title + "Filtered" badge + subtitle + table
        pill). Removed the badge and subtitle; kept the h2 + table pill.
- [x] 8. **Tag pills disappear on filter page** (user: "when click tag icon
      tag is disappear"): extracted a reusable `TagPills` component
      (`components/tag_pills.rs`) and added a tag row on `/content/list/:tag`
      so users can switch tag -> tag inline. The active tag is highlighted.
      Dashboard uses the same component now.
- [x] 9. **Inconsistent pagination** (user: "paginate before filter and
      after is different"): extracted a shared `Pagination` component
      (`components/pagination.rs`) with the same chevron + "Page X of Y" +
      "Showing a to b of N results" style. Both the Dashboard (server-side
      paging) and the `ContentTable` (client-side paging) use it now.
      Removed the duplicated inline pagination markup from both.
- [x] 10. `cargo check && cargo clippy` — clean across the workspace.

## Root cause of 'I still see cards'
The user was viewing `/dashboard`, which had its own card rendering
(`ContentListComponent`) at `dashboard.rs:370`. The first pass only touched
`/content/list/:tag`. Fixed by swapping the dashboard's render call too.

## Notes / Design Decisions
- **Why not `use_table_provider`?** reslt_core's `use_table` hardcodes its
  `use_resource` reactive deps to `(current_page, items_per_page, sort)`. The
  page's `tag` filter wouldn't trigger a refetch, and there's no exposed
  `restart()` for a manual refresh. The page-driven fetch architecture stays;
  we use reslt_core's *types* (`SortState`, `PageState`, `Col`, `PropCol`,
  `FieldAccessible`) for the table's state shape and column model.
- **Two pagination layers on the Dashboard.** The dashboard already does
  server-side paging (`get_paginated_content`, 9 items/page). The table
  component also has its own client-side pagination (`PAGE_SIZE = 10`). With
  9-item server pages, the table's footer never shows (10 > 9), so the two
  layers don't visually clash. If `page_size` in the dashboard ever rises
  above 10, set `PAGE_SIZE` in `content_table.rs` higher or strip the table's
  internal pagination.
