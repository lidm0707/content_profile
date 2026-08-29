# Plan: Project Improvements (Review 2026-08-29)

## Goal
Follow-up improvements found while reviewing the workspace after the
table-sort / editor-cursor work. Ordered by value-to-effort.

## Findings
- `content_form.rs` is 1817 lines — the form, toolbar handlers, image upload
  flow, image-size modal, and tag modals all live in one file.
- Tests exist only in `content_sdk/src/utils` (markdown + mod). No tests in
  `content_ui`, `content_proxy`, or `supabase_client`. Pure logic like
  `insert_at_cursor`, `utf16_offset_to_byte`, `find_image_under_cursor`,
  `sort_rows` is easily unit-testable.
- `test_debug.rs` sits stray at the repo root — not part of any crate.
- Playwright smoke tests only cover `home` and `dashboard`; no spec for
  content list/table, edit form, or tags pages.
- Dashboard fetches server-side pages of 9 while the table pages client-side
  at 10 — two pagination layers that only avoid clashing by accident
  (see notes in plan 12).
- `created_at` in `ContentRow` is truncated to `%Y-%m-%d`, so same-day
  ordering falls back to the `id` tiebreak rather than true timestamps.

## Tasks
- [x] 0. Extract editor helpers from `content_form.rs` into an
      `content_ui/src/components/editor/` module: `cursor.rs` (insert-at-cursor
      + caret tracking + `ImageTarget`), `toolbar.rs`, `tags_ui.rs`,
      `image_size_modal.rs`. `content_form.rs` 1817 → 944 lines. No behavior
      change; check + clippy clean.
- [x] 1. Unit tests: 13 tests added — `cursor.rs` (`insert_at_cursor` splice/
      replace/append/out-of-range/empty, UTF-16↔byte roundtrip with emoji+CJK,
      clamping, `find_image_under_cursor` plain/#img=/none) and
      `content_table.rs` (`sort_rows` newest-first + id tiebreak, toggle,
      title). `cargo test -p content_ui` green.
- [x] 2. Unify pagination: `ContentTable` gained `show_pagination: bool`
      (default true); Dashboard passes `false` since it pages server-side.
      No more reliance on 9 < 10.
- [x] 3. Sort on real timestamps: `ContentRow.created_ts` (newtype `Ts` over
      `Option<DateTime<Utc>>` so `FieldAccessible`'s Display bound holds)
      drives `created_at` sorting; id remains tiebreak. New test verifies
      same-day ordering by time, not date string.
- [x] 4. Playwright spec `playwright_cli/tests/content_list.spec.ts`: mocks 2
      rows whose created_at order differs from id order; asserts default
      newest-first row order and that clicking the Created header flips to
      oldest-first. Parsed OK by playwright (`--list` shows 8 tests). Not yet
      RUN against the app — needs `docker compose up -d` + test image rebuild.
- [x] 5. Deleted stray `test_debug.rs` (referenced no crate; logic covered by
      content_sdk markdown tests).
- [x] 6. `content_proxy`: added `logging()` access log (method/path/ok|error)
      and overrode `fail_to_proxy` to respond with a JSON error body
      (`{"error":{"code":..,"message":"proxy upstream failure"}}`) instead of
      Pingora's plain-text default, so client toasts can parse failures.
- [x] 7. `cargo check && cargo clippy` clean across workspace after each step;
      `cargo test -p content_ui` 14/14 green.

## Notes
- Review date: 2026-08-29, after plan 13 work.
- **Bug fixed (user report, "fetch wasn't desc by created_at"):**
  `supabase_client::get_paginated` / `get_paginated_with_count` sent no
  `order` param, so PostgREST returned each offset/limit page in unspecified
  order — newest-first only held within a page. Both now send
  `order=created_at.desc` (`ORDER_PARAM`/`DEFAULT_ORDER` consts); local
  Office mode `get_paginated_content` sorts by `created_at` desc too.
- Keep the reslt_core *types* approach (plan 12 notes) — do not switch to
  `use_table_provider`; the page-driven fetch architecture is intentional.
- Remaining ideas (not scheduled): edit-form + tags Playwright specs, proxy
  integration test of the JSON error path, content_form.rs is still 944 lines
  (upload flow could move into `editor/image_upload.rs` next).
