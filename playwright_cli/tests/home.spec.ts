import { test, expect } from "@playwright/test";

const EXPECTED_TITLE = "content_profile";
const HEADING = "Content Management System";
const SUBHEADING = "Powered by Dioxus & Supabase";
const DASHBOARD_CTA = "Go to Dashboard";

test("home page loads and renders hero section", async ({ page }) => {
  await page.goto("/");

  await expect(page).toHaveTitle(EXPECTED_TITLE);
  await expect(page.locator("h1")).toContainText(HEADING);
  await expect(page.locator("h1")).toContainText(SUBHEADING);
});

test("home page shows dashboard CTA", async ({ page }) => {
  await page.goto("/");

  const cta = page.getByRole("link", { name: DASHBOARD_CTA });
  await expect(cta).toBeVisible();
});

test("dashboard CTA redirects to login when unauthenticated", async ({
  page,
}) => {
  await page.goto("/");

  await page.getByRole("link", { name: DASHBOARD_CTA }).click();
  await expect(page).toHaveURL(/\/login$/);
});
