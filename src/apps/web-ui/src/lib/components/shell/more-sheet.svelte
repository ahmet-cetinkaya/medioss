<!--
  @component
  The mobile "More" sheet (FR-087): History, Statistics, Settings, Admin
  (admins only, FR-003), and Sign out, in a `<nav>` inside the shared Sheet.

  Sign out hands off to the confirm dialog (plan D6): the sheet closes first,
  then, once it has finished closing, `onSignOut()` runs; the sheet skips its
  own focus return so the confirm can return focus to the More button.

  Props: `open` (bindable), `screen`, `isAdmin`, `onSignOut()`,
  `returnFocusTo?` (the More button).
-->
<script lang="ts">
  import Sheet from "$lib/components/dialogs/sheet.svelte";
  import { t } from "$lib/i18n/locale.svelte";
  import {
    MORE_ITEMS,
    MORE_LABEL,
    navCurrentFor,
    type ScreenKey,
  } from "$lib/navigation/routes";
  import { DONATE_URL } from "$lib/navigation/support";
  import NavItem from "./nav-item.svelte";
  import { shellMessages } from "./shell.messages";

  type Props = {
    open?: boolean;
    screen: ScreenKey;
    isAdmin: boolean;
    onSignOut: () => void;
    returnFocusTo?: HTMLElement | null;
  };

  let {
    open = $bindable(false),
    screen,
    isAdmin,
    onSignOut,
    returnFocusTo,
  }: Props = $props();

  const current = $derived(navCurrentFor(screen));
  let signingOut = false;
</script>

<Sheet
  bind:open
  title={t(MORE_LABEL)}
  {returnFocusTo}
  onCloseAutoFocus={(event) => {
    if (!signingOut) return;
    signingOut = false;
    event.preventDefault();
    onSignOut();
  }}
>
  <nav aria-label={t(shellMessages.moreNav)}>
    <ul class="flex flex-col gap-1">
      {#each MORE_ITEMS.filter((item) => isAdmin || !item.requiresAdmin) as item (item.key)}
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
              onclick={() => {
                signingOut = true;
                open = false;
              }}
            />
          {/if}
        </li>
      {/each}
      <li>
        <NavItem
          variant="sidebar"
          href={DONATE_URL}
          external
          label={t(shellMessages.support)}
          icon="coffee"
        />
      </li>
    </ul>
  </nav>
</Sheet>
