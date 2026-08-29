# Plan: Table Sort + Editor Cursor Polish + Project Improvement Review

## Goal
1. Content table defaults to newest→oldest (created_at descending).
2. Verify/fix editor image & link insertion at the current cursor.
3. Review the project and produce an improvement plan.

## Tasks
- [x] 0. Table: initialise `sort_state` with `created_at` descending so content
      lists show newest first by default (content_table.rs). Also added id
      tiebreak for same-day items, and a `CREATED_AT_FIELD` const.
- [x] 1. Editor: audit cursor-aware insertion paths (format buttons, link,
      image upload, image-size modal). All paths already insert at the stored
      caret (cursor_pos tracked on input/click/keyup/blur; image upload
      snapshots pending_image_pos before the picker opens). Improvement:
      link button now selects the "Link text" placeholder after insertion so
      typing replaces it.
- [x] 2. `cargo check && cargo clippy` across the workspace — clean.
- [x] 3. Project review → wrote improvement plan `.plans/14_project_improvements.md`
      (decompose content_form.rs, unit tests for editor helpers, unify
      pagination layers, timestamp-based sorting, more Playwright specs,
      proxy logging, stray test_debug.rs).
