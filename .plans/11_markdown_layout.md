# Plan: Beautiful Markdown Rendering

## Goal
Properly detect whitespace/indentation while parsing markdown to HTML and render
it with a beautiful, fully-styled layout in `content_detail.rs`.

## Context
Current state (verified):
- `content_sdk/src/utils/markdown.rs::render_markdown_to_html` blindly does
  `markdown.replace("\n", "  \n")` — corrupts fenced code blocks, forced hard
  breaks everywhere, no real indentation handling.
- `content_detail.rs` line 217 uses `class: "prose prose-sm max-w-none"` but
  **Tailwind v4 typography plugin is NOT installed** (verified: `prose` appears
  0 times in `tailwind.css`). So the rendered HTML is completely unstyled.
- `pulldown-cmark = "0.13.3"` is already a dep in both crates.

## Tasks
- [x] 0. Replace blind `\n → "  \n"` with real CommonMark newline + indent handling
      (track fenced code blocks / indented code blocks; preserve soft breaks only
      inside paragraphs).
- [x] 1. Wrap rendered HTML in a deterministic container class so styling does
      not depend on the missing `prose` plugin.
- [x] 2. Add scoped CSS for the markdown container covering headings, paragraphs,
      lists (ul/ol/task), code (inline + block), blockquotes, tables, hr, links,
      images — dark/light friendly, readable typography scale.
- [x] 3. Wire CSS into the app via `document::Stylesheet` asset so Dioxus injects it.
- [x] 4. Update `content_detail.rs` to use the new container class (drop dead
      `prose prose-sm` classes).
- [x] 5. Update unit tests in `markdown.rs` to cover: code block preservation,
      soft break, hard break, nested list indentation, GFM task list, table.
- [x] 6. `cargo check && cargo clippy` on `content_sdk` and `content_ui`.
- [x] 7. (Follow-up) Fix `content_form.rs::PreviewModeBodyEditor` — was still
      using dead `prose prose-sm max-w-none whitespace-pre-wrap` classes, so the
      preview pane never received the new `md-render` styles. Now uses
      `MARKDOWN_CONTAINER_CLASS`. Also pinned `white-space: pre; tab-size: 4;`
      on `.md-render pre` in CSS so leading whitespace in `<pre>` survives the
      cascade regardless of UA/preflight defaults.
- [x] 8. Regression test `test_code_block_leading_whitespace_survives` confirms
      the parser emits leading 7-/12-/16-space indents verbatim.
- [x] 9. Document the markdown rendering pipeline in `AGENTS.md` (new
      "# Markdown Rendering" section: how to render, do-not-use-`prose`,
      whitespace handling, CSS wiring) and `README.md` (Features bullet,
      Customization → Styling, Roadmap recently-completed).

## Notes
- Decision: stay with `pulldown-cmark` (already a dep) rather than swap to
  `comrak` to keep the change minimal and dependency-light, per AGENTS.md
  "no over engineer / lean code".
- Decision: write custom scoped CSS instead of adding `@tailwindcss/typography`
  because the project's `input.css` is just `@import "tailwindcss"` with no
  plugin pipeline, and a single self-contained stylesheet is simpler.
- Indent detection approach: walk the source line-by-line, know when we're
  inside a ` ``` ` fence, and only inject hard-break markers (`  \n`) for
  soft line breaks that occur *outside* code and *outside* list item
  continuation lines.
