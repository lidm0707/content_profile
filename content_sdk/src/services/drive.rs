//! Google Drive image upload service (WASM only).
//!
//! Personal-tool client-side flow: preloads Google Identity Services on app
//! init, acquires a `drive.file` OAuth token synchronously within a user
//! gesture, then uploads bytes directly to the Drive API via `multipart/related`
//! and sets `anyone/reader` so the returned `thumbnail` URL is embeddable.
//!
//! Why split into two phases? Browsers only allow popups (the OAuth consent
//! window) to open inside a synchronous user-gesture handler. If the gesture
//! expires (e.g. after `await file_picker` or `await load_gis`), the browser
//! blocks the popup with `popup_failed_to_open`. So we MUST call
//! `requestAccessToken` synchronously in the click handler, store the
//! in-flight Promise, and only then open the file picker once the token
//! has arrived.

#![cfg(target_arch = "wasm32")]

use std::sync::OnceLock;

use tracing::{debug, error, info};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

const HELPER_SCRIPT: &str = r#"
(function () {
    if (window.__gdriveHelper) return;
    const GIS_SRC = "https://accounts.google.com/gsi/client";
    const SCOPE = "https://www.googleapis.com/auth/drive.file";
    const UPLOAD_URL = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";
    const PERMS_BASE = "https://www.googleapis.com/drive/v3/files";

    let gisReady = !!(window.google && window.google.accounts && window.google.accounts.oauth2);
    let gisPromise = null;

    function preloadGis() {
        if (gisReady) return Promise.resolve();
        if (gisPromise) return gisPromise;
        gisPromise = new Promise(function (resolve, reject) {
            if (window.google && window.google.accounts && window.google.accounts.oauth2) {
                gisReady = true;
                resolve();
                return;
            }
            const sel = 'script[src="' + GIS_SRC + '"]';
            const existing = document.querySelector(sel);
            if (existing) {
                existing.addEventListener("load", function () { gisReady = true; resolve(); });
                existing.addEventListener("error", function () { reject(new Error("GIS load failed")); });
                return;
            }
            const s = document.createElement("script");
            s.src = GIS_SRC;
            s.async = true;
            s.defer = true;
            s.onload = function () { gisReady = true; resolve(); };
            s.onerror = function () { reject(new Error("GIS script failed to load")); };
            document.head.appendChild(s);
        });
        return gisPromise;
    }

    // Synchronously opens the OAuth popup inside the caller's user gesture.
    // MUST be called from a synchronous gesture handler (click), NOT from an
    // awaited async task — otherwise the browser blocks the popup.
    function acquireToken(clientId) {
        if (!gisReady) {
            // Kick off load so a later retry succeeds, but reject this call.
            preloadGis();
            return Promise.reject(new Error("GIS not yet loaded — retry in a moment"));
        }
        return new Promise(function (resolve, reject) {
            let client;
            try {
                client = window.google.accounts.oauth2.initTokenClient({
                    client_id: clientId,
                    scope: SCOPE,
                    // FedCM mode: the browser shows a native sign-in dialog
                    // instead of a popup. Required now that Google's popup
                    // sends COOP (Chrome blocks GIS's window.closed polling
                    // with "Cross-Origin-Opener-Policy policy would block the
                    // window.closed call") and Firefox blocks the popup for
                    // lack of user activation.
                    use_fedcm: true,
                    callback: function (resp) {
                        if (resp && resp.access_token) resolve(resp.access_token);
                        else reject(new Error("No access token in GIS response: " + JSON.stringify(resp)));
                    },
                    error_callback: function (err) {
                        reject(new Error("OAuth error: " + JSON.stringify(err)));
                    }
                });
            } catch (e) {
                reject(new Error("initTokenClient failed: " + (e && e.message ? e.message : String(e))));
                return;
            }
            try {
                client.requestAccessToken();
            } catch (e) {
                reject(new Error("requestAccessToken failed: " + (e && e.message ? e.message : String(e))));
            }
        });
    }

    async function uploadWithToken(token, bytes, mime, name, folderId) {
        const boundary = "gdrive_boundary_" + Math.random().toString(36).slice(2);
        const meta = { name: name };
        if (folderId) meta.parents = [folderId];
        const metaJson = JSON.stringify(meta);
        const head =
            "--" + boundary + "\r\n" +
            "Content-Type: application/json; charset=UTF-8\r\n\r\n" +
            metaJson + "\r\n" +
            "--" + boundary + "\r\n" +
            "Content-Type: " + mime + "\r\n\r\n";
        const tail = "\r\n--" + boundary + "--\r\n";

        const headBuf = new TextEncoder().encode(head);
        const tailBuf = new TextEncoder().encode(tail);
        const body = new Blob([headBuf, bytes, tailBuf], { type: "multipart/related; boundary=" + boundary });

        const uploadResp = await fetch(UPLOAD_URL, {
            method: "POST",
            headers: {
                "Authorization": "Bearer " + token,
                "Content-Type": "multipart/related; boundary=" + boundary
            },
            body: body
        });

        if (!uploadResp.ok) {
            const t = await uploadResp.text();
            throw new Error("Drive upload failed (" + uploadResp.status + "): " + t);
        }

        const data = await uploadResp.json();
        if (!data.id) throw new Error("Drive upload response missing file id");

        const permResp = await fetch(PERMS_BASE + "/" + data.id + "/permissions", {
            method: "POST",
            headers: {
                "Authorization": "Bearer " + token,
                "Content-Type": "application/json"
            },
            body: JSON.stringify({ role: "reader", type: "anyone" })
        });
        if (!permResp.ok) {
            console.warn("[gdrive] permission set failed:", await permResp.text());
        }

        return "https://drive.google.com/thumbnail?id=" + data.id + "&sz=w1000";
    }

    window.__gdriveHelper = {
        preloadGis: preloadGis,
        acquireToken: acquireToken,
        uploadWithToken: uploadWithToken
    };
})();
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_name = "__gdriveHelper")]
    static GDRIVE_HELPER: GdriveHelper;

    type GdriveHelper;

    #[wasm_bindgen(method, js_name = "preloadGis")]
    fn preload_gis(this: &GdriveHelper);

    #[wasm_bindgen(method, js_name = "acquireToken")]
    fn acquire_token(this: &GdriveHelper, client_id: &str) -> js_sys::Promise;

    #[wasm_bindgen(method, js_name = "uploadWithToken")]
    fn upload_with_token(
        this: &GdriveHelper,
        token: &str,
        bytes: &js_sys::Uint8Array,
        mime: &str,
        name: &str,
        folder_id: &str,
    ) -> js_sys::Promise;
}

static HELPER_INJECTED: OnceLock<()> = OnceLock::new();

fn inject_helper_once() {
    if HELPER_INJECTED.get().is_some() {
        return;
    }

    let Some(window) = web_sys::window() else {
        error!("no global window — cannot inject GIS helper");
        return;
    };
    let Some(document) = window.document() else {
        error!("no document — cannot inject GIS helper");
        return;
    };

    let Ok(script) = document.create_element("script") else {
        error!("create helper script failed");
        return;
    };
    script.set_text_content(Some(HELPER_SCRIPT));
    if let Some(head) = document.head()
        && let Err(e) = head.append_child(&script)
    {
        error!("append helper script failed: {e:?}");
        return;
    }
    let _ = HELPER_INJECTED.set(());
    info!("Google Drive helper script injected");
}

/// Preload the Google Identity Services script so it is ready by the time the
/// user clicks the image-upload button. Idempotent. Safe to call on app init.
pub fn preload_gdrive() {
    inject_helper_once();
    GDRIVE_HELPER.with(|helper| helper.preload_gis());
    debug!("preload_gdrive: GIS preload requested");
}

/// Start OAuth token acquisition. The GIS popup opens **synchronously** during
/// this call, so this MUST be invoked directly inside a user-gesture handler
/// (e.g. an `onclick` closure body) — never from inside an awaited async task.
/// The returned Promise resolves later (after the user finishes the consent
/// flow) with the access_token string.
pub fn acquire_token_promise(client_id: &str) -> Result<js_sys::Promise, String> {
    if client_id.is_empty() {
        return Err("GOOGLE_OAUTH_CLIENT_ID not configured".to_string());
    }
    inject_helper_once();
    Ok(GDRIVE_HELPER.with(|helper| helper.acquire_token(client_id)))
}

/// Upload image bytes to Google Drive using a pre-acquired OAuth token.
/// Returns a public thumbnail URL: `https://drive.google.com/thumbnail?id=...&sz=w1000`.
pub async fn upload_with_token(
    token: &str,
    bytes: &[u8],
    mime: &str,
    name: &str,
    folder_id: Option<&str>,
) -> Result<String, String> {
    if token.is_empty() {
        return Err("OAuth token is empty".to_string());
    }
    if bytes.is_empty() {
        return Err("image bytes are empty".to_string());
    }

    inject_helper_once();

    let js_bytes = js_sys::Uint8Array::from(bytes);
    let folder_id_str = folder_id.unwrap_or("");
    let promise = GDRIVE_HELPER
        .with(|helper| helper.upload_with_token(token, &js_bytes, mime, name, folder_id_str));

    debug!(
        "awaiting Google Drive upload for {name} ({mime}, {} bytes)",
        bytes.len()
    );

    match JsFuture::from(promise).await {
        Ok(value) => {
            let url = value
                .as_string()
                .ok_or_else(|| "Drive helper returned non-string".to_string())?;
            info!("Google Drive upload succeeded: {url}");
            Ok(url)
        }
        Err(err) => {
            let msg = err.as_string().unwrap_or_else(|| format!("{err:?}"));
            error!("Google Drive upload failed: {msg}");
            Err(msg)
        }
    }
}
