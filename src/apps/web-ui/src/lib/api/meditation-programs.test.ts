import { afterEach, describe, expect, test } from "bun:test";
import {
  deleteMeditation,
  deleteProgram,
  renameMeditation,
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

describe("meditation programs client", () => {
  test("deleteProgram sends DELETE to the program resource", async () => {
    const calls = stubFetch(new Response(null, { status: 204 }));
    await deleteProgram(12);
    expect(calls[0].path).toBe("/api/meditation/programs/12");
    expect(calls[0].init?.method).toBe("DELETE");
  });

  test("a refused delete throws", async () => {
    stubFetch(new Response("nope", { status: 404 }));
    await expect(deleteProgram(12)).rejects.toThrow();
  });

  test("deleteMeditation sends DELETE to the meditation resource", async () => {
    const calls = stubFetch(new Response(null, { status: 204 }));
    await deleteMeditation(5);
    expect(calls[0].path).toBe("/api/meditation/5");
    expect(calls[0].init?.method).toBe("DELETE");
  });

  test("renameMeditation sends the name to the meditation name resource", async () => {
    const calls = stubFetch(Response.json({ id: 5, name: "New" }));
    const renamed = await renameMeditation(5, "New");
    expect(calls[0].path).toBe("/api/meditation/meditations/5/name");
    expect(calls[0].init?.method).toBe("PUT");
    expect(calls[0].init?.body).toBe(JSON.stringify({ name: "New" }));
    expect(renamed.name).toBe("New");
  });
});
