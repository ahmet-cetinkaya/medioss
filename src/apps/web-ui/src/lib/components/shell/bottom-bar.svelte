<!--
  @component
  Mobile bottom bar below 768 px (FR-087): Home, Library, Soundscape, Mood,
  and More. Shown with `class="flex md:hidden"` by AppShell; fixed to the
  bottom edge above `var(--safe-area-bottom)`.

  More is a button that opens the More sheet. When the current screen lives
  in the sheet, More carries `aria-current="page"` (a valid global ARIA
  state on a button; screen readers announce it as "current page") plus the
  filled icon and weight 600, so each visible surface has exactly one current
  item.

  Props: `screen`, `isAdmin`, `onSignOut(trigger)` (called with the More
  button after the sheet has closed), `class`.
-->
<script lang="ts">
  import { t } from "$lib/i18n/locale.svelte";
  import {
    isMoreCurrent,
    MOBILE_PRIMARY,
    MORE_ICON,
    MORE_LABEL,
    navCurrentFor,
    type ScreenKey,
  } from "$lib/navigation/routes";
  import MoreSheet from "./more-sheet.svelte";
  import NavItem from "./nav-item.svelte";
  import { shellMessages } from "./shell.messages";

  type Props = {
    screen: ScreenKey;
    isAdmin: boolean;
    onSignOut: (trigger: HTMLElement | null) => void;
    class?: string;
  };

  let { screen, isAdmin, onSignOut, class: className = "" }: Props = $props();

  const current = $derived(navCurrentFor(screen));
  let moreOpen = $state(false);
  let moreButton = $state<HTMLElement | null>(null);
</script>

<nav
  aria-label={t(shellMessages.bottomNav)}
  class="glass-shell elevation-2 fixed inset-x-0 bottom-0 z-[42] rounded-t-panel px-2 pt-1.5 pb-[calc(0.375rem+var(--safe-area-bottom))] {className}"
>
  <ul class="grid w-full grid-cols-5 gap-1">
    {#each MOBILE_PRIMARY as item (item.key)}
      {#if item.kind === "screen"}
        <li>
          <NavItem
            variant="bar"
            href={item.href}
            label={t(item.label)}
            icon={item.icon}
            current={item.key === current}
          />
        </li>
      {/if}
    {/each}
    <li>
      <NavItem
        bind:ref={moreButton}
        variant="bar"
        label={t(MORE_LABEL)}
        icon={MORE_ICON}
        current={isMoreCurrent(screen)}
        onclick={() => (moreOpen = true)}
      />
    </li>
  </ul>
</nav>
<MoreSheet
  bind:open={moreOpen}
  {screen}
  {isAdmin}
  returnFocusTo={moreButton}
  onSignOut={() => onSignOut(moreButton)}
/>
