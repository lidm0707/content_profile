---
name: playwright-cli
description: Run and maintain the Playwright smoke tests in playwright_cli/ — build the test image, execute tests against the running app, debug failures, and add new page specs.
---

# Playwright Smoke Tests (`playwright_cli/`)

Dockerised Playwright (Chromium) tests that verify the Dioxus WASM app renders correctly. Tests join the app's Docker network so they can reach `http://content_proxy:6190`.

## Prerequisites

The app must be running before tests can pass:

```sh
# from project root
docker compose up -d
docker compose ps -a   # both content_proxy + content_ui must be Up
```

If `content_proxy` shows `Exited (137)` it was OOM-killed — restart with `docker compose up -d`.

## Directory Layout

| Path | Purpose |
|------|---------|
| `tests/home.spec.ts` | Public page specs (home page hero, CTA, navigation) |
| `tests/dashboard.spec.ts` | Authenticated page specs (uses session seed + Supabase mock) |
| `tests/fixtures/auth.ts` | `seedFakeSession(page)` — injects fake session into `localStorage` before page load |
| `tests/fixtures/supabaseMock.ts` | `mockSupabaseRest(page)` — intercepts `/rest/v1/**` and returns canned JSON |
| `playwright.config.ts` | Playwright config — serial (`workers: 1`), Chromium only, `baseURL` from `APP_URL` env |
| `package.json` / `yarn.lock` | Yarn-managed deps |
| `Dockerfile` | Node 20 + Playwright + Chromium, uses `yarn` |
| `docker-compose.test.yml` | Runs the test container on `content_profile_content_net` (external). Renamed from `docker-compose.yml` to avoid clashing with the project-root compose file when both are detected by `docker compose`. |
| `.dockerignore` | Excludes `node_modules/`, reports, `.git/` |

## Commands

All commands run from `playwright_cli/`.

### Run tests in Docker (recommended)

```sh
docker compose -f docker-compose.test.yml run --rm playwright
```

### Rebuild the image after editing tests/config

```sh
docker compose -f docker-compose.test.yml build
```

### Point at a different URL

```sh
APP_URL=http://localhost:6190 docker compose -f docker-compose.test.yml run --rm playwright
```

Default is `http://content_proxy:6190` (reaches the app over the shared Docker network). `localhost` only works if the Docker daemon can route to the host.

### Run locally (no Docker)

```sh
yarn install
yarn playwright install chromium
APP_URL=http://localhost:6190 yarn test
```

## Configuration Notes

- **Serial execution**: `workers: 1` and `fullyParallel: false`. Pingora doesn't handle concurrent browser connections well — keep tests serial.
- **Retries**: 2 on CI, 0 locally.
- **Artifacts on failure**: screenshots + video written to `test-results/`, HTML report to `playwright-report/`.
- **`navigationTimeout: 15_000`**: Dioxus WASM hydration is slow on first load.

## Writing New Specs

1. Add a `.spec.ts` file under `tests/`.
2. Use `await page.goto('/')` — `baseURL` is already configured.
3. For navigation assertions, remember the app redirects unauthenticated users to `/login`.
4. Prefer `getByRole` / `getByText` over CSS selectors — Dioxus class names can shift.
5. Rebuild the image after adding the file: `docker compose -f docker-compose.test.yml build`.

## Testing Protected Pages (auth-required routes)

Protected routes (`/dashboard`, `/content/edit/:id`, `/tags`, `/tags/edit/:id`) require a session. The client-side check is only `now < session.expires_at` (see `content_ui/src/app.rs`), with no signature validation, so tests can bypass login by seeding localStorage.

### Pattern 1 — seed a fake session (UI smoke tests)

Use `seedFakeSession(page)` from `tests/fixtures/auth.ts`. It calls `page.addInitScript` so the session is in `localStorage` before the app boots on every navigation — no goto-then-reload dance.

```ts
import { test, expect } from "@playwright/test";
import { seedFakeSession } from "./fixtures/auth";

test("dashboard renders when authenticated", async ({ page }) => {
  await seedFakeSession(page, { email: "tester@example.com" });
  await page.goto("/dashboard");
  await expect(page).toHaveURL(/\/dashboard$/);
  await expect(page.getByText("Content Dashboard")).toBeVisible();
});
```

Caveat: the fake `access_token` is rejected by Supabase server-side, so any real API call (fetch content, create content) will 401. Use this for rendering / navigation / form-display checks only.

### Pattern 2 — also mock Supabase REST (full UI flow without backend)

For tests that need the dashboard list to populate or `Create Content` to succeed, add `mockSupabaseRest(page)` from `tests/fixtures/supabaseMock.ts`. It intercepts `**/rest/v1/**` and returns canned rows for `content`, `tags`, `content_tags`.

```ts
import { test, expect } from "@playwright/test";
import { seedFakeSession } from "./fixtures/auth";
import { mockSupabaseRest, DEFAULT_MOCK_CONTENT } from "./fixtures/supabaseMock";

test("dashboard shows mocked content", async ({ page }) => {
  await seedFakeSession(page);
  await mockSupabaseRest(page);
  await page.goto("/dashboard");
  await expect(page.getByText(DEFAULT_MOCK_CONTENT[0].title)).toBeVisible({ timeout: 15_000 });
});
```

To customise responses, pass your own handler:

```ts
await mockSupabaseRest(page, (url, req) => ({
  body: JSON.stringify([{ id: 1, title: "Custom", /* ... */ }]),
  contentType: "application/json",
}));
```

### Pattern 3 — real login via `storageState` (integration tests)

If you need a real Supabase session (e.g. to verify actual writes), use Playwright's [storageState](https://playwright.dev/docs/auth) pattern: log in once in a global setup, save `storageState` to a JSON file, reuse it across tests. Requires real Supabase credentials — not appropriate for CI smoke tests.

## Common Failures

| Symptom | Cause | Fix |
|---------|-------|-----|
| `ERR_CONNECTION_REFUSED` | App not running or proxy crashed | `docker compose up -d` from project root |
| `timeout` on `page.goto` | WASM hydration slow | Already set to 15s; if still failing, app may be broken |
| `toHaveURL` mismatch | Auth redirect to `/login` | Expected — dashboard requires login |
| All tests fail intermittently | `content_proxy` OOM-killed (exit 137) | Restart the app |

## Network Wiring

The test container joins `content_profile_content_net` as an external network. This network is created by the project-root `docker-compose.yml`. If the network name changes, update `playwright_cli/docker-compose.test.yml` → `networks.content_net.name`.
