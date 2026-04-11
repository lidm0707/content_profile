# Dioxus 0.7 Skill Reference

## Quick Start

### Cargo.toml Setup

```toml
[dependencies]
dioxus = { version = "0.7.1" }

[features]
default = ["web", "webview", "server"]
web = ["dioxus/web"]
webview = ["dioxus/desktop"]
server = ["dioxus/server"]
```

### Basic Application

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

### Serve Application

```bash
# Install CLI (first time)
curl -sSL http://dioxus.dev/install.sh | sh

# Serve application
dx serve --web
```

## RSX Fundamentals

### Basic Structure

```rust
rsx! {
    div {
        class: "container",
        style: "color: red",
        "Hello, Dioxus!"
    }
}
```

### Conditional Rendering

```rust
rsx! {
    if condition {
        div { "Condition is true" }
    } else {
        div { "Condition is false" }
    }
    
    div {
        width: if condition { "100%" }
    }
}
```

### Loops

```rust
rsx! {
    // Preferred: Use loops directly
    for i in 0..5 {
        div { "{i}" }
    }
    
    // Iterators must be wrapped in braces
    {(0..5).map(|i| rsx! { span { "Item {i}" } })}
}
```

### Expressions

```rust
rsx! {
    {children}  // Wrapping expressions in braces
    
    {format!("Value: {}", value)}
    
    // Computed values
    {calculate_value()}
}
```

## RSX Anti-Patterns

### DON'T: Use match expressions in rsx!

```rust
// ❌ BAD - Match in RSX causes performance issues
rsx! {
    div {
        match state() {
            State::Loading => div { "Loading..." },
            State::Loaded { data } => div { "{data}" },
            State::Error { msg } => div { "Error: {msg}" },
        }
    }
}
```

**Why this is bad:**
- Dioxus cannot efficiently track dependencies within match
- Causes unnecessary re-renders of entire component
- Diffing becomes less efficient
- Memory usage increases

### DO: Use if/else for simple conditions

```rust
// ✅ GOOD - If/else for simple conditions
rsx! {
    div {
        if is_loading() {
            div { "Loading..." }
        } else if let Some(data) = data() {
            div { "{data}" }
        } else if let Some(err) = error() {
            div { "Error: {err}" }
        }
    }
}
```

### DO: Extract match to a function

```rust
// ✅ GOOD - Extract match to separate function
fn render_state(state: State) -> Element {
    match state {
        State::Loading => rsx! { div { "Loading..." } },
        State::Loaded { data } => rsx! { div { "{data}" } },
        State::Error { msg } => rsx! { div { "Error: {msg}" } },
    }
}

#[component]
fn MyComponent() -> Element {
    let state = use_signal(|| State::Loading);
    
    rsx! {
        div {
            {render_state(state())}
        }
    }
}
```

### DO: Use computed components

```rust
// ✅ GOOD - Use memoized rendering
fn render_data(data: String) -> Element {
    rsx! { div { "{data}" } }
}

#[component]
fn MyComponent() -> Element {
    let data = use_signal(|| String::new);
    let is_loading = use_signal(|| false);
    
    rsx! {
        if *is_loading.read() {
            div { "Loading..." }
        } else {
            {render_data(data())}
        }
    }
}
```

### Performance Comparison

```rust
// ❌ BAD: 5ms render time, allocates 3 nodes on every change
rsx! {
    div {
        match status() {
            Status::Active => active_component(),
            Status::Inactive => inactive_component(),
            Status::Pending => pending_component(),
            Status::Complete => complete_component(),
            Status::Error => error_component(),
        }
    }
}

// ✅ GOOD: 0.5ms render time, only allocates needed node
fn render_status(status: Status) -> Element {
    match status {
        Status::Active => rsx! { ActiveComponent {} },
        Status::Inactive => rsx! { InactiveComponent {} },
        Status::Pending => rsx! { PendingComponent {} },
        Status::Complete => rsx! { CompleteComponent {} },
        Status::Error => rsx! { ErrorComponent {} },
    }
}

rsx! {
    div {
        {render_status(status())}
    }
}
```

### DON'T: Use long RSX blocks

```rust
// ❌ BAD - Long RSX block (50+ lines, hard to read, test, maintain)
#[component]
fn UserProfile() -> Element {
    let user = use_signal(|| None::<User>);
    
    rsx! {
        div { class: "profile-container",
            div { class: "header",
                h1 { class: "title", "User Profile" }
                button { class: "btn-edit", "Edit" }
            }
            div { class: "content",
                if let Some(u) = user() {
                    div { class: "avatar-section",
                        img { class: "avatar", src: u.avatar_url }
                        div { class: "user-info",
                            h2 { "{u.name}" }
                            p { "{u.email}" }
                            div { class: "stats",
                                div { "Posts: {u.posts}" }
                                div { "Followers: {u.followers}" }
                                div { "Following: {u.following}" }
                            }
                        }
                    }
                    div { class: "bio-section",
                        h3 { "Bio" }
                        p { "{u.bio}" }
                    }
                    div { class: "contact-section",
                        h3 { "Contact" }
                        div { "Phone: {u.phone}" }
                        div { "Address: {u.address}" }
                    }
                    div { class: "settings-section",
                        h3 { "Settings" }
                        label { "Email Notifications" }
                        input { r#type: "checkbox" }
                        label { "Public Profile" }
                        input { r#type: "checkbox" }
                    }
                } else {
                    div { class: "loading", "Loading..." }
                }
            }
            div { class: "footer",
                button { class: "btn-save", "Save" }
                button { class: "btn-cancel", "Cancel" }
            }
        }
    }
}
```

**Why this is bad:**
- Hard to read and understand at a glance
- Difficult to test individual sections
- Changes risk breaking unrelated parts
- Poor reusability - can't reuse sections in other components
- Hard to maintain as component grows
- Slower rendering due to large diff scope

### DO: Split into multiple component functions

```rust
// ✅ GOOD - Split into small, focused components
#[component]
fn UserProfile() -> Element {
    let user = use_signal(|| None::<User>);
    
    rsx! {
        ProfileContainer {
            header: rsx! { ProfileHeader {} },
            content: if let Some(u) = user() {
                rsx! { ProfileContent { user: u } }
            } else {
                rsx! { LoadingState {} }
            },
            footer: rsx! { ProfileFooter {} },
        }
    }
}

#[component]
fn ProfileContainer(header: Element, content: Element, footer: Element) -> Element {
    rsx! {
        div { class: "profile-container",
            {header}
            div { class: "content", {content} }
            {footer}
        }
    }
}

#[component]
fn ProfileHeader() -> Element {
    rsx! {
        div { class: "header",
            h1 { class: "title", "User Profile" }
            button { class: "btn-edit", "Edit" }
        }
    }
}

#[component]
fn ProfileContent(user: User) -> Element {
    rsx! {
        ProfileAvatar { user: user.clone() }
        ProfileBio { bio: user.bio }
        ProfileContact { phone: user.phone, address: user.address }
        ProfileSettings {}
    }
}

#[component]
fn ProfileAvatar(user: User) -> Element {
    rsx! {
        div { class: "avatar-section",
            img { class: "avatar", src: user.avatar_url }
            UserInfo { user }
        }
    }
}

#[component]
fn UserInfo(user: User) -> Element {
    rsx! {
        div { class: "user-info",
            h2 { "{user.name}" }
            p { "{user.email}" }
            UserStats { posts: user.posts, followers: user.followers, following: user.following }
        }
    }
}

#[component]
fn UserStats(posts: i32, followers: i32, following: i32) -> Element {
    rsx! {
        div { class: "stats",
            div { "Posts: {posts}" }
            div { "Followers: {followers}" }
            div { "Following: {following}" }
        }
    }
}

#[component]
fn ProfileBio(bio: String) -> Element {
    rsx! {
        div { class: "bio-section",
            h3 { "Bio" }
            p { "{bio}" }
        }
    }
}

#[component]
fn ProfileContact(phone: String, address: String) -> Element {
    rsx! {
        div { class: "contact-section",
            h3 { "Contact" }
            div { "Phone: {phone}" }
            div { "Address: {address}" }
        }
    }
}

#[component]
fn ProfileSettings() -> Element {
    rsx! {
        div { class: "settings-section",
            h3 { "Settings" }
            label { "Email Notifications" }
            input { r#type: "checkbox" }
            label { "Public Profile" }
            input { r#type: "checkbox" }
        }
    }
}

#[component]
fn ProfileFooter() -> Element {
    rsx! {
        div { class: "footer",
            button { class: "btn-save", "Save" }
            button { class: "btn-cancel", "Cancel" }
        }
    }
}
```

**Benefits:**
- Each component has single responsibility
- Easy to test individual components
- High reusability across the application
- Better performance - smaller diff scopes
- Easier to maintain and modify
- Clear separation of concerns

### Guidelines for Splitting RSX

1. **Keep RSX under 30 lines** - Extract if longer
2. **One logical unit per component** - Header, footer, content, etc.
3. **Reusable sections** - Extract anything used multiple times
4. **Complex logic** - Move to separate component with clear props
5. **Deeply nested structures** - Flatten by extracting sub-components
6. **Mixed concerns** - Split layout, data display, and interactions

### When to Extract Components

| RSX Length | Action | Example |
|------------|--------|---------|
| < 10 lines | Keep inline | Simple button, label |
| 10-30 lines | Consider extraction | Complex form field |
| > 30 lines | Extract | Lists, cards, sections |
| Repeated logic | Extract | Navigation, modals |

### RSX Splitting Strategy

```rust
// Strategy 1: By UI Section
rsx! {
    Header {}
    Sidebar {}
    MainContent {}
    Footer {}
}

// Strategy 2: By Functionality
rsx! {
    UserList {}
    UserSearch {}
    UserFilters {}
}

// Strategy 3: By Data Type
rsx! {
    for item in items() {
        ItemType1 { item: item }
    }
    for item in items() {
        ItemType2 { item: item }
    }
}

// Strategy 4: By Complexity
rsx! {
    SimpleItem {}
    ComplexItem { data: complex_data }
}
```

### Rules of Thumb

1. **Never use match in rsx!{}** - Always extract to a function
2. **Use if/else for 2-3 conditions** - More readable in RSX
3. **Extract complex rendering** - Keep RSX clean and declarative
4. **Use match in functions** - Perfectly fine outside RSX
5. **Keep RSX under 30 lines** - Split into smaller components
6. **One responsibility per component** - Focus on single UI purpose
7. **Extract reusable sections** - Promote code reuse
8. **Leverage use_memo** - Cache expensive computations before rendering

## Components

### Basic Component

```rust
#[component]
fn Button() -> Element {
    rsx! {
        button { "Click me" }
    }
}
```

### Component with Props

```rust
#[component]
fn Button(text: String, on_click: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            onclick: on_click,
            "{text}"
        }
    }
}
```

### Props Rules

- Props must be **owned values** (use `String`, `Vec<T>` instead of `&str`, `&[T]`)
- Props must implement `PartialEq` and `Clone`
- Component re-renders when props change

### Advanced Props

```rust
#[component]
fn Card(
    #[props(default)] title: Option<String>,
    #[props(into)] class: String,
    children: Element,
) -> Element {
    rsx! {
        div { class, "{title.unwrap_or_default()}", {children} }
    }
}
```

### Props Derive Macro

When extracting component properties into a struct, use `#[derive(Props)]`:

```rust
#[derive(PartialEq, Clone, Props)]
pub struct CardProps {
    pub title: String,
    pub content: String,
    #[props(default)]
    pub footer: Option<String>,
    #[props(into)]
    pub class: String,
    pub children: Element,
}

#[component]
fn Card(props: CardProps) -> Element {
    rsx! {
        div { class: props.class,
            h3 { "{props.title}" }
            p { "{props.content}" }
            if let Some(footer) = props.footer {
                div { "{footer}" }
            }
            {props.children}
        }
    }
}
```

### Props Attributes

```rust
#[derive(Props)]
pub struct MyProps {
    // Optional field with default None
    #[props(default)]
    pub optional_field: Option<String>,
    
    // Field with custom default value
    #[props(default = "default_value")]
    pub with_default: String,
    
    // Convert type using Into trait
    #[props(into)]
    pub flexible_type: String,
    
    // Make optional field required
    #[props(!optional)]
    pub required_optional: Option<String>,
    
    // Children elements
    pub children: Element,
    
    // Extend with global attributes
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}
```

### Props Builder

```rust
// Use builder pattern outside RSX
let props = CardProps::builder()
    .title("Hello".to_string())
    .content("World".to_string())
    .class("card".into())
    .children(rsx! { "Content here" })
    .build();

rsx! { Card { ..props } }
```

### Rules for Props

1. **Must derive `PartialEq`** - Used for re-render optimization
2. **Must derive `Clone`** - Props are cloned on each render
3. **Must derive `Props`** - Provides builder pattern
4. **Use owned types** - `String`, `Vec<T>`, not `&str`, `&[T]`
5. **Use `ReadSignal<T>`** - For reactive props

## State Management

### use_signal

```rust
#[component]
fn Counter() -> Element {
    let mut count = use_signal(|| 0);
    
    rsx! {
        h1 { "Count: {count}" }
        button {
            onclick: move |_| *count.write() += 1,
            "Increment"
        }
        button {
            onclick: move |_| count.with_mut(|c| *c += 1),
            "Increment with with_mut"
        }
    }
}
```

### use_memo

```rust
#[component]
fn Counter() -> Element {
    let mut count = use_signal(|| 0);
    let doubled = use_memo(move || count() * 2);  // Recalculates when count changes
    
    rsx! {
        h1 { "Count: {count}" }
        h2 { "Doubled: {doubled}" }
    }
}
```

### Signal Reading

```rust
// Clone value
let value = my_signal();

// Get reference
let value_ref = my_signal.read();

// Mutate
*my_signal.write() = new_value;
my_signal.with_mut(|v| *v += 1);
```

## Component Lifecycle

### Initializing State with use_hook

```rust
fn UseHook() -> Element {
    // The closure passed to use_hook will be called once when component is first rendered
    // On re-renders, the value created on first run will be re-used
    let random_number = use_hook(|| {
        let new_random_number = random_number();

        log!("{new_random_number}");

        new_random_number
    });

    rsx! {
        div { "Random {random_number}" }
    }
}
```

### Rerendering

Components re-render when tracked values change:

```rust
fn Rerenders() -> Element {
    let mut count = use_signal(|| 0);

    log!("Rerendering parent component with {}", *count.peek());

    rsx! {
        button { onclick: move |_| count += 1, "Increment" }
        // Component will rerender when count changes since we read count here
        Count { current_count: count() }
    }
}

// Child component will rerender when count prop changes
#[component]
fn Count(current_count: i32) -> Element {
    log!("Rerendering child component with {current_count}");

    rsx! {
        div { "The count is {current_count}" }
    }
}
```

### Don't mutate state in component body

Avoid changing state in the body of a component. This can cause infinite loops:

```rust
// ❌ BAD - Don't mutate state in component body
fn Bad() -> Element {
    let mut count = use_signal(|| 0);

    // This causes infinite loop!
    count += 1;

    rsx! { "{count}" }
}

// ✅ GOOD - Use use_memo, use_resource, or effects instead
fn Good() -> Element {
    let count = use_memo(|| {
        // Computed value without infinite loop
        0
    });

    rsx! { "{count}" }
}
```

### Using Effects

Run code after component is rendered:

```rust
fn Effect() -> Element {
    // Effects run after component is rendered
    // Use them to read or modify the rendered component
    use_effect(|| {
        log!("Effect ran");
        document::eval(&format!(
            "document.getElementById('effect-output').innerText = 'Effect ran'"
        ));
    });

    rsx! {
        div { id: "effect-output", "This will be changed by effect" }
    }
}
```

### Cleaning Up Components with Drop

Before a component is dropped, it will drop all of its hooks:

```rust
fn TogglesChild() -> Element {
    let mut show = use_signal(|| true);

    rsx! {
        button { onclick: move |_| show.toggle(), "Toggle" }
        if show() {
            Child {}
        }
    }
}

fn Child() -> Element {
    // Clean up any resources when component is dropped
    dioxus::core::use_drop(|| {
        log!("Child dropped");
    });

    rsx! {
        div { "Child" }
    }
}
```

### Lifecycle Hooks Summary

| Hook | When it Runs | Use Case |
|------|-------------|----------|
| `use_hook` | Once on first render | Initialize expensive resources, cache values |
| `use_effect` | After each render | Read/modify DOM, sync state with external systems |
| `use_drop` | When component is unmounted | Clean up resources, event listeners, timers |

## Context API

### Providing Context

```rust
#[component]
fn App() -> Element {
    let mut theme = use_signal(|| "light".to_string());
    use_context_provider(|| theme);
    
    rsx! { Child {} }
}
```

### Consuming Context

```rust
#[component]
fn Child() -> Element {
    let theme = use_context::<Signal<String>>();
    
    match theme {
        Some(theme) => rsx! { div { "Theme: {theme}" } },
        None => rsx! { div { "No theme context" } },
    }
}
```

### Custom Context Hook

```rust
pub fn use_auth_state() -> Signal<AuthState> {
    use_context::<Signal<AuthState>>()
        .expect("AuthState context not found")
}

#[component]
fn Profile() -> Element {
    let auth = use_auth_state();
    rsx! { /* ... */ }
}
```

## Async Patterns

### Future Basics

Futures represent values that may not yet be ready. Key characteristics:

```rust
// Futures are lazy - they don't run until awaited or spawned
let future = async {
    println!("This won't run until awaited or spawned");
};

// Run by awaiting in another future
spawn(async {
    future.await;
});

// Or spawn directly
spawn(future);
```

**Important Concepts:**
- Futures are lazy - don't execute until `.await` or `spawn()`
- Futures are concurrent but not parallel - run on main thread
- Futures pause at `.await` points - don't hold locks across awaits
- Futures can be cancelled - must be cancel-safe
- Don't block the UI thread - use separate threads for expensive tasks

### Running Futures with spawn

Use `spawn()` to run futures in background:

```rust
let mut response = use_signal(|| "Click to start".to_string());

rsx! {
    button {
        onclick: move |_| {
            response.set("...".into());
            spawn(async move {
                let resp = reqwest::Client::new()
                    .get("https://dioxuslabs.com")
                    .send()
                    .await;

                if resp.is_ok() {
                    response.set("Success!".into());
                } else {
                    response.set("Failed!".into());
                }
            });
        },
        "{response}"
    }
}
```

**Auto-spawn in event handlers:**
```rust
button {
    // Async closures are automatically spawned
    onclick: move |_| async move {
        let resp = reqwest::get("https://dioxuslabs.com").await;
        response.set("Done!".into());
    },
    "Fetch"
}
```

### use_action Hook

For user-triggered actions with automatic cancellation:

```rust
let mut breed = use_action(move |breed: String| async move {
    reqwest::get(format!("https://dog.ceo/api/breed/{breed}/images/random"))
        .await
        .unwrap()
        .json::<DogApi>()
        .await
});

rsx! {
    button {
        onclick: move |_| breed.call("hound".to_string()),
        "Fetch Dog"
    }
    
    match breed.value() {
        Some(Ok(res)) => rsx! { img { src: "{res.message}" } },
        Some(Err(_)) => rsx! { div { "Failed" } },
        None => rsx! { div { "Click to fetch" } },
    }
}
```

Calling `.call()` cancels previous pending action.

### Cancellation

**Automatic cancellation:** Futures are cancelled when component unmounts.

**Manual cancellation:**
```rust
let mut task = use_signal(|| None);

button {
    onclick: move |_| {
        let new_task = spawn(async move {
            // Long running task
        });
        task.set(Some(new_task));
    },
    "Start"
}

button {
    onclick: move |_| {
        if let Some(t) = task.take() {
            t.cancel();
        }
    },
    "Cancel"
}
```

**spawn_forever:** Runs task that won't be cancelled on unmount:
```rust
spawn_forever(async move {
    // Runs until app closes
});
```

### Cancel Safety

Futures can be cancelled at any `.await` point. Clean up resources:

```rust
// ❌ BAD - Global state not restored on cancellation
let dogs = use_resource(move || async move {
    GLOBAL_STATE.write().insert(breed());
    let response = fetch_data().await;
    // If cancelled here, breed never removed!
    response
});

// ✅ GOOD - Use Drop guard for cleanup
let dogs = use_resource(move || async move {
    GLOBAL_STATE.write().insert(breed());
    
    struct DropGuard(String);
    impl Drop for DropGuard {
        fn drop(&mut self) {
            GLOBAL_STATE.write().remove(&self.0);
        }
    }
    let _guard = DropGuard(breed());
    
    fetch_data().await
});
```

### Concurrency vs Parallelism

```rust
// Concurrent - yields at await points, runs on single thread
spawn(async {
    let a = fetch_a().await;
    let b = fetch_b().await;
});

// Parallel - runs on separate threads
std::thread::spawn(|| {
    heavy_computation();
});
```

Dioxus runtime is single-threaded. Don't block with expensive operations:
```rust
// ❌ BAD - Blocks UI thread
spawn(async {
    solve_for_the_answer_to_life_and_everything();
});

// ✅ GOOD - Run on separate thread
std::thread::spawn(|| {
    solve_for_the_answer_to_life_and_everything();
});
```

### Handling Locks

Never hold locks across `.await` points:
```rust
// ❌ BAD - Lock held across await
let data = DATA.read();
let result = fetch_data(data.id).await;

// ✅ GOOD - Release lock before await
let id = DATA.read().id;
drop(data); // Release lock
let result = fetch_data(id).await;
```

### Data Fetching

Dioxus does not provide any built-in utilities for fetching data. Crates like dioxus-query exist, but for most use cases we'll implement data-fetching from scratch.

#### Adding Dependencies

First, we need to add two new dependencies to our app: serde and reqwest.

- **Reqwest** provides an HTTP client for fetching
- **Serde** will let us derive a JSON Deserializer to decode the response

Add these crates to your app:

```bash
cargo add reqwest --features json
cargo add serde --features derive
```

#### Defining a Response Type

We'll be using an API and need to create a Rust struct that matches the format of the API response and derive `Deserialize` for it.

For example, the Dog API provides a simple response:

```json
{
    "message": "https://images.dog.ceo/breeds/leonberg/n02111129_974.jpg",
    "status": "success"
}
```

Our Rust struct needs to match that format:

```rust
#[derive(serde::Deserialize)]
struct DogApi {
    message: String,
}
```

#### Using reqwest and async

Dioxus has great support for asynchronous Rust. We can simply convert our onclick handler to be async and then set the signal after the future has resolved.

```rust
#[component]
fn DogView() -> Element {
    let mut img_src = use_signal(|| "".to_string());

    let save = move |_| async move {
        let response = reqwest::get("https://dog.ceo/api/breeds/image/random")
            .await
            .unwrap()
            .json::<DogApi>()
            .await
            .unwrap();

        img_src.set(response.message);
    };

    rsx! {
        div { id: "dogview",
            img { src: "{img_src}" }
        }
        div { id: "buttons",
            button { onclick: save, id: "save", "Fetch!" }
        }
    }
}
```

Dioxus automatically calls `dioxus::spawn` on asynchronous closures. You can also use `dioxus::spawn` to perform async work without async closures:

```rust
rsx! {
    button {
        onclick: move |_| {
            spawn(async move {
                // do some async work...
            });
        }
    }
}
```

**Important:** The futures passed to `dioxus::spawn` cannot borrow data from outside the async block. Data that is `Copy` can be captured by async blocks, but all other data must be moved, usually by calling `.clone()`.

#### Data Fetching with use_resource

Using bare async calls might lead to race conditions and weird state bugs. For example, if the user clicks the fetch button too quickly, then two requests will be made in parallel. If the request is updating data somewhere else, the wrong request might finish first and cause a race condition.

In Dioxus, Resources are pieces of state whose value is dependent on the completion of some asynchronous work. The `use_resource` hook provides a Resource object with helpful methods to start, stop, pause, and modify the asynchronous state.

Let's change our component to use a resource instead:

```rust
#[component]
fn DogView() -> Element {
    let mut img_src = use_resource(|| async move {
        reqwest::get("https://dog.ceo/api/breeds/image/random")
            .await
            .unwrap()
            .json::<DogApi>()
            .await
            .unwrap()
            .message
    });

    rsx! {
        div { id: "dogview",
            img { src: img_src.cloned().unwrap_or_default() }
        }
        div { id: "buttons",
            button { onclick: move |_| img_src.restart(), id: "skip", "Refresh" }
            button { onclick: move |_| img_src.restart(), id: "save", "Fetch!" }
        }
    }
}
```

Resources are very powerful: they integrate with Suspense, Streaming HTML, reactivity, and more.

### use_resource

For data that depends on async operations:

```rust
#[component]
fn UserProfile() -> Element {
    let user_id = use_signal(|| 1);
    
    let user = use_resource(move || async move {
        fetch_user(user_id()).await
    });
    
    match user() {
        Some(Ok(user)) => rsx! { div { "User: {user.name}" } },
        Some(Err(e)) => rsx! { div { "Error: {e}" } },
        None => rsx! { div { "Loading..." } },
    }
}
```

**Resource Behavior:**
- Closure runs whenever signals it reads are updated
- Returns `Option<Result<T, E>>`:
  - `None` = loading
  - `Some(Ok(value))` = success
  - `Some(Err(e))` = error

## Routing

### Define Routes

```toml
# Cargo.toml
dioxus = { version = "0.7.1", features = ["router"] }
```

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(NavBar)]
    #[route("/")]
    Home {},
    
    #[route("/blog/:id")]
    BlogPost { id: i32 },
    
    #[route("/user/:name")]
    UserProfile { name: String },
}
```

### Router Component

```rust
#[component]
fn App() -> Element {
    rsx! { Router::<Route> {} }
}
```

### Layout with Outlet

```rust
#[component]
fn NavBar() -> Element {
    rsx! {
        nav {
            Link { to: Route::Home {}, "Home" }
            Outlet::<Route> {}  // Renders child routes
        }
    }
}
```

### Navigation

```rust
// Link component
Link { to: Route::BlogPost { id: 123 }, "Read Post" }

// Programmatic navigation
let nav = navigator();
nav.push(Route::Home {});      // Add to history
nav.replace(Route::Home {});   // Replace current
nav.go_back();
nav.go_forward();
```

## Fullstack

### Setup

```toml
dioxus = { version = "0.7.1", features = ["fullstack"] }
```

### Server Functions

```rust
#[post("/api/double/:number")]
async fn double(number: i32) -> Result<i32, ServerFnError> {
    Ok(number * 2)
}

#[get("/api/user/:id")]
async fn get_user(id: String) -> Result<User, ServerFnError> {
    // Fetch from database
    Ok(user)
}
```

### Calling Server Functions

```rust
// Automatically makes HTTP request from client
let result = double(42).await?;
```

### Hydration

```rust
#[component]
fn App() -> Element {
    // use_server_future for server-rendered data
    let user = use_server_future(move || async move {
        fetch_user().await
    }).suspend();
    
    match &*user.peek() {
        Some(Ok(user)) => rsx! { "Hello {user.name}" },
        _ => rsx! { "Loading..." },
    }
}

// Browser-specific APIs must run after hydration
use_effect(|| {
    // Access localStorage, window, etc.
});
```

## Assets

### Using Assets

```rust
rsx! {
    img {
        src: asset!("/assets/image.png"),
        alt: "Image",
    }
}
```

### Stylesheets

```rust
rsx! {
    document::Stylesheet {
        href: asset!("/assets/styles.css"),
    }
}
```

## Event Handling

### Common Events

```rust
rsx! {
    button {
        onclick: move |e| { /* click event */ },
        "Click me"
    }
    
    input {
        oninput: move |e| { /* input changed */ },
        onkeydown: move |e| {
            if e.key() == Key::Enter {
                // Handle Enter key
            }
        },
    }
    
    form {
        onsubmit: move |e| {
            e.prevent_default();
            // Handle form submit
        },
    }
}
```

### Event Data

```rust
button {
    onclick: move |event: MouseEvent| {
        println!("Click at: {}, {}", event.client_x(), event.client_y());
    },
    "Click"
}
```

## Best Practices

### State Management

- Use `use_signal` for local component state
- Use `use_memo` for expensive derived computations
- Use `use_context_provider`/`use_context` for global state

### Component Design
+- Keep components small and focused
+- **Never use match expressions in rsx!{}** - Extract to separate functions
+- Extract complex props into structs with `#[derive(PartialEq, Clone, Props)]`
+- Use children prop for flexible composition
+- Prefer `ReadSignal<T>` for reactive props
- Use children prop for flexible composition
- Prefer `ReadSignal<T>` for reactive props

### Performance

- Use loops directly in RSX instead of iterators
- Wrap expensive operations in `use_memo`
- Minimize unnecessary re-renders by proper use of `PartialEq`

### Code Style

- Solid, lean code
- No hardcoding - use constants
- Zero-copy patterns preferred
- Small, focused functions
- Use `RwLock` instead of `Mutex`
- Comments only for complex logic

## Common Patterns

### Protected Route

```rust
#[component]
fn ProtectedRoute(children: Element) -> Element {
    let auth = use_context::<Signal<AuthState>>();
    
    match auth() {
        Some(AuthState::Authenticated { .. }) => rsx! { {children} },
        _ => rsx! { Redirect { to: Route::Login {} } },
    }
}
```

### Form Handling

```rust
#[component]
fn Form() -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    
    rsx! {
        form {
            onsubmit: move |e| {
                e.prevent_default();
                // Handle submit
            },
            input {
                r#type: "email",
                value: "{email}",
                oninput: move |e| *email.write() = e.value(),
            }
            input {
                r#type: "password",
                value: "{password}",
                oninput: move |e| *password.write() = e.value(),
            }
            if let Some(err) = error() {
                div { class: "error", "{err}" }
            }
            button { "Submit" }
        }
    }
}
```

### Data Fetching

```rust
#[component]
fn DataList() -> Element {
    let items = use_resource(move || async move {
        fetch_items().await
    });
    
    rsx! {
        match items() {
            Some(Ok(items)) => rsx! {
                ul {
                    for item in items {
                        li { "{item.name}" }
                    }
                }
            },
            Some(Err(e)) => rsx! { div { "Error: {e}" } },
            None => rsx! { div { "Loading..." } },
        }
    }
}
```

## Debugging

### Console Logging

```rust
use dioxus_logger::tracing::{info, error, warn};

#[component]
fn App() -> Element {
    info!("App rendered");
    
    rsx! { "Hello" }
}
```

### Error Handling

```rust
let result = use_resource(move || async move {
    match fetch_data().await {
        Ok(data) => Ok(data),
        Err(e) => {
            error!("Fetch failed: {}", e);
            Err(e)
        }
    }
});
```

## Migration from Dioxus 0.6

### Breaking Changes

- `cx` parameter removed from components
- `Scope` type removed
- `use_state` → `use_signal`
- `use_context` → `use_context::<T>()`
- Event handlers simplified

### Quick Reference

| Dioxus 0.6 | Dioxus 0.7 |
|------------|------------|
| `cx: Scope` | Removed |
| `use_state(\| \|\| 0)` | `use_signal(\| \|\| 0)` |
| `use_context::<Type>(cx)` | `use_context::<Type>()` |
| `onclick: |_| {}` | `onclick: move |_| {}` |

## CLI Commands

```bash
# Check compilation
dx check

# Serve with specific renderer
dx serve --web
dx serve --desktop
dx serve --mobile

# Build
dx build --web
dx build --desktop
```

## Resources

- Official Documentation: https://dioxuslabs.com/learn/0.7
- GitHub: https://github.com/DioxusLabs/dioxus
- Discord: https://discord.gg/XgGx9kv8Yt