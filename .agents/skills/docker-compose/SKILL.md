---
name: docker-compose
description: Manage local Docker Compose workflows for this project — building images, running services, debugging container exits, and rebuilding after code changes.
---

# Docker Compose Local Development

This project has two Docker services defined in `docker-compose.yml`:

| Service | Dockerfile | Role | Port |
|---------|-----------|------|------|
| `content_proxy` | `Dockerfile.proxy` | Pingora reverse proxy — routes `/rest/` and `/auth/` to Supabase, everything else to `content_ui` | `6190` |
| `content_ui` | `Dockerfile.ui` | Dioxus web app served by nginx | `80` (internal only) |

Both run on the `content_net` bridge network.

## Required Env Vars

The `.env` file at project root must provide:

- `SUPABASE_URL` — e.g. `https://xxx.supabase.co`
- `SUPABASE_ANON_KEY` — public anon key

Never read or expose `.env` contents directly.

## Commands

### Full rebuild (after code changes)

```sh
docker compose build --no-cache
docker compose up -d
```

### Rebuild a single service

```sh
docker compose build --no-cache content_proxy
docker compose up -d content_proxy
```

### Quick rebuild (may use cache)

```sh
docker compose build content_proxy
docker compose up -d content_proxy
```

### Check status

```sh
docker compose ps -a
```

### View logs

```sh
docker compose logs -f content_proxy
docker compose logs -f content_ui
```

### Stop everything

```sh
docker compose down
```

## Verifying Services Are Running

After `docker compose up -d`, always verify:

1. `docker compose ps -a` — both services should show `Up`
2. `curl -s -o /dev/null -w "%{http_code}" http://localhost:6190/` — should return `200`

## Debugging Container Exit (Code 0)

If `content_proxy-1` exits with code 0 immediately:

1. Check logs: `docker compose logs content_proxy`
2. If no output at all, the binary is likely the **dummy stub** from Dockerfile cache
3. The Dockerfile uses a two-stage build: first a dummy `fn main() {}` to cache dependencies, then copies real source and rebuilds
4. Docker layer caching can cause the second `cargo build` to reuse the dummy binary
5. Fix: `docker compose build --no-cache content_proxy` to force full rebuild

The `Dockerfile.proxy` includes `touch content_proxy/src/main.rs` before the final build to prevent this cache issue.

## Debugging Container Exit (Non-Zero)

- Exit code **101**: Rust panic. Check `docker compose logs` for the panic message.
- Exit code **137**: OOM killed. Check `docker stats`.
- Missing `.env` vars will cause `content_proxy` to panic at startup on `env::var("SUPABASE_URL").expect(...)`.

## content_ui Build Notes

`Dockerfile.ui` expects the Dioxus build output at `target/dx/content_ui/release/web/public/`. Build the UI first:

```sh
dx bundle --release
```

Then build the Docker image. The `.dockerignore` allows `target/dx/` through but blocks the rest of `target/`.

## Network Architecture

```
Browser → :6190 (content_proxy)
              ├─ /rest/*, /auth/* → Supabase Cloud (HTTPS)
              └─ /*              → content_ui:80 (nginx)
```
