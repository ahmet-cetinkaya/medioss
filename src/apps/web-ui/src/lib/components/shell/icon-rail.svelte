<!--
  @component
  Icon rail for 768–1279 px (DESIGN_SYSTEM §4.3, FR-085, FR-086). Shown with
  `class="hidden md:flex xl:hidden"` by AppShell. Each item keeps its label
  in the DOM (the accessible name) and shows it as a tooltip on hover and
  keyboard focus. Groups follow FR-002 order, separated by dividers.

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
</script>

<nav
  aria-label={t(shellMessages.railNav)}
  class="w-22 shrink-0 flex-col items-center gap-4 py-6 {className}"
>
  <BrandLockup compact href="/" />
  {#each NAV_GROUPS as group, index (group.key)}
    <ul
      aria-label={t(group.label)}
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
</nav>
