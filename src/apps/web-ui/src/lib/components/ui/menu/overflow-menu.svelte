<!--
  Overflow ("three-dot") menu: wraps secondary actions behind one icon button.
  Built on bits-ui DropdownMenu (already a devDependency used by vendored
  shadcn-svelte primitives). Keyboard + focus behavior come from bits-ui:
  Enter/Space/ArrowDown opens, arrows move, Escape closes and returns focus.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { DropdownMenu } from "bits-ui";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { menuMessages } from "./overflow-menu.messages.ts";

  let {
    label,
    children,
    align = "end",
  }: {
    /** Accessible name for the trigger, e.g. "More actions for Body Scan". */
    label: string;
    /** Menu item snippet. Use <OverflowItem> inside. */
    children: Snippet;
    align?: "start" | "end";
  } = $props();

  let open = $state(false);
  const id = `overflow-${Math.random().toString(36).slice(2, 9)}`;
</script>

<DropdownMenu.Root bind:open>
  <DropdownMenu.Trigger
    {id}
    aria-label={label}
    class="focus-ring elevation-1 inline-flex min-h-11 min-w-11 items-center justify-center rounded-full border border-rim bg-surface-solid text-ink-body hover:bg-panel-tint hover:text-ink-strong motion-safe:transition-colors motion-safe:duration-(--duration-fast)"
  >
    <AppIcon name="dots-three" size={20} />
  </DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <!-- Marker keeps the menu visible under the player focus-mode CSS. -->
    <div data-dialog-portal="" class="contents">
      <DropdownMenu.Content
        sideOffset={6}
        {align}
        class="elevation-2 z-50 min-w-44 rounded-panel border border-rim bg-surface-solid p-1 focus-ring"
      >
        {@render children()}
      </DropdownMenu.Content>
    </div>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
