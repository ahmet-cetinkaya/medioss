<!--
  @component
  Brand lockup (DESIGN_SYSTEM §5.1): logo orb + "MediOSS" wordmark. With
  `compact` the wordmark uses a smaller treatment for tight spaces.

  Props: `compact` (default false), `href` (renders a link), `class`.
-->
<script lang="ts">
  import { brandMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import { cn } from "$lib/utils.js";
  import Orb from "./orb.svelte";

  type Props = { compact?: boolean; href?: string; class?: string };

  let { compact = false, href, class: className }: Props = $props();

  const classes = $derived(
    cn("inline-flex min-w-0 items-center gap-4 rounded-full", className),
  );
</script>

{#snippet body()}
  <Orb size="logo" />
  <span
    class={compact
      ? "text-base text-ink-strong whitespace-nowrap"
      : "text-brand text-ink-strong whitespace-nowrap overflow-hidden text-clip"}
  >
    {t(brandMessages.productName)}
  </span>
{/snippet}

{#if href}
  <a {href} class={cn("focus-ring min-h-11 px-0.5", classes)}
    >{@render body()}</a
  >
{:else}
  <span class={cn("min-h-11", classes)}>{@render body()}</span>
{/if}
