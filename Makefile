# Content CMS — build & run tasks.
# All Docker assets live in docker/.
#
# USAGE
# -----
#   Daily flow (from the repo root, needs .env with SUPABASE_URL etc.):
#     make rebuild    # THE command after any UI/code change:
#                     #   tailwind CSS -> dx build (WASM) -> docker images -> up
#     make up         # start nginx + content_ui  (http://localhost:6190)
#     make up-build   # one-step: build images + start (up --build)
#     make test       # run Playwright smoke tests against the stack
#     make logs       # watch logs            make ps: container status
#     make down       # stop the stack        make restart: down + up
#
#   NOTE: --env-file .env is required because the compose file lives in
#   docker/ — without it compose would look for docker/.env and warn that
#   SUPABASE_URL is not set.
#
#   Rust development:
#     make check      # cargo check the workspace
#     make clippy     # cargo clippy the workspace
#     make test-rust  # cargo unit tests (content_ui)
#
#   Other:
#     make clean      # stop stack and remove local images/volumes
#     make help       # list all targets

COMPOSE := docker compose --env-file .env -f docker/docker-compose.yml
COMPOSE_TEST := docker compose -f playwright_cli/docker-compose.test.yml

.PHONY: help tailwind build up up-build down restart logs ps test check clippy test-rust clean

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

tailwind: ## Rebuild Tailwind CSS (run after any class change)
	cd content_ui && npx @tailwindcss/cli -i ./assets/input.css -o ./assets/tailwind.css --minify

web: tailwind ## Compile the WASM app into target/dx/.../public (needed by ui.Dockerfile)
	dx build --release --platform web --package content_ui

build: web ## Rebuild the Docker images (always bundles fresh source)
	$(COMPOSE) build

up: ## Start nginx + content_ui in the background
	$(COMPOSE) up -d

up-build: ## Build images and start in one step (docker compose up --build)
	$(COMPOSE) up -d --build

down: ## Stop the stack
	$(COMPOSE) down

restart: down up ## Restart the stack

logs: ## Follow logs from the running stack
	$(COMPOSE) logs -f

ps: ## Show container status
	$(COMPOSE) ps -a

test: ## Run Playwright smoke tests against the running stack
	$(COMPOSE_TEST) run --rm playwright

check: ## cargo check the workspace
	cargo check

clippy: ## cargo clippy the workspace
	cargo clippy

test-rust: ## Run Rust unit tests
	cargo test -p content_ui

rebuild: web build up ## Tailwind + WASM + images + restart — run after any UI/code change

clean: ## Stop the stack and remove build artifacts
	$(COMPOSE) down --rmi local --volumes
