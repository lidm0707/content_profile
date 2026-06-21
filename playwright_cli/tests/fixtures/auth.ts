import type { Page } from "@playwright/test";

const SESSION_KEY = "cms_auth_session";
const FAR_FUTURE_EXPIRES_AT = 9999999999;

export interface FakeUserOptions {
  email?: string;
  accessToken?: string;
}

export function buildFakeSessionPayload(opts: FakeUserOptions = {}) {
  const {
    email = "test@example.com",
    accessToken = "fake-test-jwt-not-valid",
  } = opts;

  return {
    session: {
      access_token: accessToken,
      refresh_token: "fake-refresh-token",
      expires_at: FAR_FUTURE_EXPIRES_AT,
      token_type: "bearer",
      user: {
        id: "00000000-0000-0000-0000-000000000000",
        email,
        email_confirmed_at: "2024-01-01T00:00:00Z",
        created_at: "2024-01-01T00:00:00Z",
        updated_at: "2024-01-01T00:00:00Z",
        last_sign_in_at: "2024-01-01T00:00:00Z",
      },
    },
  };
}

export async function seedFakeSession(
  page: Page,
  opts: FakeUserOptions = {},
): Promise<void> {
  const payload = buildFakeSessionPayload(opts);
  await page.addInitScript(({ key, value }) => {
    try {
      window.localStorage.setItem(key, JSON.stringify(value));
    } catch {
      // ignore — page may not have localStorage ready yet
    }
  }, { key: SESSION_KEY, value: payload });
}
