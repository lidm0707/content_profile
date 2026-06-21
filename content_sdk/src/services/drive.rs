//! Google Drive image upload service (WASM only).
//!
//! Personal-tool client-side flow: injects a JS helper that uses Google Identity
//! Services to obtain a `drive.file` token, uploads bytes directly to the Drive
//! API via `multipart/related`, then sets `anyone/reader` so the returned
//! `thumbnail` URL is publicly embeddable in markdown.

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
    let gisPromise = null;

    function loadGis() {
        if (gisPromise) return gisPromise;
        gisPromise = new Promise(function (resolve, reject) {
            if (window.google && window.google.accounts && window.google.accounts.oauth2) {
                resolve();
                return;
            }
            const sel = 'script[src="' + GIS_SRC + '"]';
            const existing = document.querySelector(sel);
            if (existing) {
                if (window.google) { resolve(); return; }
                existing.addEventListener("load", resolve);
                existing.addEventListener("error", function () { reject(new Error("GIS load failed")); });
                return;
            }
            const s = document.createElement("script");
            s.src = GIS_SRC;
            s.async = true;
            s.defer = true;
            s.onload = resolve;
            s.onerror = function () { reject(new Error("GIS script failed to load")); };
            document.head.appendChild(s);
        });
        return gisPromise;
    }

    function getToken(clientId) {
        return loadGis().then(function () {
            return new Promise(function (resolve, reject) {
                const client = window.google.accounts.oauth2.initTokenClient({
                    client_id: clientId,
                    scope: SCOPE,
                    callback: function (resp) {
                        if (resp && resp.access_token) resolve(resp.access_token);
                        else reject(new Error("No access token in GIS response"));
                    },
                    error_callback: function (err) {
                        reject(new Error("OAuth error: " + JSON.stringify(err)));
                    }
                });
                client.requestAccessToken();
            });
        });
    }

    async function uploadAndShare(clientId, bytes, mime, name, folderId) {
        const token = await getToken(clientId);

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

    window.__gdriveHelper = { uploadAndShare: uploadAndShare };
})();
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_name = "__gdriveHelper")]
    static GDRIVE_HELPER: GdriveHelper;

    type GdriveHelper;

    #[wasm_bindgen(method, js_name = "uploadAndShare")]
    fn upload_and_share(
        this: &GdriveHelper,
        client_id: &str,
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

/// Upload image bytes to Google Drive and return a public thumbnail URL.
///
/// Format: `https://drive.google.com/thumbnail?id={fileId}&sz=w1000`
/// Requires `GOOGLE_OAUTH_CLIENT_ID` to be configured.
/// Pass a folder ID as the last argument to upload into a specific Drive folder.
pub async fn upload_image(
    client_id: &str,
    bytes: &[u8],
    mime: &str,
    name: &str,
    folder_id: Option<&str>,
) -> Result<String, String> {
    if client_id.is_empty() {
        return Err("GOOGLE_OAUTH_CLIENT_ID not configured".to_string());
    }
    if bytes.is_empty() {
        return Err("image bytes are empty".to_string());
    }

    inject_helper_once();

    let js_bytes = js_sys::Uint8Array::from(bytes);
    let folder_id_str = folder_id.unwrap_or("");
    let promise = GDRIVE_HELPER
        .with(|helper| helper.upload_and_share(client_id, &js_bytes, mime, name, folder_id_str));

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
