// The meditation name lifted from PlayerContent to the logic-free wrapper,
// which passes it to AppShell as the h1 and document title (FR-043).

export const playerTitle = $state<{ name: string | undefined }>({
  name: undefined,
});
