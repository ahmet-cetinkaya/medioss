// The Library search query, kept in the address as `?q=` (plan D10, spec
// FR-033). Shared by the shell's header search and the Library content.
// No browser access at module top level: consumers call `connect()` from
// `onMount`; it reads `q` and follows back/forward (`popstate`).

const PARAM = "q";

function readQuery(): string {
  return new URLSearchParams(window.location.search).get(PARAM)?.trim() ?? "";
}

class LibraryQuery {
  #query = $state("");
  #connections = 0;

  /** The active query, trimmed; "" means no search. */
  get query(): string {
    return this.#query;
  }

  #onPopState = (): void => {
    this.#query = readQuery();
  };

  /** Starts following the address; returns the matching disconnect. */
  connect = (): (() => void) => {
    if (this.#connections++ === 0) {
      this.#query = readQuery();
      window.addEventListener("popstate", this.#onPopState);
    }
    let connected = true;
    return () => {
      if (!connected) return;
      connected = false;
      if (--this.#connections === 0)
        window.removeEventListener("popstate", this.#onPopState);
    };
  };

  /** Pushes `?q=<query>`; a whitespace-only query clears instead. */
  submit = (query: string): void => {
    const next = query.trim();
    if (next === "") return this.clear();
    this.#push(next);
  };

  /** Pushes the address without `q`. */
  clear = (): void => {
    this.#push(null);
  };

  #push(next: string | null): void {
    const url = new URL(window.location.href);
    if (next === null) url.searchParams.delete(PARAM);
    else url.searchParams.set(PARAM, next);
    if (url.href !== window.location.href)
      window.history.pushState(window.history.state, "", url);
    this.#query = next ?? "";
  }
}

export const libraryQuery = new LibraryQuery();
