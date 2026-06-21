FROM node:20-slim

WORKDIR /app

ENV CI=1 \
    PLAYWRIGHT_BROWSERS_PATH=/ms-playwright

COPY package.json yarn.lock* ./
RUN yarn install --frozen-lockfile --silent
RUN yarn playwright install --with-deps chromium

COPY playwright.config.ts ./
COPY tests ./tests

CMD ["yarn", "playwright", "test"]
