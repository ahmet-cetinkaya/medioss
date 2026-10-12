<!--
  @component
  The glossy brand orb (DESIGN_SYSTEM §2.4). Decorative (`aria-hidden`).
  Layered gradients plus inset shadows make it read as glass. It uses no
  backdrop filter, so it looks the same under reduced transparency.

  Props: `size` ("logo" 40 px | "hero" 68 px | "inline" 16 px, default
  "logo"), `variant` ("brand" | "user"; user is a deeper, cooler purple), `class`.
-->
<script lang="ts">
  import { cn } from "$lib/utils.js";

  type Props = {
    size?: "logo" | "hero" | "inline";
    variant?: "brand" | "user";
    class?: string;
  };

  let { size = "logo", variant = "brand", class: className }: Props = $props();

  const SIZES = {
    logo: "size-10 shadow-[0_8px_18px_-4px_rgb(126_83_173/0.45),inset_0_-3px_6px_rgb(110_58_120/0.35),inset_0_2px_3px_rgb(255_255_255/0.6),inset_0_0_0_1px_rgb(255_255_255/0.45)]",
    hero: "size-17 shadow-[0_16px_32px_-8px_rgb(126_83_173/0.5),inset_0_-5px_10px_rgb(110_58_120/0.35),inset_0_3px_5px_rgb(255_255_255/0.6),inset_0_0_0_1px_rgb(255_255_255/0.45)]",
    inline:
      "size-4 shadow-[0_3px_6px_-1px_rgb(126_83_173/0.4),inset_0_-1px_2px_rgb(110_58_120/0.35),inset_0_1px_1px_rgb(255_255_255/0.6),inset_0_0_0_1px_rgb(255_255_255/0.4)]",
  } as const;

  const FILLS = {
    brand: "bg-(image:--gradient-orb)",
    user: "bg-(image:--gradient-orb-user)",
  } as const;
</script>

<span
  aria-hidden="true"
  class={cn(
    "inline-block shrink-0 rounded-full forced-colors:border forced-colors:border-current",
    SIZES[size],
    FILLS[variant],
    className,
  )}
></span>
