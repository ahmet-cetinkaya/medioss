import type { LibraryFilter } from "./library-groups";

/** The Library filter, shared between the header dropdown and the content
 *  (favorites list), because they live in different components. */
export const libraryFilter = $state<{ value: LibraryFilter }>({ value: "all" });
