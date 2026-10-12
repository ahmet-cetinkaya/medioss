// Viewport presets used by the responsive suites (D1 tiers: <768, 768-1279, >=1280).
export const VIEWPORT_WIDTHS = [
  320, 375, 768, 800, 1024, 1100, 1280, 1440,
] as const;

export type ViewportWidth = (typeof VIEWPORT_WIDTHS)[number];

export function viewport(width: ViewportWidth): {
  width: number;
  height: number;
} {
  return { width, height: width < 768 ? 740 : 900 };
}

export const VIEWPORTS = Object.fromEntries(
  VIEWPORT_WIDTHS.map((width) => [width, viewport(width)]),
) as Record<ViewportWidth, { width: number; height: number }>;
