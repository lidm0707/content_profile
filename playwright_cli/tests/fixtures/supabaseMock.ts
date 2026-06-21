import type { Page, Request, Route } from "@playwright/test";

const SUPABASE_REST_PATTERN = /\/rest\/v1\//;
const SUPABASE_REST_GLOB = "**/*";

export interface MockContentRow {
  id: number;
  title: string;
  slug: string;
  body: string;
  status: string;
  created_at: string;
  updated_at: string;
  synced_at: string | null;
}

export const DEFAULT_MOCK_CONTENT: MockContentRow[] = [
  {
    id: 1,
    title: "Mock Post One",
    slug: "mock-post-one",
    body: "---\ntags: []\n---\nHello from mock",
    status: "published",
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
    synced_at: null,
  },
];

export interface MockTagRow {
  id: number;
  name: string;
  slug: string;
  parent_id: number | null;
  created_at: string;
  updated_at: string;
  synced_at: string | null;
}

export const DEFAULT_MOCK_TAGS: MockTagRow[] = [
  {
    id: 10,
    name: "RUST",
    slug: "rust",
    parent_id: null,
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
    synced_at: null,
  },
];

export interface MockResponse {
  status?: number;
  contentType?: string;
  body: string;
  headers?: Record<string, string>;
}

export type RestHandler = (url: URL, request: Request) => MockResponse;

function jsonResponse(
  body: unknown,
  opts: { status?: number; headers?: Record<string, string> } = {},
): MockResponse {
  const exposeHeaders = "content-range";
  return {
    status: opts.status ?? 200,
    contentType: "application/json",
    body: JSON.stringify(body),
    headers: {
      "access-control-expose-headers": exposeHeaders,
      ...(opts.headers ?? {}),
    },
  };
}

function isPaginatedWithCount(request: Request): boolean {
  const prefer = request.headers()["prefer"] ?? "";
  const url = new URL(request.url());
  const hasCountExact = prefer
    .split(",")
    .map((p) => p.trim())
    .includes("count=exact");
  const hasOffsetLimit =
    url.searchParams.has("offset") || url.searchParams.has("limit");
  return hasCountExact || hasOffsetLimit;
}

function contentRangeHeader(total: number, returned: number): string {
  const end = returned > 0 ? returned - 1 : 0;
  return `0-${end}/${total}`;
}

function defaultRestHandler(url: URL, request: Request): MockResponse {
  const table = url.pathname.split("/rest/v1/")[1]?.split("?")[0];
  const method = request.method();
  const paginated = isPaginatedWithCount(request);

  if (table === "content") {
    if (method === "GET") {
      const headers = paginated
        ? {
            "content-range": contentRangeHeader(
              DEFAULT_MOCK_CONTENT.length,
              DEFAULT_MOCK_CONTENT.length,
            ),
          }
        : undefined;
      return jsonResponse(DEFAULT_MOCK_CONTENT, { headers });
    }
    if (method === "POST") {
      const sent = JSON.parse(request.postData() ?? "{}");
      return jsonResponse([
        {
          ...DEFAULT_MOCK_CONTENT[0],
          ...sent,
          id: 999,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ]);
    }
  }

  if (table === "tags") {
    const headers = paginated
      ? {
          "content-range": contentRangeHeader(
            DEFAULT_MOCK_TAGS.length,
            DEFAULT_MOCK_TAGS.length,
          ),
        }
      : undefined;
    return jsonResponse(DEFAULT_MOCK_TAGS, { headers });
  }

  if (table === "content_tags") {
    return jsonResponse([]);
  }

  return jsonResponse([]);
}

export async function mockSupabaseRest(
  page: Page,
  handler: RestHandler = defaultRestHandler,
): Promise<void> {
  await page.route(SUPABASE_REST_GLOB, (route: Route) => {
    const request = route.request();
    const url = new URL(request.url());
    if (!SUPABASE_REST_PATTERN.test(url.pathname)) {
      return route.continue();
    }
    try {
      const result = handler(url, request);
      return route.fulfill(result);
    } catch {
      return route.continue();
    }
  });
}
