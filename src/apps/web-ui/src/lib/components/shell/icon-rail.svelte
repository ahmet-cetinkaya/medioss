<!--
  @component
  Icon rail for 768–1279 px (DESIGN_SYSTEM §4.3, FR-085, FR-086). Shown with
  `class="hidden md:flex xl:hidden"` by AppShell. Each item keeps its label
  in the DOM (the accessible name) and shows it as a tooltip on hover and
  keyboard focus. Groups follow FR-002 order, separated by dividers.

  The nav itself never clips: an inner box scrolls when the window is short,
  and the tooltip lives outside that box (positioned against the nav), so
  the labels are never cut off at the rail's edge and never add a horizontal
  scrollbar.

  Props: `screen`, `isAdmin` (shows Admin, FR-003), `onSignOut(trigger)`,
  `class`.
-->
<script lang="ts">
  import BrandLockup from "$lib/components/content/brand-lockup.svelte";
  import { t } from "$lib/i18n/locale.svelte";
  import {
    NAV_GROUPS,
    navCurrentFor,
    type ScreenKey,
  } from "$lib/navigation/routes";
  import NavItem from "./nav-item.svelte";
  import { shellMessages } from "./shell.messages";

  type Props = {
    screen: ScreenKey;
    isAdmin: boolean;
    onSignOut: (trigger: HTMLElement) => void;
    class?: string;
  };

  let { screen, isAdmin, onSignOut, class: className = "" }: Props = $props();

  const current = $derived(navCurrentFor(screen));

  // Measured caps so the clamp math never depends on the rendered text.
  const TIP_WIDTH_PX = 220;
  const VIEWPORT_MARGIN_PX = 8;

  let navBox: HTMLElement;
  let tip = $state<{ label: string; top: number; left: number } | null>(null);

  // The item's rect against the nav's: the tooltip is a child of the nav, so
  // its coordinates are nav-relative.
  function showTip(event: Event) {
    const item = (event.target as HTMLElement).closest<HTMLElement>(
      "[data-tip]",
    );
    if (!item || !navBox) return;
    const itemRect = item.getBoundingClientRect();
    const navRect = navBox.getBoundingClientRect();
    // Anchor just right of the item, then clamp into the viewport: a long
    // label beside the left rail would otherwise overflow the window.
    const offsetFromNav = itemRect.right - navRect.left + 12;
    const maxByWindow = window.innerWidth - TIP_WIDTH_PX - VIEWPORT_MARGIN_PX;
    tip = {
      label: item.dataset.tip ?? "",
      top: itemRect.top + itemRect.height / 2,
      left: Math.min(offsetFromNav, Math.max(0, maxByWindow)),
    };
  }

  // Leaving an item (pointer or focus) hides its tooltip unless the pointer
  // or focus moved to the same item's children.
  function hideTip(event: Event) {
    const event_ = event as PointerEvent | FocusEvent;
    const from = event.target as HTMLElement;
    const to = event_.relatedTarget as HTMLElement | null;
    if (to && from.contains(to)) return;
    tip = null;
  }
</script>

<nav
  bind:this={navBox}
  aria-label={t(shellMessages.railNav)}
  class="relative w-22 shrink-0 flex-col items-center gap-4 py-6 {className}"
>
  <div class="max-w-full overflow-hidden">
    <BrandLockup compact href="/" />
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    data-rail-scroll
    class="flex min-h-0 w-22 flex-1 flex-col items-center gap-4 overflow-y-auto overflow-x-hidden [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
    onpointerover={showTip}
    onpointerout={hideTip}
    onfocusin={showTip}
    onfocusout={hideTip}
    onscroll={() => (tip = null)}
  >
    {#each NAV_GROUPS as group, index (group.key)}
      <ul
        class="flex flex-col items-center gap-2 {index > 0
          ? 'border-t border-divider pt-4'
          : ''}"
      >
        {#each group.items.filter((item) => isAdmin || !item.requiresAdmin) as item (item.key)}
          <li>
            {#if item.kind === "screen"}
              <NavItem
                variant="rail"
                href={item.href}
                label={t(item.label)}
                icon={item.icon}
                current={item.key === current}
              />
            {:else}
              <NavItem
                variant="rail"
                label={t(item.label)}
                icon={item.icon}
                onclick={(event) => onSignOut(event.currentTarget)}
              />
            {/if}
          </li>
        {/each}
      </ul>
    {/each}
  </div>
  <!-- The tooltip layer: fixed, so neither the scroll box nor the nav ever
       clips it or counts its width. -->
  <span
    data-rail-tooltip
    class="glass-solid elevation-1 pointer-events-none fixed top-0 start-0 z-40 -translate-y-1/2 rounded-full px-3 py-1.5 text-sm whitespace-nowrap text-ink-strong opacity-0 motion-safe:transition-opacity motion-safe:duration-(--duration-fast) {tip
      ? 'opacity-100'
      : ''}"
    style:left={tip ? `${tip.left}px` : "0px"}
    style:top="{tip?.top ?? -100}px"
  >
    {tip?.label ?? ""}
  </span>
</nav>
