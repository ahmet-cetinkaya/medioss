/**
 * Move the stage under <body>, lock page scroll and hide the page UI
 * (`player-focus-on`, see global.css). The cleanup puts the stage back.
 */
export function enterFocusOverlay(element: HTMLElement): () => void {
  const parent = element.parentElement;
  const next = element.nextSibling;
  document.body.append(element);
  const previousOverflow = document.body.style.overflow;
  document.body.style.overflow = "hidden";
  document.body.classList.add("player-focus-on");
  return () => {
    document.body.style.overflow = previousOverflow;
    document.body.classList.remove("player-focus-on");
    parent?.insertBefore(element, next);
  };
}

const WAKE_EVENTS = ["pointermove", "pointerdown", "keydown"] as const;

/**
 * Report idle after `idleMs` without input; any input reports awake again.
 * Starts awake. The cleanup stops listening.
 */
export function watchIdle(
  idleMs: number,
  onChange: (idle: boolean) => void,
): () => void {
  let timer: ReturnType<typeof setTimeout>;
  const wake = () => {
    onChange(false);
    clearTimeout(timer);
    timer = setTimeout(() => onChange(true), idleMs);
  };
  for (const name of WAKE_EVENTS) window.addEventListener(name, wake, true);
  wake();
  return () => {
    clearTimeout(timer);
    for (const name of WAKE_EVENTS)
      window.removeEventListener(name, wake, true);
  };
}
