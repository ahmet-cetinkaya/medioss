<!--
  @component
  Soft Glass button (DESIGN_SYSTEM §5.13). Renders `<a>` when `href` is set,
  otherwise `<button type="button">`.

  Variants (`variant`):
  - `primary`: accent fill, white label, pill, flat (no elevation). Renders
    `data-variant="primary"`. At most one per panel or dialog (FR-101).
  - `secondary`: solid white pill, strong text, rim, flat (no elevation).
  - `ghost`: transparent, body text; hover is solid white at 50 %.
  - `destructive`: muted red fill (`--destructive`), white label (>= 4.5:1).
  - `icon`: 48 px solid circle; always sets `size="icon"`. Prefer IconButton,
    which requires an accessible name.
  - Legacy aliases: `default` = primary, `outline` = secondary, `link` =
    underlined accent text. `data-variant` reports the resolved name.

  Sizes (`size`), all with a hit area of at least 44 x 44 px (FR-095):
  `default` (44 px), `sm` / `xs` (44 px, tighter padding and smaller text),
  `lg` (48 px), `icon` / `icon-lg` (48 px circle), `icon-sm` / `icon-xs`
  (44 px circle).

  `busy`: sets `aria-busy="true"`, disables the control (no second activation,
  FR-020) and shows a ring before the label that spins only when motion is
  allowed (FR-093). Every variant uses the `focus-ring` recipe.
-->
<script lang="ts" module>
  import { type VariantProps, tv } from "tailwind-variants";
  import { cn, type WithElementRef } from "$lib/utils.js";
  import type {
    HTMLAnchorAttributes,
    HTMLButtonAttributes,
  } from "svelte/elements";

  export const buttonVariants = tv({
    base: "focus-ring group/button relative inline-flex min-h-11 min-w-11 max-w-full shrink-0 cursor-pointer wrap-anywhere items-center justify-center rounded-full border border-transparent text-sm leading-tight font-medium text-center select-none motion-safe:transition-colors motion-safe:duration-(--duration-fast) disabled:pointer-events-none disabled:opacity-60 aria-disabled:pointer-events-none aria-disabled:opacity-60 aria-busy:cursor-progress aria-invalid:border-destructive [&_svg]:pointer-events-none [&_svg]:shrink-0",
    variants: {
      variant: {
        primary:
          "bg-accent-fill text-primary-foreground hover:bg-accent-strong",
        secondary:
          "border-rim bg-surface-solid text-ink-strong hover:bg-surface-solid/80",
        ghost:
          "bg-transparent text-ink-body hover:bg-surface-solid/50 hover:text-ink-strong",
        destructive:
          "bg-destructive text-primary-foreground hover:bg-destructive/90",
        icon: "border-rim bg-surface-solid text-ink-strong hover:bg-surface-solid/80",
        // Legacy aliases (pre-redesign islands and vendored shadcn parts).
        default:
          "bg-accent-fill text-primary-foreground hover:bg-accent-strong",
        outline:
          "border-rim bg-surface-solid text-ink-strong hover:bg-surface-solid/80",
        link: "text-accent-strong underline-offset-4 hover:underline",
      },
      size: {
        default: "gap-2 px-5",
        xs: "gap-1.5 px-3 text-xs",
        sm: "gap-1.5 px-4 text-[0.8125rem]",
        lg: "min-h-12 gap-2 px-6 text-base",
        icon: "size-12 p-0",
        "icon-xs": "size-11 p-0",
        "icon-sm": "size-11 p-0",
        "icon-lg": "size-12 p-0",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "default",
    },
  });

  export type ButtonVariant = VariantProps<typeof buttonVariants>["variant"];
  export type ButtonSize = VariantProps<typeof buttonVariants>["size"];

  export type ButtonProps = WithElementRef<HTMLButtonAttributes> &
    WithElementRef<HTMLAnchorAttributes> & {
      variant?: ButtonVariant;
      size?: ButtonSize;
      busy?: boolean;
    };

  const ALIASES: Partial<Record<NonNullable<ButtonVariant>, string>> = {
    default: "primary",
    outline: "secondary",
  };
</script>

<script lang="ts">
  let {
    class: className,
    variant = "primary",
    size = "default",
    busy = false,
    ref = $bindable(null),
    href = undefined,
    type = "button",
    disabled,
    children,
    ...restProps
  }: ButtonProps = $props();

  const resolvedSize = $derived(variant === "icon" ? "icon" : size);
  const classes = $derived(
    cn(buttonVariants({ variant, size: resolvedSize }), className),
  );
  const dataVariant = $derived(ALIASES[variant ?? "primary"] ?? variant);
  const inactive = $derived(Boolean(disabled) || busy);
</script>

{#snippet content()}
  {#if busy}
    <span
      aria-hidden="true"
      class="size-4 shrink-0 rounded-full border-2 border-current border-r-transparent motion-safe:animate-spin"
    ></span>
  {/if}
  {@render children?.()}
{/snippet}

{#if href}
  <a
    bind:this={ref}
    data-slot="button"
    data-variant={dataVariant}
    class={classes}
    href={inactive ? undefined : href}
    aria-disabled={inactive || undefined}
    aria-busy={busy || undefined}
    role={inactive ? "link" : undefined}
    tabindex={inactive ? -1 : undefined}
    {...restProps}
  >
    {@render content()}
  </a>
{:else}
  <button
    bind:this={ref}
    data-slot="button"
    data-variant={dataVariant}
    class={classes}
    {type}
    disabled={inactive}
    aria-busy={busy || undefined}
    {...restProps}
  >
    {@render content()}
  </button>
{/if}
