# Playwright Smoke Tests

Dockerised Playwright tests that verify the content_profile app renders correctly.

## Prerequisites

The app must be running first (the Playwright container joins the app's network):

```bash
# from the project root
docker compose up --build -d
```

The app exposes itself on the `content_profile_content_net` Docker network as `http://content_proxy:6190`.

## Run the tests

```bash
cd playwright_cli
docker compose -f docker-compose.test.yml run --rm playwright
```

## What it checks

- Home page loads with the correct title (`content_profile`)
- Hero heading renders (`Content Management System` + `Powered by Dioxus & Supabase`)
- "Go to Dashboard" CTA is visible
- Clicking the CTA without auth redirects to `/login`

## Reports

After a run, HTML report and failure artefacts are written to:

- `playwright_cli/playwright-report/` — open with `npx playwright show-report`
- `playwright_cli/test-results/` — screenshots / videos on failure

## Point at a different URL

```bash
APP_URL=http://localhost:6190 docker compose -f docker-compose.test.yml run --rm playwright
```

Note: `localhost` only works if the Docker daemon can route to the host. The default (`http://content_proxy:6190`) uses the shared Docker network and is recommended.

## Run locally (without Docker)

```bash
cd playwright_cli
yarn install
yarn playwright install chromium
APP_URL=http://localhost:6190 yarn test
```
