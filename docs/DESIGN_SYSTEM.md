# Design System — "Soft Glass"

This document describes the visual language every MediOSS screen follows. It is derived from two reference shots of a meditation dashboard (a full desktop dashboard and an exploded component sheet of the same design). The references are **inspiration only**: their brand ("Sense"), copy, photography, and features that MediOSS does not have are not reproduced. Section 11 maps the patterns onto real MediOSS screens.

All numeric values are **estimated from screenshots** and normalized to a 1440 px wide desktop viewport. Treat them as the starting point of the token set, then tune them in the browser; the tokens in section 3 are the single source of truth once implemented.

---

## 1. Design Principles

1. **Calm over dense.** Generous whitespace, few elements per view, one clear focal point. The page title and one featured card carry the hierarchy; everything else recedes.
2. **Soft depth, not hard edges.** Surfaces are translucent white "frosted glass" layered on a warm pastel gradient. Depth comes from blur, a bright 1 px rim, and diffuse low-opacity shadows — never from dark borders or solid fills.
3. **Everything is round.** Pills for controls, large radii for cards, circles for avatars and icon buttons. No sharp corners anywhere in the UI.
4. **Color lives in imagery, not chrome.** The interface itself is near-monochrome (white, warm grays, near-black text). Saturated color comes from abstract pastel imagery and a single purple accent used sparingly for "act here" signals.
5. **One accent, one meaning.** Purple marks the active location and primary action (active nav icon, open-arrow, play). It is never decoration. Warm orange remains only where it has a separate data or imagery meaning.
6. **Modular by construction.** The component sheet shows every piece working as a standalone card. Components must render correctly in isolation and compose without layout-specific overrides.

---

## 2. Visual Language

### 2.1 Canvas

The page background is a large, soft, multi-stop gradient that drifts slowly across warm pastels when motion is allowed. Use the same vivid static gradient when reduced motion is preferred. Do not add a white fade at the top:

| Region         | Approx. color             | Role                       |
| -------------- | ------------------------- | -------------------------- |
| Top / top-left | `#E8E6E6` light warm gray | Calm, neutral reading area |
| Bottom-left    | `#EBC0A6` peach           | Warmth                     |
| Bottom-center  | `#EAA8B8` rose            | Transition                 |
| Bottom-right   | `#C9AAF2` lavender        | Cool counterweight         |

The gradient sits _behind_ the glass shell, so it tints every translucent surface. Keep it low-contrast: it must never compete with content.

### 2.2 Glass surfaces

Three surface levels, all white at different opacities:

| Level | Usage | Fill | Rim | Blur |
| --- | --- | --- | --- | --- |
| `surface-shell` | Outer app shell | white ≈ 30–40 % | 1.5 px white ≈ 70 % | strong (≈ 24–40 px) |
| `surface-panel` | Side panels, standalone cards (assistant panel, sidebar card) | white ≈ 40–55 % | 1 px white ≈ 80 % | medium |
| `surface-solid` | Interactive controls: active nav pill, selected chip, search field, icon buttons, now-playing pill, input | white ≈ 85–100 % | 1 px white | none needed |

The rule of thumb: **the more interactive, the more opaque.** Containers are see-through; things you click are solid white.

### 2.3 Depth and elevation

Shadows are wide, soft, low-opacity and slightly warm (pink/mauve tinted rather than black):

| Token | Use | Approx. value |
| --- | --- | --- |
| `elevation-0` | Flat items (everything inside a surface: buttons, chips, icon buttons, active nav pill, badges, search field, avatars, slider thumbs) | none |
| `elevation-1` | The outermost content layer: content-card thumbnails, standard tiles and bento cards sitting directly on a panel | `0 6px 20px -8px rgb(150 100 120 / 0.18)` |
| `elevation-2` | True overlay surfaces: app shell, bottom bar, dialogs, sheets, menus, skip link | `0 16px 40px -16px rgb(150 100 120 / 0.22)` |
| `elevation-3` | The single featured card per view | `0 30px 60px -20px rgb(150 100 120 / 0.30)` |

Elevation is a hierarchy signal with two rules. Only **one** element per view uses `elevation-3`, and **shadows never stack**: an elevation belongs to the outermost surface of a nesting chain, and everything inside that surface is flat (`elevation-0`). A filled control reads as elevated through its fill and rim alone — never its own shadow. Stacked shadows (a shadow-casting button on a shadow-casting card on a shadow-casting panel) are the visual defect this rule exists to prevent.

### 2.4 The orb

A glossy sphere is the brand mark and the system's "voice":

- Layered radial gradients: white specular highlight at top-left → lavender core (`#EEE6FF`) → amethyst (`#B58AE6`) → deep purple rim (`#67418F`). The rim stays darker than every canvas stop, so the mark does not dissolve into the page.
- Inset light and shade (bright top edge, deep purple underside, thin white edge ring) make it read as glass without `backdrop-filter`, so it renders the same under reduced transparency. It has no animation.
- Soft purple glow beneath it (large blur), so it appears to float.
- The orb and action chrome share a purple family, but their roles differ: the orb is decorative (`aria-hidden`), while controls and active-location marks carry the action accent. Its midpoint, glow, and rim inform the accent ladder. Orange remains in the mood scale and warm imagery, not action chrome. In forced-colors mode the orb gets a visible border.
- Appears at three sizes: **logo** (≈ 40 px, beside the wordmark), **hero** (≈ 64–72 px, above the greeting, on a dotted halo), **inline avatar** (≈ 16 px, before the speaker name in a feed item).
- The current user is represented by a deeper, cooler purple orb variant (`--gradient-orb-user`) or by their avatar image when one exists.

### 2.5 Dot halo pattern

Behind the hero orb sits a fine dot grid (≈ 1.5 px dots, ≈ 10 px pitch, light gray) masked by a radial fade, so it is visible near the orb and dissolves toward the edges. It is the only texture in the system; use it only behind the hero greeting and, optionally, faintly behind feed content in the same panel.

### 2.6 Imagery

All imagery is **abstract, motion-blurred, dreamy pastel** — long-exposure light, soft mountain silhouettes, blurred petals. No people, no literal objects, no hard focus. Palettes per category observed: pink/violet/orange, mint/sage, blue/peach, green/peach, coral/orange.

Imagery always has its **bottom edge washed to white** (a vertical gradient from transparent to white ≈ 70–85 %) where a title sits on it, which is what keeps dark text legible on top of an image.

---

## 3. Design Tokens

### 3.1 Color

| Token | Approx. value | Use |
| --- | --- | --- |
| `--text-strong` | `#2A2A2A` | Page title, card titles, active nav label, speaker names |
| `--text-body` | `#4A4A4A` | Nav labels, body copy, feed messages |
| `--text-muted` | `#575757` | Meta lines (author, duration), placeholders, section labels |
| `--text-faint` | `#6E6E6E` | Feedback icons, separators in meta lines |
| `--accent` | `#B58AE6` | Medium purple fill with dark foreground; never white text or indicators |
| `--accent-soft` | `#B58AE6` at 12 % | Soft purple backgrounds |
| `--accent-strong` | `#67418F` | Accessible action text, active nav icon, focus ring and button hover (>= 4.5:1 on tested glass) |
| `--accent-fill` | `#7E53AD` | Primary action fill (white label 5.63:1; 3.65:1 on tested glass) |
| `--chart-mark` | `#7E53AD` | Chart accent marks (>= 3:1; measured minimum 3.65:1) |
| `--surface-solid` | `#FFFFFF` | Controls (see 2.2) |
| `--surface-panel` | `rgb(255 255 255 / 0.45)` | Panels |
| `--surface-shell` | `rgb(255 255 255 / 0.35)` | Shell |
| `--rim` | `rgb(255 255 255 / 0.75)` | 1 px glass borders |
| `--divider` | `rgb(0 0 0 / 0.08)` | Hairline separators inside panels |
| `--chip-outline` | `rgb(255 255 255 / 0.8)` + `rgb(0 0 0 / 0.04)` | Unselected chip border |

Destructive and success states are not shown in the references. Derive them in the same muted register (desaturated red / sage green), never pure saturated hues.

### 3.2 Gradients

| Token | Definition (approx.) |
| --- | --- |
| `--gradient-canvas` | Diagonal from `#E8E6E6` (top-left, 0–35 %) through `#EBC0A6` (bottom-left) and `#EAA8B8` (bottom-center) to `#C9AAF2` (bottom-right) |
| `--gradient-orb` | Three layered radial gradients: white specular (top-left), soft white bounce light (bottom-right), and a lavender `#EEE6FF` → amethyst `#B58AE6` → deep purple `#67418F` body. `--gradient-orb-user` is the deeper, cooler purple variant. |
| `--gradient-image-wash` | `linear-gradient(to bottom, transparent 45%, rgb(255 255 255 / 0.85))` |
| `--gradient-panel-tint` | Panel fill fading from near-white at the top to a faint lavender at the bottom |

### 3.3 Typography

A geometric sans-serif with round forms (the reference matches **Poppins**). The font must be **self-hosted** (bundled woff2, no font CDN) to keep the app free of third-party requests.

| Style | Size / line-height | Weight | Notes |
| --- | --- | --- | --- |
| `display` | 60 / 1.05 | 600 | Page title ("Dashboard"); tight tracking (≈ -0.02em) |
| `greeting` | 34 / 1.2 | 400 with 700 for the name | Centered; mixed weight on one line ("Good morning **Ahmet**, how are you feeling?") |
| `title-lg` | 24 / 1.2 | 500 | Text on category tiles |
| `heading` | 20 / 1.3 | 600 | Section headings ("Recommended") |
| `brand` | 28 / 1 | 500 | Wordmark beside the logo orb |
| `body-lg` | 16 / 1.5 | 400 (600 when active) | Nav items, card titles |
| `body` | 15 / 1.7 | 400 | Feed messages, descriptions |
| `label` | 14 / 1.2 | 500 (600 when selected) | Chips, button labels |
| `meta` | 13 / 1.3 | 400 | Author, duration; muted color |
| `overline` | 11 / 1 | 500 | Uppercase sidebar group labels ("GENERAL"); tracking ≈ +0.06em; muted color |

Contrast comes from size and weight, not color: most text is one of two grays.

### 3.4 Spacing

8 px base grid. Observed rhythm:

| Token      | Value | Typical use                                        |
| ---------- | ----- | -------------------------------------------------- |
| `space-1`  | 4 px  | Icon-to-text in meta lines                         |
| `space-2`  | 8 px  | Chip gap, small inner gaps                         |
| `space-3`  | 12 px | Icon-to-label in nav and buttons                   |
| `space-4`  | 16 px | Card inner padding (small), stacked text           |
| `space-5`  | 20 px | Tile grid gap, content card gap                    |
| `space-6`  | 24 px | Panel inner padding                                |
| `space-8`  | 32 px | Shell inner padding, column gap                    |
| `space-12` | 48 px | Between major sections (tile grid → "Recommended") |

### 3.5 Radius

| Token | Value | Use |
| --- | --- | --- |
| `radius-full` | 9999 px | Pills: nav pill, chips, search, inputs, now-playing card; circles: icon buttons, avatars, orb |
| `radius-thumb` | 20 px | Content card thumbnails, tile inner image |
| `radius-tile` | 28 px | Tile outer frame |
| `radius-panel` | 32 px | Panels and standalone cards |
| `radius-shell` | 40 px | App shell |

Nested radii follow `inner = outer − padding` so frames stay concentric (tile: 28 px frame, 6–8 px rim, ≈ 20–22 px image).

### 3.6 Sizing

| Element                              | Size                 |
| ------------------------------------ | -------------------- |
| Icon (nav, inputs)                   | 22 px, 1.5 px stroke |
| Icon (meta, feedback)                | 16 px                |
| Icon button                          | 48 px circle         |
| Small overlay badge                  | 28 px circle         |
| Tile action button (open arrow)      | 44 px circle         |
| Control height (search, chip, input) | 44–48 px             |
| Nav item height                      | 56 px                |
| Minimum touch target                 | 44 × 44 px           |

### 3.7 Motion

Not visible in static references; proposed to match the calm tone:

- Durations 150 ms (hover/press) to 300 ms (panel/tile transitions); easing `cubic-bezier(0.22, 1, 0.36, 1)` (soft ease-out).
- Hover on tiles and cards: lift by 2–4 px; image may scale to 1.02 inside its frame. Never add or step up elevation on hover — a hover shadow landing on an already-shadowed parent is stacked shadow (see §2.3). Buttons change fill on hover only.
- Press: scale to 0.98.
- The canvas color drift uses a 90-second cycle and moves continuously at a subtle pace. No bouncing, no spinning, no attention-grabbing loops. All motion is disabled under `prefers-reduced-motion: reduce`.

---

## 4. Layout

### 4.1 App shell (desktop)

The whole app sits inside one rounded glass shell floating on the gradient canvas, with ≈ 48 px of canvas visible around it.

```text
┌──────────────────────────────────────────────────────────────────────────┐
│  ● Brand            Page Title                 [ Search… ] (🔔) (👤)     │
│                                                                          │
│  OVERLINE          ┌───────────────── main ─────────────┐ ┌── aside ───┐ │
│  ▣ Active item     │  bento tile grid                   │ │  orb       │ │
│  ○ Item            │                                    │ │  greeting  │ │
│  ○ Item            │                                    │ │  feed      │ │
│                    │  Heading            (chip)(chip)   │ │  now-play  │ │
│  OVERLINE          │  [card] [card] [card]              │ │  [input  ] │ │
│  ○ Item            └────────────────────────────────────┘ └────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
   sidebar ≈ 248 px        main: fluid                       aside ≈ 384 px
```

- **Sidebar** has no panel background of its own on the full dashboard; it sits directly on the shell. In the component sheet it also works as a standalone `surface-panel` card, and it can split into two cards (one per group).
- **Header row** spans main + aside: page title left-aligned with the main column, utilities (search, icon buttons) right-aligned above the aside.
- **Aside** is a full-height `surface-panel` card; its input is pinned to the bottom.

### 4.2 Bento tile grid

Category tiles form an asymmetric bento grid, not a uniform grid:

```text
┌───────────── featured (≈ 2/3) ────────┐ ┌─ tall (≈ 1/3) ─┐
│                                       │ │                │
│  Title                          (↗)   │ │  Title         │
└──────┬──────────────────────────┬─────┘ └────────────────┘
       │ narrow                   │ ┌──────── wide ────────┐
       │  Title                   │ │  Title               │
       └──────────────────────────┘ └──────────────────────┘
```

- The **featured tile** is larger, uses `elevation-3`, carries the open-arrow button, and **overlaps** the tile below it by ≈ 24–32 px (higher `z-index`). That overlap is deliberate: it makes one destination unmistakably primary.
- Tiles in a row share a bottom edge; widths vary to avoid a mechanical rhythm.
- Use 4–5 tiles maximum. More destinations belong in navigation, not in the bento.

### 4.3 Responsive behavior (proposed)

The references are desktop-only; MediOSS is mobile-first, so the system defines these adaptations:

| Breakpoint | Layout |
| --- | --- |
| ≥ 1280 px | Three columns as in 4.1 |
| 1024–1279 px | Sidebar collapses to an icon rail (icons only, tooltips, active pill becomes a circle); aside stays |
| 768–1023 px | Icon rail; aside moves below main as a full-width panel |
| < 768 px | No shell inset (shell fills the viewport with `radius-shell` only at the top); header becomes a compact top bar (brand orb + smaller wordmark + icon buttons); navigation becomes a bottom bar of 4–5 pill items; bento becomes a 2-column grid with the featured tile spanning both columns (no overlap); content cards become a horizontal scroll row with translucent glass-panel backdrops |

---

## 5. Components

Each entry lists anatomy, specification, states, and usage rules. Components live under `src/apps/web-ui/src/lib/components/ui/` (primitives) or `src/lib/components/` (composites) per the existing structure.

### 5.1 Brand lockup

- **Anatomy:** logo orb (≈ 40 px, with glow) + wordmark "MediOSS" (`brand` style, `--text-strong`), gap 16 px.
- **Usage:** top of the sidebar, and in the compact mobile top bar (orb + smaller wordmark).

### 5.2 Sidebar navigation

- **Group label:** `overline` style, uppercase, `--text-muted`, 24 px below the previous group, 16 px above its first item.
- **Nav item:** 56 px tall, full sidebar width, `radius-full`, 22 px line icon + `body-lg` label, 12 px gap, 24 px horizontal padding, `--text-body`.
- **States:**
  - _Default:_ transparent, icon stroked in `--text-body`.
  - _Hover:_ `surface-solid` at ≈ 50 %, no shadow.
  - _Active (current page):_ `surface-solid` pill, flat; label weight 600 `--text-strong`; icon switches to its **filled** variant in `--accent-strong`. Set `aria-current="page"`.
  - _Focus-visible:_ 2 px `--accent-strong` ring offset 2 px (see section 8).
- **Rules:** exactly one active item; the sign-out item sits in the last group and uses the same item style (no red), but performs a confirmed action.

### 5.3 Header utilities

- **Search field:** `surface-solid` pill, 48 px tall, ≈ 300 px wide, 1 px `--rim`, flat; leading 20 px search icon in `--text-muted`, placeholder "Search…" in `--text-muted`, 16 px horizontal padding.
- **Icon button:** 48 px `surface-solid` circle, 1 px `--rim`, flat, 22 px line icon in `--text-strong`. Requires an `aria-label`. Notification state: an 8 px `--accent-strong` dot at the top-right of the icon.
- **Avatar button:** same as icon button; shows the user's avatar image cropped to the circle when one exists, otherwise a user icon.

### 5.4 Page title

- `display` style, `--text-strong`, left-aligned with the main column, vertically centered with the header utilities. One per page; it is the page's `<h1>`.

### 5.5 Category tile

The signature component.

- **Anatomy:** outer frame (`surface-solid` at ≈ 70 %, `radius-tile`, 6–8 px padding, 1 px `--rim`) → inner image (`radius-thumb`, abstract artwork, `--gradient-image-wash` at the bottom) → title (`title-lg`, `--text-strong`) at bottom-left, 24 px from the inner edges.
- **Variants:**
  - _Standard:_ `elevation-1` (the tile is the outermost content layer).
  - _Featured:_ `elevation-3`, larger, overlaps its neighbor, carries a **tile action button** at bottom-right: 44 px white circle, flat, with an 18 px `↗` arrow in `--accent-strong`.
- **States:** hover lifts (section 3.7); focus-visible ring follows the outer radius. The whole tile is one link; the arrow button is decorative inside it (`aria-hidden`), not a second tab stop.

### 5.6 Filter chip group

- **Anatomy:** a row of pills, 44 px tall, 18 px horizontal padding, `label` style, 8 px gap. Placed on the same row as a section heading, right-aligned.
- **States:**
  - _Selected:_ `surface-solid`, flat, weight 600, `--text-strong` (selection shows through fill and weight, not shadow).
  - _Unselected:_ transparent fill, 1 px `--chip-outline` border, weight 400, `--text-body`.
  - _Hover (unselected):_ `surface-solid` at ≈ 50 %.
- **Semantics:** single-select filters are a `radiogroup` (or toggle buttons with `aria-pressed`); the selected state must not rely on fill alone — the weight change is part of the signal.

### 5.7 Content card

- **Anatomy:** thumbnail (≈ 4:3, `radius-thumb`, `elevation-1`) with an optional **overlay badge** at top-right (28 px white circle, 14 px icon) → title (`body-lg`, weight 500, `--text-strong`, 12 px below thumbnail) → meta line.
- **Overlay badge:** the reference uses it for a bookmark; MediOSS has no save/bookmark feature, so the slot shows a completion check (`completed`) or stays empty. If it ever becomes interactive it is a separate button with an `aria-label` and `aria-pressed`.
- **Meta line:** `meta` style, `--text-muted`, items separated by a `•` (bullet) or a 1 px × 12 px vertical divider, 6–8 px gaps. Example: `10 min • 40 %` (duration and progress) or `Program title • 10 min`.
- **Usage:** horizontal rows of 3–4 cards under a section heading. The card is one link.
- **Overlay variant (`overlay`):** an image-first variant where the title and meta line sit on the cover over a dark scrim (`--gradient-image-scrim`) instead of below it; the ink flips to `--ink-on-image` / `--ink-on-image-muted`. Use it only where the artwork is the content — currently the Library's program and search-result cards. The default (text-below) variant stays the choice for the admin panels and horizontal compact rows.

### 5.8 Greeting hero

- **Anatomy:** dot halo (section 2.5) → hero orb (64–72 px, glowing) → greeting text (`greeting` style, centered, max ≈ 12 words, name in weight 700).
- **Usage:** top of the aside panel on the home screen; time-of-day aware ("Good morning / afternoon / evening"). Never more than one per screen.

### 5.9 Feed item

The reference uses this for a chat; MediOSS uses the same anatomy for non-chat feeds (daily suggestion, reflections, mood notes).

- **Anatomy:** header row (16 px orb or avatar + speaker/source name in weight 600 `--text-strong`) → body text (`body` style, `--text-body`, line-height 1.7) → optional action row (16 px icons in `--text-faint`, 12 px gap).
- **Alignment:** items authored by the current user align right (header and text right-aligned); system items align left.
- **No bubbles.** Items are plain text on the panel, separated by a 1 px `--divider` hairline with 20 px vertical spacing.

### 5.10 Now-playing / suggestion pill

- **Anatomy:** `surface-solid` pill (`radius-full`, ≈ 80 px tall, 12 px padding, flat) → 56 px circular thumbnail → text stack (title `body-lg` weight 600; meta line with a vertical divider, e.g. `Meditation | 10 min`) → trailing 40 px play button (white circle, flat, solid `--accent-fill` triangle).
- **States:** playing → pause glyph; loading → the glyph is replaced by a subtle ring, not a spinner animation that ignores reduced motion.
- **Usage:** inside the aside (daily suggestion, continue program) and as the persistent mini-player.

### 5.11 Pill input

- **Anatomy:** `surface-solid` pill, 52 px tall, 20 px horizontal padding, placeholder in `--text-muted`, trailing icon buttons (20 px icons in `--text-body`, 12 px apart).
- **Usage:** pinned to the bottom of the aside panel. Multi-line text grows upward with the radius dropping to `radius-panel` once it exceeds one line.

### 5.12 Panels

- **Anatomy:** `surface-panel`, `radius-panel`, 1 px `--rim`, 24 px padding, optional `--gradient-panel-tint`.
- **Usage:** the aside, standalone sidebar cards, and any grouped content (forms, statistics, settings sections). Panels never nest panels; group inside a panel with spacing and `--divider`.

### 5.13 Buttons (derived)

The references show no text buttons; the system extends its patterns:

| Variant | Style |
| --- | --- |
| Primary | `--accent-fill`, white `label` text, `radius-full`, 44 px tall, flat |
| Secondary | `surface-solid` fill, `--text-strong`, 1 px `--rim`, flat (same look as a selected chip) |
| Ghost | Transparent, `--text-body`, hover `surface-solid` at ≈ 50 % (same look as an inactive nav item) |
| Icon | Section 5.3 icon button |

Use at most one primary button per view.

---

## 6. Iconography

- **Style:** outline icons, 1.5 px stroke, round caps and joins, 24 px grid rendered at 22 px. Filled variants only for active/selected states (active nav item, completion check, play glyph).
- **Color:** inherits text color; `--accent-strong` for active icons and `--accent-fill` for primary-action icons.
- **Set:** one consistent open-source set, bundled locally (no icon CDN). The existing UI has no icon library yet; choosing one is an open decision (section 12).
- Icon-only controls always carry an `aria-label`; decorative icons next to a text label are `aria-hidden`.

---

## 7. Imagery Guidelines

- **Default artwork is generated, not photographed.** Category tiles and missing covers use CSS "aurora" backgrounds: 2–3 large blurred radial gradients per category palette (section 2.6), plus the white wash. This keeps the repository free of licensed photography and costs no network requests.
- **User content wins.** Programs can have an uploaded cover (`has_cover`; meditations inherit it via `program_has_cover`). When one exists, use it in thumbnails and tiles with the same radius and, where text overlays it, the same white wash. Meditations without a program cover use the generated artwork.
- **Per-category palettes** (proposed): meditation → pink/violet/orange; sleep → mint/sage/peach; soundscape → green/peach; mood → coral/orange; statistics → blue/peach.
- Artwork is decorative: `alt=""` unless it conveys content.

---

## 8. Accessibility (WCAG 2.1 AA)

MediOSS commits to WCAG 2.1 AA. The soft aesthetic needs these guardrails:

- **Contrast:** `--text-muted` on a translucent surface over the pastel canvas is the main risk. Verify every text token against the _actual composited_ background (worst case: bottom-right lavender). Body text needs 4.5:1, large text (≥ 24 px, or ≥ 19 px bold) needs 3:1. Darken `--text-muted` before reducing its use.
- **Text on imagery:** always use the white wash; never place text on unwashed artwork.
- **Focus:** a visible 2 px `--accent-strong` focus ring (offset 2 px, following the element's radius) on every interactive element; never remove outlines without a replacement.
- **State without color:** selected and active states change weight and/or icon fill in addition to color.
- **Reduced transparency:** under `prefers-reduced-transparency: reduce`, and where `backdrop-filter` is unsupported, glass surfaces fall back to opaque near-white fills (`#F7F5F5`, panel `#FBFAFA`).
- **Reduced motion:** honor `prefers-reduced-motion` (section 3.7).
- **Touch targets:** at least 44 × 44 px, including small overlay controls (pad their hit area beyond the visual circle).
- **Semantics:** one `<h1>` per page (the page title), landmarks for `nav`, `main`, and `aside`, `aria-current="page"` on the active nav item.

---

## 9. Dark Mode

The references are light-only and the system does not define a dark theme yet. The existing `.dark` variables in `global.css` stay untouched until a dark variant (deep plum canvas, smoked-glass surfaces) is designed explicitly.

---

## 10. Implementation Guidance

- **Tokens first.** Implement section 3 as CSS custom properties in `src/apps/web-ui/src/styles/global.css` and expose them through the existing `@theme inline` block, mapping onto shadcn variables so vendored components inherit the look: `--background` → canvas base, `--card` → `surface-panel`, `--primary` → `--accent-fill`, `--foreground` → `--text-strong`, `--muted-foreground` → `--text-muted`, `--border` → `--rim`, `--ring` → `--accent-strong`, `--radius` → panel radius. Add the remaining tokens (surfaces, elevation, gradients) as new theme entries rather than overloading shadcn names.
- **Recipes, not one-off utilities.** Encode each surface and elevation as a reusable class or component variant (for example a `glass-panel` and `glass-solid` utility, `tailwind-variants` variants on `button`/`card`) so screens never hand-assemble `bg-white/40 backdrop-blur-2xl border-white/70 shadow-[...]` chains.
- **Shell once.** Build the app shell (canvas + glass shell + sidebar + header + optional aside) as a layout used by every authenticated page; pages provide only the main column (and, optionally, aside content). `AuthLayout.astro` (login, setup) uses the canvas and a single centered panel without navigation.
- **Font and icons are bundled** with the frontend build; nothing is fetched from a third-party origin at runtime.

---

## 11. Mapping to MediOSS Screens

Navigation (sidebar groups) built only from features that exist:

| Group | Items (page) |
| --- | --- |
| Practice | Home (`index`), Library (`library`), Soundscape (`soundscape`), Mood (`mood`) |
| Progress | History (`history`), Statistics (`stats`) |
| Account | Settings (`settings`), Admin (`admin`, admins only), Sign out |

Program and player pages (`program`, `player`) are reached from content, not from navigation.

| Screen | Patterns |
| --- | --- |
| Home | Page title; bento tiles (featured: Library/Meditate; others: Soundscape, Mood, History or Statistics); "Recent programs" content-card row with program-category chips; aside with greeting hero, daily suggestion and continue-program as now-playing pills, recent mood notes as feed items, pill input for a quick mood note |
| Library | Search in header; content-card grid of meditations; program rows with chips for program categories |
| Program detail | Featured tile as hero (program cover + open/start action), session list as feed items or now-playing pills |
| Player | Full-height panel: large artwork with white wash, now-playing controls scaled up, notes as feed items with a pill input |
| Soundscape | Sounds as standard tiles grouped by category chips; active mix as a panel with per-sound sliders; mini-player pill |
| Mood | Score selector as a chip group (or orb-like selectable circles), note in a pill input, history as feed items, trend chart in a panel |
| History / Statistics | Panels containing charts and summary figures; range filters as chips |
| Settings | Stacked panels per section; derived buttons (5.13); avatar button preview |
| Admin | Panels with tables/forms; upload and download actions as secondary buttons |
| Login / Setup | Canvas + brand lockup + one centered panel with pill inputs and a primary button |

The reference's AI chat assistant has **no MediOSS counterpart** (the app is privacy-first and has no assistant). Its visual patterns — greeting hero, feed items, pill input — are reused for real features as listed above; no chat feature is implied.

---

## 12. Open Decisions

These must be settled before or during the first screen implementation:

1. **Typeface:** Poppins (closest match, OFL-licensed) vs. another self-hostable geometric sans; packaging (npm font package vs. vendored woff2).
2. **Icon set:** which open-source outline set to bundle (it needs both outline and filled variants for active states).
3. **Aside on non-home screens:** keep a contextual aside everywhere, or only on Home.
4. **Exact token values:** confirm colors and contrast in the browser (section 8) and freeze them in `global.css`.
5. **Dark mode:** whether it is in scope for the redesign (section 9).
