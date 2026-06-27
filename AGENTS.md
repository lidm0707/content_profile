You are an expert [0.7 Dioxus](https://dioxuslabs.com/learn/0.7) assistant. Dioxus 0.7 changes every api in dioxus. Only use this up to date documentation. `cx`, `Scope`, and `use_state` are gone

Provide concise code examples with detailed descriptions

# Dioxus Dependency

You can add Dioxus to your `Cargo.toml` like this:

```toml
[dependencies]
dioxus = { version = "0.7.1" }

[features]
default = ["web", "webview", "server"]
web = ["dioxus/web"]
webview = ["dioxus/desktop"]
server = ["dioxus/server"]
```

# Launching your application

You need to create a main function that sets up the Dioxus runtime and mounts your root component.

```rust
use dioxus::prelude::*;

fn main() {
	dioxus::launch(App);
}

#[component]
fn App() -> Element {
	rsx! { "Hello, Dioxus!" }
}
```

Then serve with `dx serve`:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
dx serve
```

# UI with RSX

```rust
rsx! {
	div {
		class: "container", // Attribute
		color: "red", // Inline styles
		width: if condition { "100%" }, // Conditional attributes
		"Hello, Dioxus!"
	}
	// Prefer loops over iterators
	for i in 0..5 {
		div { "{i}" } // use elements or components directly in loops
	}
	if condition {
		div { "Condition is true!" } // use elements or components directly in conditionals
	}

	{children} // Expressions are wrapped in brace
	{(0..5).map(|i| rsx! { span { "Item {i}" } })} // Iterators must be wrapped in braces
}
```

# Assets

The asset macro can be used to link to local files to use in your project. All links start with `/` and are relative to the root of your project.

```rust
rsx! {
	img {
		src: asset!("/assets/image.png"),
		alt: "An image",
	}
}
```

## Styles

The `document::Stylesheet` component will inject the stylesheet into the `<head>` of the document

```rust
rsx! {
	document::Stylesheet {
		href: asset!("/assets/styles.css"),
	}
}
```

# Components

Components are the building blocks of apps

* Component are functions annotated with the `#[component]` macro.
* The function name must start with a capital letter or contain an underscore.
* A component re-renders only under two conditions:
	1.  Its props change (as determined by `PartialEq`).
	2.  An internal reactive state it depends on is updated.

```rust
#[component]
fn Input(mut value: Signal<String>) -> Element {
	rsx! {
		input {
            value,
			oninput: move |e| {
				*value.write() = e.value();
			},
			onkeydown: move |e| {
				if e.key() == Key::Enter {
					value.write().clear();
				}
			},
		}
	}
}
```

Each component accepts function arguments (props)

* Props must be owned values, not references. Use `String` and `Vec<T>` instead of `&str` or `&[T]`.
* Props must implement `PartialEq` and `Clone`.
* To make props reactive and copy, you can wrap the type in `ReadOnlySignal`. Any reactive state like memos and resources that read `ReadOnlySignal` props will automatically re-run when the prop changes.

# State

A signal is a wrapper around a value that automatically tracks where it's read and written. Changing a signal's value causes code that relies on the signal to rerun.

## Local State

The `use_signal` hook creates state that is local to a single component. You can call the signal like a function (e.g. `my_signal()`) to clone the value, or use `.read()` to get a reference. `.write()` gets a mutable reference to the value.

Use `use_memo` to create a memoized value that recalculates when its dependencies change. Memos are useful for expensive calculations that you don't want to repeat unnecessarily.

```rust
#[component]
fn Counter() -> Element {
	let mut count = use_signal(|| 0);
	let mut doubled = use_memo(move || count() * 2); // doubled will re-run when count changes because it reads the signal

	rsx! {
		h1 { "Count: {count}" } // Counter will re-render when count changes because it reads the signal
		h2 { "Doubled: {doubled}" }
		button {
			onclick: move |_| *count.write() += 1, // Writing to the signal rerenders Counter
			"Increment"
		}
		button {
			onclick: move |_| count.with_mut(|count| *count += 1), // use with_mut to mutate the signal
			"Increment with with_mut"
		}
	}
}
```

## Context API

The Context API allows you to share state down the component tree. A parent provides the state using `use_context_provider`, and any child can access it with `use_context`

```rust
#[component]
fn App() -> Element {
	let mut theme = use_signal(|| "light".to_string());
	use_context_provider(|| theme); // Provide a type to children
	rsx! { Child {} }
}

#[component]
fn Child() -> Element {
	let theme = use_context::<Signal<String>>(); // Consume the same type
	rsx! {
		div {
			"Current theme: {theme}"
		}
	}
}
```

## Context type must match exactly (Memo vs Signal)

`use_context::<T>()` looks up by **exact type**. If you provide a `Memo<T>` (the result of `use_memo`), consumers MUST read it as `Memo<T>` — not `Signal<T>`. They are distinct types and are NOT interchangeable, even though both implement `Readable` and deref to `T`.

This is an easy mistake because `use_memo` is the natural way to build a derived config/value, and it feels like "just a signal".

```rust
// ❌ Provider: stored as Memo<Config>
let config = use_memo(move || Config::new(/* ... */));
use_context_provider(move || config);

// ❌ Consumer panics at runtime: "Could not find context Signal<Config>"
let config = use_context::<Signal<Config>>();

// ✅ Consumer must use the exact type that was provided
let config = use_context::<Memo<Config>>();
```

The runtime panic (`Could not find context ...`) gives you the type name it was looking for — use that to pick the matching provider type.

If you need a writable signal in the consumer, either:
- provide a `Signal<T>` from the start (and update it via `use_effect`), or
- derive a new local `Signal<T>` inside the consumer from the `Memo<T>` value.

# Async

For state that depends on an asynchronous operation (like a network request), Dioxus provides a hook called `use_resource`. This hook manages the lifecycle of the async task and provides the result to your component.

* The `use_resource` hook takes an `async` closure. It re-runs this closure whenever any signals it depends on (reads) are updated
* The `Resource` object returned can be in several states when read:
1. `None` if the resource is still loading
2. `Some(value)` if the resource has successfully loaded

```rust
let mut dog = use_resource(move || async move {
	// api request
});

match dog() {
	Some(dog_info) => rsx! { Dog { dog_info } },
	None => rsx! { "Loading..." },
}
```

# Routing

All possible routes are defined in a single Rust `enum` that derives `Routable`. Each variant represents a route and is annotated with `#[route("/path")]`. Dynamic Segments can capture parts of the URL path as parameters by using `:name` in the route string. These become fields in the enum variant.

The `Router<Route> {}` component is the entry point that manages rendering the correct component for the current URL.

You can use the `#[layout(NavBar)]` to create a layout shared between pages and place an `Outlet<Route> {}` inside your layout component. The child routes will be rendered in the outlet.

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
	#[layout(NavBar)] // This will use NavBar as the layout for all routes
		#[route("/")]
		Home {},
		#[route("/blog/:id")] // Dynamic segment
		BlogPost { id: i32 },
}

#[component]
fn NavBar() -> Element {
	rsx! {
		a { href: "/", "Home" }
		Outlet<Route> {} // Renders Home or BlogPost
	}
}

#[component]
fn App() -> Element {
	rsx! { Router::<Route> {} }
}
```

```toml
dioxus = { version = "0.7.1", features = ["router"] }
```

# Fullstack

Fullstack enables server rendering and ipc calls. It uses Cargo features (`server` and a client feature like `web`) to split the code into a server and client binaries.

```toml
dioxus = { version = "0.7.1", features = ["fullstack"] }
```

## Server Functions

Use the `#[post]` / `#[get]` macros to define an `async` function that will only run on the server. On the server, this macro generates an API endpoint. On the client, it generates a function that makes an HTTP request to that endpoint.

```rust
#[post("/api/double/:path/&query")]
async fn double_server(number: i32, path: String, query: i32) -> Result<i32, ServerFnError> {
	tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	Ok(number * 2)
}
```

## Hydration

Hydration is the process of making a server-rendered HTML page interactive on the client. The server sends the initial HTML, and then the client-side runs, attaches event listeners, and takes control of future rendering.

### Errors
The initial UI rendered by the component on the client must be identical to the UI rendered on the server.

* Use the `use_server_future` hook instead of `use_resource`. It runs the future on the server, serializes the result, and sends it to the client, ensuring the client has the data immediately for its first render.
* Any code that relies on browser-specific APIs (like accessing `localStorage`) must be run *after* hydration. Place this code inside a `use_effect` hook.

# Google Drive Image Upload

Personal-tool client-side flow. Upload images directly from the editor to Google Drive via Google Identity Services (GIS) OAuth — no backend needed.

## Architecture

- `content_sdk::services::drive` (wasm32 only) injects a JS helper script into `<head>` that loads GIS, requests a `drive.file` token, uploads bytes via `multipart/related`, sets `anyone/reader`, and returns a `thumbnail` URL.
- `content_ui` calls `upload_image(client_id, bytes, mime, name, folder_id)` from the content form's image button. Pass `None` for `folder_id` to upload to Drive root, or a folder ID string to target a specific folder.
- Config: `GOOGLE_OAUTH_CLIENT_ID` and `GOOGLE_DRIVE_FOLDER_ID` env vars wired through `build.rs` → `Config::google_oauth_client_id`, `Config::google_drive_folder_id`.

## Setup

See `README.md` → "Configure Google Drive (Image Upload)". Requires:
1. Google Cloud project with Drive API enabled.
2. OAuth Client ID (Web application) with authorized JS origins.
3. `.env`: `GOOGLE_OAUTH_CLIENT_ID=...apps.googleusercontent.com`.

## JS Interop Pattern

When you need Promise-based JS APIs from Rust/WASM:
1. Inject a `<script>` with a self-executing IIFE that exposes a global object (`window.__helper = { method: ... }`).
2. Bind with `#[wasm_bindgen(thread_local_v2, js_name = "__helper")]` static + `#[wasm_bindgen(method)]` fn.
3. Call via `HELPER.with(|h| h.method(...))` and await the returned `js_sys::Promise` with `JsFuture::from(promise).await`.

## File Upload in Dioxus 0.7

- `onchange` on `<input type="file">` gives `Event<FormData>`.
- `e.files()` returns `Vec<FileData>` (not Option).
- `FileData::read_bytes().await` returns `bytes::Bytes`.
- Use `bytes.as_ref()` to get `&[u8]`.

# Markdown Rendering

Markdown → HTML lives in `content_sdk::utils::markdown::render_markdown_to_html`.
It is the **single** render path used by both `ContentDetail` and the form's
preview pane.

## How to render markdown

```rust
use content_sdk::utils::{MARKDOWN_CONTAINER_CLASS, render_markdown_to_html};

let html = render_markdown_to_html(&body);

rsx! {
    div {
        class: "{MARKDOWN_CONTAINER_CLASS}",
        dangerous_inner_html: html,
    }
}
```

**Always** wrap the output in a container using `MARKDOWN_CONTAINER_CLASS`
(`md-render`). The styles for headings, lists, tables, blockquotes, code
blocks, etc. are scoped under `.md-render` in `content_ui/assets/markdown.css`.

## Do NOT use Tailwind `prose` classes

This project does **not** install the `@tailwindcss/typography` plugin, so
`class: "prose prose-sm max-w-none"` does nothing — the rendered HTML ends up
completely unstyled. Use `MARKDOWN_CONTAINER_CLASS` instead.

## Whitespace / indentation handling

The renderer walks the `pulldown-cmark` event stream and converts
`SoftBreak → HardBreak` **only outside** code blocks, so:

- In prose, a single `\n` becomes `<br>` (the author's line wraps survive).
- In fenced (```) or indented (4-space) code blocks, newlines and leading
  whitespace are preserved verbatim.
- GFM tables, strikethrough, and task lists are enabled.

The CSS pins `white-space: pre; tab-size: 4;` on `.md-render pre` — this is
load-bearing. Without it, leading whitespace inside `<pre>` can collapse
under the Tailwind v4 preflight cascade. Do not remove.

## Adding the stylesheet

`markdown.css` is wired in `content_ui/src/app.rs`:

```rust
const MARKDOWN_CSS: Asset = asset!("/assets/markdown.css");

rsx! {
    document::Stylesheet { href: MARKDOWN_CSS }
}
```

If you add new markdown element styles, edit `content_ui/assets/markdown.css`
under the `.md-render` selector.
