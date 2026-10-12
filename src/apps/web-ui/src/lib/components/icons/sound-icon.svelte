<!--
  @component
  Decorative sound icon (T28): the uploaded image when the sound has one, else
  the bundled glyph for its `icon_key`, else a glyph for its category. It is
  aria-hidden; the sound's name stays the accessible label.

  Props: `sound`, `categorySlug` (fallback), `size` (px, default 40), `class`.
-->
<script lang="ts">
  import { cn } from "$lib/utils.js";
  import { soundIconUrl, type SoundscapeSound } from "$lib/api/soundscape";
  import AppIcon from "./app-icon.svelte";
  import { isSoundIconKey, type SoundIconKey } from "./icon-name";
  import { categoryIconKey } from "./sound-icon-category";

  type Props = {
    sound: Pick<SoundscapeSound, "id" | "icon_key" | "has_icon">;
    categorySlug?: string;
    size?: number;
    class?: string;
  };

  let {
    sound,
    categorySlug = "",
    size = 40,
    class: className,
  }: Props = $props();

  const glyph = $derived<SoundIconKey>(
    isSoundIconKey(sound.icon_key)
      ? (sound.icon_key as SoundIconKey)
      : categoryIconKey(categorySlug),
  );
</script>

<span
  aria-hidden="true"
  class={cn(
    "inline-flex shrink-0 items-center justify-center overflow-hidden rounded-tile bg-surface-solid text-ink-strong",
    className,
  )}
  style="width: {size}px; height: {size}px"
>
  {#if sound.has_icon}
    <img
      src={soundIconUrl(sound.id)}
      alt=""
      width={size}
      height={size}
      loading="lazy"
      class="size-full object-cover"
    />
  {:else}
    <AppIcon name={glyph} size={22} />
  {/if}
</span>
