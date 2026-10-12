import { afterEach, describe, expect, test } from "bun:test";
import { ApiError, readErrorBody } from "./api-error";
import { completeSetup, login, logout } from "./identity";
import { request, setRequestDependenciesForTests } from "./request";

type Call = { path: string; init?: RequestInit };

function stub(response: () => Response | Promise<Response>): Call[] {
  const calls: Call[] = [];
  setRequestDependenciesForTests({
    fetch: async (path, init) => {
      calls.push({ path, init });
      return response();
    },
    redirectToLogin: () => {
      throw new Error("unexpected redirect");
    },
  });
  return calls;
}

const flush = () => new Promise((resolve) => setTimeout(resolve, 10));

afterEach(() => setRequestDependenciesForTests({}));

describe("readErrorBody", () => {
  test("JSON error body yields its message and parsed value", async () => {
    const response = Response.json({ error: "email already exists" });
    expect(await readErrorBody(response)).toEqual({
      message: "email already exists",
      data: { error: "email already exists" },
    });
  });

  test("empty body yields an empty message", async () => {
    expect(await readErrorBody(new Response(null, { status: 409 }))).toEqual({
      message: "",
      data: undefined,
    });
  });

  test("text/plain body yields the text", async () => {
    const response = new Response("missing import file", {
      status: 400,
      headers: { "content-type": "text/plain" },
    });
    expect(await readErrorBody(response)).toEqual({
      message: "missing import file",
      data: undefined,
    });
  });

  test("invalid JSON yields the raw text", async () => {
    const response = new Response("{not json", {
      headers: { "content-type": "application/json" },
    });
    expect(await readErrorBody(response)).toEqual({
      message: "{not json",
      data: undefined,
    });
  });
});

describe("request", () => {
  test("a refusal throws ApiError with status, action, and server text", async () => {
    stub(() =>
      Response.json({ error: "email already exists" }, { status: 409 }),
    );
    const error = await request("/api/x", undefined, {
      action: "users.create",
    }).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toBeInstanceOf(Error);
    expect(error).toMatchObject({
      status: 409,
      action: "users.create",
      serverMessage: "email already exists",
      message: "email already exists",
    });
  });

  test("an empty refusal keeps a meaningful message and the generic action", async () => {
    stub(() => new Response(null, { status: 409 }));
    const error = await request("/api/x").catch((e: unknown) => e);
    expect(error).toMatchObject({
      status: 409,
      action: "generic",
      serverMessage: "",
      message: "request failed (409)",
    });
  });

  test("invalid JSON in a successful response still throws ApiError", async () => {
    stub(
      () =>
        new Response("<html>proxy login page</html>", {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
    );
    const error = await request("/api/x", undefined, {
      action: "mood.list",
    }).catch((caught: unknown) => caught);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toMatchObject({
      status: 200,
      action: "mood.list",
      serverMessage: "",
      message: "request failed (200)",
    });
  });

  test("a fetch TypeError becomes ApiError with status 0", async () => {
    stub(() => {
      throw new TypeError("Failed to fetch");
    });
    const error = await request("/api/x", undefined, {
      action: "mood.list",
    }).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toMatchObject({ status: 0, action: "mood.list" });
  });

  test("an AbortError is rethrown unchanged", async () => {
    const abort = new DOMException("aborted", "AbortError");
    stub(() => {
      throw abort;
    });
    expect(await request("/api/x").catch((e: unknown) => e)).toBe(abort);
  });

  test("a 401 redirects to login and never settles", async () => {
    let redirects = 0;
    setRequestDependenciesForTests({
      fetch: async () => Response.json({ error: "x" }, { status: 401 }),
      redirectToLogin: () => {
        redirects++;
        return true;
      },
    });
    let settled = false;
    request("/api/x").then(
      () => (settled = true),
      () => (settled = true),
    );
    await flush();
    expect(redirects).toBe(1);
    expect(settled).toBe(false);
  });

  test("a 401 whose redirect is skipped throws ApiError{status: 401}", async () => {
    setRequestDependenciesForTests({
      fetch: async () => Response.json({ error: "x" }, { status: 401 }),
      redirectToLogin: () => false,
    });
    const error = await request("/api/x").catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toMatchObject({ status: 401, action: "generic" });
  });

  test('a 401 with onUnauthorized "throw" throws ApiError{status: 401}', async () => {
    stub(() => Response.json({ error: "x" }, { status: 401 }));
    const error = await request("/api/x", undefined, {
      onUnauthorized: "throw",
    }).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(401);
  });

  test("sends same-origin credentials; JSON type only for non-FormData", async () => {
    const calls = stub(() => new Response(null, { status: 204 }));
    expect(await request("/api/x", { method: "POST", body: "{}" })).toBe(
      undefined,
    );
    await request("/api/x", { method: "POST", body: new FormData() });
    expect(calls[0].init?.credentials).toBe("same-origin");
    const headers = (i: number) => calls[i].init?.headers as Headers;
    expect(headers(0).get("content-type")).toBe("application/json");
    expect(headers(1).has("content-type")).toBe(false);
  });

  test("Headers and tuple-array headers reach fetch; a set type is kept", async () => {
    const calls = stub(() => new Response(null, { status: 204 }));
    await request("/api/x", {
      method: "POST",
      body: "{}",
      headers: new Headers({ "x-setup-secret": "dummy-secret" }),
    });
    await request("/api/x", {
      method: "POST",
      body: "x",
      headers: [
        ["x-dummy", "dummy-value"],
        ["content-type", "text/plain"],
      ],
    });
    await request("/api/x", {
      method: "POST",
      body: new FormData(),
      headers: [["x-dummy", "dummy-value"]],
    });
    const headers = (i: number) => calls[i].init?.headers as Headers;
    expect(headers(0).get("x-setup-secret")).toBe("dummy-secret");
    expect(headers(0).get("content-type")).toBe("application/json");
    expect(headers(1).get("x-dummy")).toBe("dummy-value");
    expect(headers(1).get("content-type")).toBe("text/plain");
    expect(headers(2).get("x-dummy")).toBe("dummy-value");
    expect(headers(2).has("content-type")).toBe(false);
  });

  test("empty-body successes resolve undefined; JSON bodies parse", async () => {
    stub(() => new Response("", { status: 201 }));
    expect(await request("/api/x")).toBe(undefined);
    stub(() => Response.json({ id: 1 }));
    expect(await request<{ id: number }>("/api/x")).toEqual({ id: 1 });
  });
});

describe("login", () => {
  const throttled = (headers: Record<string, string>, retryAfter: unknown) =>
    stub(() =>
      Response.json(
        { error: "too many attempts", retry_after: retryAfter },
        { status: 429, headers },
      ),
    );

  test("2xx resolves ok", async () => {
    stub(() => Response.json({ id: 1 }));
    expect(await login("a@b.c", "password")).toEqual({ ok: true });
  });

  test("429 uses the Retry-After header first", async () => {
    throttled({ "retry-after": "7" }, 3);
    expect(await login("a@b.c", "password")).toEqual({
      ok: false,
      throttled: true,
      retryAfterSeconds: 7,
    });
  });

  test("429 falls back to the body retry_after", async () => {
    throttled({}, 4);
    expect(await login("a@b.c", "password")).toMatchObject({
      retryAfterSeconds: 4,
    });
  });

  test("429 waits at least 1 second", async () => {
    throttled({}, 0);
    expect(await login("a@b.c", "password")).toMatchObject({
      retryAfterSeconds: 1,
    });
    throttled({ "retry-after": "0" }, null);
    expect(await login("a@b.c", "password")).toMatchObject({
      retryAfterSeconds: 1,
    });
    throttled({}, null);
    expect(await login("a@b.c", "password")).toMatchObject({
      retryAfterSeconds: 1,
    });
  });

  test("401 throws ApiError for the login action and never redirects", async () => {
    stub(() =>
      Response.json({ error: "invalid credentials" }, { status: 401 }),
    );
    const error = await login("a@b.c", "wrong").catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toMatchObject({ status: 401, action: "login" });
  });
});

describe("logout", () => {
  test("204 resolves as signed out now", async () => {
    stub(() => new Response(null, { status: 204 }));
    expect(await logout()).toEqual({ alreadySignedOut: false });
  });

  test("401 resolves as already signed out without redirecting", async () => {
    stub(() => Response.json({ error: "x" }, { status: 401 }));
    expect(await logout()).toEqual({ alreadySignedOut: true });
  });

  test("other failures throw", async () => {
    stub(() => new Response(null, { status: 403 }));
    expect(await logout().catch((e: unknown) => e)).toMatchObject({
      status: 403,
      action: "logout",
    });
  });
});

describe("completeSetup", () => {
  test("sends the secret in the x-setup-secret header only", async () => {
    const calls = stub(() => Response.json({ id: 1 }, { status: 201 }));
    await completeSetup("s3cret", "a@b.c", "password1");
    const headers = calls[0].init?.headers as Headers;
    expect(headers.get("x-setup-secret")).toBe("s3cret");
    expect(JSON.parse(String(calls[0].init?.body))).toEqual({
      email: "a@b.c",
      password: "password1",
    });
  });

  test("a 403 throws ApiError for the setup action", async () => {
    stub(() => Response.json({ error: "forbidden" }, { status: 403 }));
    expect(
      await completeSetup("x", "a@b.c", "password1").catch((e: unknown) => e),
    ).toMatchObject({ status: 403, action: "setup" });
  });
});
