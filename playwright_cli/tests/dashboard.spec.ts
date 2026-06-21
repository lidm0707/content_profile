import { test, expect } from "@playwright/test";
import { seedFakeSession } from "./fixtures/auth";
import {
  mockSupabaseRest,
  DEFAULT_MOCK_CONTENT,
} from "./fixtures/supabaseMock";

test.describe("auth seed via localStorage", () => {
  test("dashboard renders for seeded authenticated user", async ({ page }) => {
    await seedFakeSession(page, { email: "tester@example.com" });
    await mockSupabaseRest(page);

    await page.goto("/dashboard");

    await expect(page).toHaveURL(/\/dashboard$/);
    await expect(page.getByText("Content Dashboard")).toBeVisible({
      timeout: 15_000,
    });
    await expect(
      page
        .getByRole("navigation")
        .getByRole("link", { name: "Create Content" }),
    ).toBeVisible();
    await expect(page.getByText("tester@example.com")).toBeVisible();
  });

  test("dashboard shows mocked content cards", async ({ page }) => {
    await seedFakeSession(page);
    await mockSupabaseRest(page);

    await page.goto("/dashboard");

    await expect(page.getByText(DEFAULT_MOCK_CONTENT[0].title)).toBeVisible({
      timeout: 15_000,
    });
  });
});

test.describe("unauthenticated fallback", () => {
  test("dashboard redirects to /login without seeded session", async ({
    page,
  }) => {
    await page.goto("/dashboard");
    await expect(page).toHaveURL(/\/login$/);
  });
});
