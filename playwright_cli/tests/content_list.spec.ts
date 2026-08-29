import { test, expect } from "@playwright/test";
import { seedFakeSession } from "./fixtures/auth";
import {
  mockSupabaseRest,
  MockContentRow,
  RestHandler,
} from "./fixtures/supabaseMock";

// Two rows whose ids/insertion order disagree with created_at, so the
// client-side sort (newest first) is actually observable.
const TWO_POSTS: MockContentRow[] = [
  {
    id: 1,
    title: "Older Post",
    slug: "older-post",
    body: "---\ntags: []\n---\nOlder",
    status: "published",
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
    synced_at: null,
  },
  {
    id: 2,
    title: "Newer Post",
    slug: "newer-post",
    body: "---\ntags: []\n---\nNewer",
    status: "draft",
    created_at: "2025-06-15T12:00:00Z",
    updated_at: "2025-06-15T12:00:00Z",
    synced_at: null,
  },
];

const twoPostsHandler: RestHandler = (url) => {
    const table = url.pathname.split("/rest/v1/")[1]?.split("?")[0];
    if (table === "content") {
      return {
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(TWO_POSTS),
      };
    }
    if (table === "tags") {
      return {
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([]),
      };
    }
    return {
      status: 200,
      contentType: "application/json",
      body: JSON.stringify([]),
    };
};

test.describe("content list table", () => {
  test("renders rows sorted newest first by default", async ({ page }) => {
    await seedFakeSession(page);
    await mockSupabaseRest(page, twoPostsHandler);

    await page.goto("/content/list/");

    await expect(page.getByText("Newer Post").first()).toBeVisible({
      timeout: 15_000,
    });
    await expect(page.getByText("Older Post").first()).toBeVisible();

    // Default sort is created_at descending: the newer post's row comes first.
    const newerRow = page.getByRole("row", { name: /Newer Post/ });
    const olderRow = page.getByRole("row", { name: /Older Post/ });
    await expect(newerRow).toBeVisible();
    await expect(olderRow).toBeVisible();
    const newerBox = await newerRow.boundingBox();
    const olderBox = await olderRow.boundingBox();
    expect(newerBox!.y).toBeLessThan(olderBox!.y);
  });

  test("clicking Created header toggles to oldest first", async ({ page }) => {
    await seedFakeSession(page);
    await mockSupabaseRest(page, twoPostsHandler);

    await page.goto("/content/list/");
    await expect(page.getByText("Newer Post").first()).toBeVisible({
      timeout: 15_000,
    });

    // First click on the active column flips descending -> ascending.
    await page.getByRole("columnheader", { name: /Created/ }).click();

    const newerRow = page.getByRole("row", { name: /Newer Post/ });
    const olderRow = page.getByRole("row", { name: /Older Post/ });
    const newerBox = await newerRow.boundingBox();
    const olderBox = await olderRow.boundingBox();
    expect(olderBox!.y).toBeLessThan(newerBox!.y);
  });
});
