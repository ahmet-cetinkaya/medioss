<!--
  @component
  One navigation entry (DESIGN_SYSTEM §5.2, FR-004, FR-086). A link when
  `href` is set, otherwise a button (Sign out, More).

  The current item gets `aria-current="page"`, a solid pill, weight 600, and
  the filled icon in `accent-strong`, so "current" is never color alone. A
  button may be current too (More, when the current screen lives in it).

  Variants: `sidebar` (56 px labeled row; also the More sheet), `rail` (48 px
  circle; the label stays in the DOM as the accessible name and shows as a
  tooltip on hover and keyboard focus), `bar` (bottom bar: icon over a small
  label).

  Props: `label` (translated), `icon`, `variant`, `current`, `href?`,
  `onclick?`, `bind:ref`.
-->
<script lang="ts">
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import type { IconName } from "$lib/components/icons/icon-name";

  type Props = {
    label: string;
    icon: IconName;
    variant: "sidebar" | "rail" | "bar";
    current?: boolean;
    href?: string;
    onclick?: (event: MouseEvent & { currentTarget: HTMLElement }) => void;
    ref?: HTMLElement | null;
  };

  let {
    label,
    icon,
    variant,
    current = false,
    href,
    onclick,
    ref = $bindable(null),
  }: Props = $props();

  const LAYOUT = {
    sidebar: "flex h-14 w-full items-center gap-3 px-6 text-base text-start",
    rail: "group relative grid size-12 place-items-center",
    bar: "flex min-h-14 w-full flex-col items-center justify-center gap-0.5 px-0.5 py-1 text-center text-[0.6875rem] leading-tight",
  } as const;

  const classes = $derived(
    [
      "focus-ring cursor-pointer rounded-full motion-safe:transition-colors motion-safe:duration-(--duration-fast)",
      LAYOUT[variant],
      current
        ? "bg-surface-solid font-semibold text-ink-strong"
        : "font-normal text-ink-body hover:bg-surface-solid/50 hover:text-ink-strong",
    ].join(" "),
  );
</script>

{#snippet body()}
  <AppIcon
    name={icon}
    filled={current}
    class={current ? "shrink-0 text-accent-strong" : "shrink-0"}
  />
  {#if variant === "rail"}
    <span
      class="glass-solid elevation-1 pointer-events-none absolute start-full top-1/2 z-40 ms-3 -translate-y-1/2 rounded-full px-3 py-1.5 text-sm whitespace-nowrap text-ink-strong opacity-0 group-hover:opacity-100 group-focus-visible:opacity-100 motion-safe:transition-opacity motion-safe:duration-(--duration-fast)"
    >
      {label}
    </span>
  {:else}
    <!-- The bar's labels are single short words: keep them whole (shrink, never
         split mid-word); the other variants may wrap at spaces only. -->
    <span
      class={variant === "bar"
        ? "min-w-0 text-[0.625rem] tracking-tight"
        : "min-w-0 wrap-anywhere"}>{label}</span
    >
  {/if}
{/snippet}

{#if href}
  <a
    bind:this={ref}
    {href}
    class={classes}
    aria-current={current ? "page" : undefined}
  >
    {@render body()}
  </a>
{:else}
  <button
    bind:this={ref}
    type="button"
    class={classes}
    aria-current={current ? "page" : undefined}
    {onclick}
  >
    {@render body()}
  </button>
{/if}
