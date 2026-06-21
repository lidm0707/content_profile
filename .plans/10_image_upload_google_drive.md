# Plan: Image Upload to Google Drive (Direct Frontend)

## Goal
Personal tool. User clicks the image button (🖼️) in the content editor → picks an image file → image uploads to their Google Drive → public URL is returned → inserted into the markdown body as `![filename](url)`.

## Architecture Decision: Direct Frontend (GIS OAuth)
Personal tool = single user = no backend needed. Pure WASM, matches the existing pure-client architecture.

Flow:
1. Load Google Identity Services script (`accounts.google.com/gsi/client`) on demand.
2. Request OAuth token with `drive.file` scope (popup, silent after first grant).
3. Upload image bytes directly to `googleapis.com/upload/drive/v3/files` (Drive API supports CORS).
4. Set `anyone/reader` permission on the file → makes it public.
5. Return `https://drive.google.com/thumbnail?id={fileId}&sz=w1000` and insert as markdown. (User-confirmed format — returns raw image bytes, works directly in `<img src>`.)

## Prerequisites (user must do, one-time, in Google Cloud Console)
1. Create / select a Google Cloud project.
2. Enable **Google Drive API**.
3. Configure **OAuth consent screen** (External, add yourself as test user).
4. Create **OAuth Client ID** (Web application type).
5. Add authorized JavaScript origins:
   - `http://localhost:8080` (dx serve dev)
   - prod origin (e.g. `https://your-domain`)
6. Copy the **Client ID** and set it as env var `GOOGLE_OAUTH_CLIENT_ID` in `.env`.

## Tasks

- [x] 0. Confirm approach with user (done: direct frontend, Google Drive, personal tool).
- [x] 1. Add `GOOGLE_OAUTH_CLIENT_ID` env wiring:
  - [x] 1a. Read in `content_ui/build.rs`, emit as `cargo:rustc-env`.
  - [x] 1b. Add `google_oauth_client_id` field to `content_sdk/src/utils/config.rs::Config` + `Config::new` param.
  - [x] 1c. Pass through in `content_ui/src/app.rs` when constructing `Config`.
- [x] 2. Add Google JS interop + Drive service in `content_sdk`:
  - [x] 2a. New module `content_sdk/src/services/drive.rs`.
  - [x] 2b. `load_gis_script()` — inject helper script into `<head>` via `web-sys`.
  - [x] 2c. `request_access_token` handled inside JS helper via GIS `initTokenClient`.
  - [x] 2d. `upload_image(client_id, bytes, mime, name) -> Result<String, String>` returns thumbnail URL.
  - [x] 2e. Registered module in `content_sdk/src/services/mod.rs`.
  - [x] 2f. Added `web-sys` / `wasm-bindgen` / `js-sys` / `wasm-bindgen-futures` features.
- [x] 3. Wire into content form UI (`content_ui/src/components/content_form.rs`):
  - [x] 3a. Replaced `handle_format_image` with `handle_trigger_image_upload` + hidden `<input type="file" accept="image/*">`.
  - [x] 3b. On file change, reads bytes via `FileData::read_bytes`, calls `drive_upload_image`.
  - [x] 3c. While uploading: disables toolbar, shows ⏳.
  - [x] 3d. On success: inserts `![{alt}]({url})` via `append_markdown` + `format_image`.
  - [x] 3e. On error: surfaces message via existing error_message signal.
- [x] 4. Validate:
  - [x] 4a. `cargo check` workspace (native + wasm32). ✅
  - [x] 4b. `cargo clippy` workspace (native + wasm32, zero warnings). ✅
  - [ ] 4c. Manual test with real Google OAuth Client ID (user provides).

## Notes
- The current image button is at `content_form.rs` L817 `handle_format_image`. Keep the `body` Signal and `append_markdown` helper.
- GIS script must be loaded once; cache a `OnceCell` / static to avoid re-injecting.
- Token client can be reused; GIS silently refreshes within the session after first grant.
- Returned URL format: `https://drive.google.com/thumbnail?id={fileId}&sz=w1000` (user-confirmed). Returns raw image bytes — works directly in `<img src>` / markdown images. `sz=w1000` caps width at 1000px. Still requires `anyone/reader` permission (set in 2d).
- Web environment only — guard `drive.rs` behind `#[cfg(target_arch = "wasm32")]` or the `web` feature so non-web builds don't break.
- Do NOT read `.env` from tooling. User sets `GOOGLE_OAUTH_CLIENT_ID` themselves.
