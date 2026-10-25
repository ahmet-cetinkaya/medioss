<!--
  @component
  Labeled sidebar for >= 1280 px (DESIGN_SYSTEM §5.2, FR-002). Shown with
  `class="hidden xl:flex"` by AppShell; `display: none` removes it from the
  accessibility tree at other widths. The brand lockup sits on top; groups
  follow FR-002 order with overline labels; Sign out is the last item.

  Props: `screen` (current screen key), `isAdmin` (shows Admin, FR-003),
  `onSignOut(trigger)`, `class`.
-->
<script lang="ts">
  import BrandLockup from "$lib/components/content/brand-lockup.svelte";
  import { t } from "$lib/i18n/locale.svelte";
  import {
    NAV_GROUPS,
    navCurrentFor,
    type ScreenKey,
  } from "$lib/navigation/routes";
  import SupportDialog from "$lib/components/dialogs/support-dialog.svelte";
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
  const groupId = $props.id();
  let supportOpen = $state(false);
</script>

<nav
  aria-label={t(shellMessages.sidebarNav)}
  class="w-64 shrink-0 flex-col gap-6 px-4 py-8 {className}"
>
  <BrandLockup href="/" class="mb-2 ms-4" />
  {#each NAV_GROUPS as group (group.key)}
    <div role="group" aria-labelledby="{groupId}-{group.key}">
      <p
        id="{groupId}-{group.key}"
        class="text-overline mb-4 ms-6 text-ink-muted uppercase"
      >
        {t(group.label)}
      </p>
      <ul class="flex flex-col gap-1">
        {#each group.items.filter((item) => isAdmin || !item.requiresAdmin) as item (item.key)}
          <li>
            {#if item.kind === "screen"}
              <NavItem
                variant="sidebar"
                href={item.href}
                label={t(item.label)}
                icon={item.icon}
                current={item.key === current}
              />
            {:else}
              <NavItem
                variant="sidebar"
                label={t(item.label)}
                icon={item.icon}
                onclick={(event) => onSignOut(event.currentTarget)}
              />
            {/if}
          </li>
        {/each}
      </ul>
    </div>
  {/each}
  <NavItem
    variant="sidebar"
    label={t(shellMessages.support)}
    icon="coffee"
    onclick={() => (supportOpen = true)}
  />
  <SupportDialog bind:open={supportOpen} />
</nav>
