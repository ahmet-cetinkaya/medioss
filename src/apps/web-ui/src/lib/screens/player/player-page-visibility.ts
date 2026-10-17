/** Hide reports (PB-2): `pagehide` fires on window, not document. */
export function bindPageVisibility(handlers: {
  hide: () => void;
  show: () => void;
}): () => void {
  const { hide, show } = handlers;
  const visibility = () =>
    document.visibilityState === "hidden" ? hide() : show();
  window.addEventListener("pagehide", hide);
  window.addEventListener("pageshow", show);
  document.addEventListener("visibilitychange", visibility);
  return () => {
    window.removeEventListener("pagehide", hide);
    window.removeEventListener("pageshow", show);
    document.removeEventListener("visibilitychange", visibility);
  };
}
