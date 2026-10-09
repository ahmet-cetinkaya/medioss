<!--
  @component
  Icon-only button (DESIGN_SYSTEM §5.3): 48 px solid circle, flat,
  22 px icon. `label` is required and becomes the accessible name (FR-097);
  the icon is decorative.

  Props: `label` (translated), `icon` (IconName), `filled` (filled glyph for
  active states), `href` (renders `<a>`), `pressed` (sets `aria-pressed`;
  buttons only), `busy` (see Button), `bind:ref`, `class`, and every native
  button or anchor attribute (`onclick`, `disabled`, `type`, ...).
-->
<script lang="ts">
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import type { IconName } from "$lib/components/icons/icon-name";
  import { Button, type ButtonProps } from "$lib/components/ui/button/index.js";

  type Props = Omit<ButtonProps, "variant" | "size" | "children"> & {
    label: string;
    icon: IconName;
    filled?: boolean;
    pressed?: boolean;
  };

  let {
    label,
    icon,
    filled = false,
    pressed,
    busy = false,
    href,
    ref = $bindable(null),
    ...restProps
  }: Props = $props();
</script>

<Button
  bind:ref
  variant="icon"
  {href}
  {busy}
  aria-label={label}
  aria-pressed={href ? undefined : pressed}
  {...restProps}
>
  {#if !busy}
    <AppIcon name={icon} {filled} />
  {/if}
</Button>
