import { afterEach, describe, expect, test } from "bun:test";
import {
  favoriteMeditation,
  fetchFavorites,
  unfavoriteMeditation,
} from "./meditation";
import { setRequestDependenciesForTests } from "./request";

type Call = { path: string; init: RequestInit | undefined };

function stubFetch(response: Response): Call[] {
  const calls: Call[] = [];
  setRequestDependenciesForTests({
    fetch: async (path, init) => {
      calls.push({ path, init });
      return response;
    },
  });
  return calls;
}

afterEach(() => setRequestDependenciesForTests({}));

describe("meditation favorites client", () => {
  test("fetchFavorites reads the favorites list", async () => {
    const calls = stubFetch(Response.json([{ id: 4, favorite: true }]));
    const favorites = await fetchFavorites();
    expect(calls[0].path).toBe("/api/meditation/favorites");
    expect(favorites).toEqual([{ id: 4, favorite: true } as never]);
  });

  test("favoriteMeditation sends PUT to the favorite resource", async () => {
    const calls = stubFetch(new Response(null, { status: 204 }));
    await favoriteMeditation(7);
    expect(calls[0].path).toBe("/api/meditation/7/favorite");
    expect(calls[0].init?.method).toBe("PUT");
  });

  test("unfavoriteMeditation sends DELETE to the favorite resource", async () => {
    const calls = stubFetch(new Response(null, { status: 204 }));
    await unfavoriteMeditation(7);
    expect(calls[0].path).toBe("/api/meditation/7/favorite");
    expect(calls[0].init?.method).toBe("DELETE");
  });

  test("a refused toggle throws", async () => {
    stubFetch(new Response("nope", { status: 404 }));
    await expect(favoriteMeditation(7)).rejects.toThrow();
  });
});
