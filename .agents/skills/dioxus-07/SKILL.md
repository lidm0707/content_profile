---
name: dioxus-07
description: Helps with Dioxus 0.7 development — RSX, signals, components, routing, fullstack server functions, and hydration. Use this for any Dioxus UI, state management, or fullstack work in this project.
---

# Dioxus 0.7 Assistant

You are an expert Dioxus 0.7 assistant. Dioxus 0.7 changed nearly every API from earlier versions. Never use pre-0.7 patterns.

## Critical Breaking Changes from Pre-0.7

These APIs are **gone**. Never generate code using them:
- `cx` — removed. Components no longer take a scope parameter.
- `Scope` — removed.
- `use_state` — removed. Use `use_signal` instead.
- `cx.spawn()` — removed. Use `spawn` (free function) or `use_resource`.
- `to_owned!` — removed. Rust 2021 closures capture by move natively.
- `gather_async` — removed.
- `suspend` — removed (0.6 concept).

## Component Signature

```rust
#[component]
fn MyComponent() -> Element {
    rsx! { "Hello" }
}
```

No `cx`, no `Scope`, no function parameters unless you define props.

## State: use_signal, use_memo, use_resource

```rust
let mut count = use_signal(|| 0);
let doubled = use_memo(move || count() * 2);
```

Read a signal by calling it: `count()`. Write: `count.set(5)` or `*count.write() += 1`.

## Full Reference

For the complete Dioxus 0.7 API reference including RSX patterns, routing, fullstack server functions, hydration, event handling, anti-patterns, and migration tips, read:

```
.skill/dioxus_skill.md
```

This file contains ~1500 lines of detailed examples and patterns. Consult it whenever you need API details, component patterns, or async/state guidance.

## Quick Checks Before Generating Dioxus Code

1. No `cx`, `Scope`, or `use_state` anywhere.
2. Components use `#[component]` macro and return `Element`.
3. Props are owned values (`String`, `Vec<T>`), never references.
4. Signals are read by calling them `signal()`, not `.get()`.
5. Use `use_resource` for async data fetching, `use_server_future` for fullstack hydration.
6. Server functions use `#[post]` / `#[get]` macros, return `Result<T, ServerFnError>`.
7. Prefer `for` loops in RSX over iterators. Wrap iterators in braces `{}` when used.
