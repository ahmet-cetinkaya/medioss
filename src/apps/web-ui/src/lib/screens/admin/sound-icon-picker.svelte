<!--
  @component
  Icon picker for a sound (T28): a radio grid of the bundled glyphs (plus "no
  icon") and an "Upload" tile (its glyph becomes the chosen image), an image upload (PNG, JPEG or WebP, up to 1 MB, checked
  here before the server checks it again), and a remove-image action.

  Props: `bind:key` (glyph key or ""), `bind:file` (chosen image or null),
  `bind:clearImage` (true to delete the stored image on save), `hasImage`
  (the sound already has an uploaded image), `idPrefix` (unique ids).
-->
<script lang="ts">
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import {
    SOUND_ICON_KEYS,
    type SoundIconKey,
  } from "$lib/components/icons/icon-name";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { soundIconUrl } from "$lib/api/soundscape";
  import { t } from "$lib/i18n/translate";
  import { soundIconMessages as m } from "./admin.messages";

  const OPTIONS: ("" | SoundIconKey)[] = ["", ...SOUND_ICON_KEYS];
  const MAX_BYTES = 1024 * 1024;
  const TYPES = ["image/png", "image/jpeg", "image/webp"];

  type Props = {
    key?: string;
    file?: File | null;
    clearImage?: boolean;
    hasImage?: boolean;
    idPrefix: string;
    soundId?: number;
  };

  let {
    key = $bindable(""),
    file = $bindable(null),
    clearImage = $bindable(false),
    hasImage = false,
    idPrefix,
    soundId,
  }: Props = $props();

  let input = $state<HTMLInputElement | null>(null);
  let problem = $state<"badType" | "tooLarge" | null>(null);

  let previewUrl = $state<string | null>(null);
  $effect(() => {
    if (!file) {
      previewUrl = null;
      return;
    }
    const url = URL.createObjectURL(file);
    previewUrl = url;
    return () => URL.revokeObjectURL(url);
  });
  const existingUrl = $derived(soundId ? soundIconUrl(soundId) : null);

  const showsImage = $derived(!!file || (hasImage && !clearImage));

  function choose(event: Event & { currentTarget: HTMLInputElement }) {
    const picked = event.currentTarget.files?.[0] ?? null;
    problem = null;
    if (picked && !TYPES.includes(picked.type)) problem = "badType";
    else if (picked && picked.size > MAX_BYTES) problem = "tooLarge";
    if (problem) {
      event.currentTarget.value = "";
      return;
    }
    file = picked;
    if (picked) clearImage = false;
  }

  function removeImage() {
    file = null;
    problem = null;
    if (input) input.value = "";
    clearImage = hasImage;
  }
</script>

<fieldset class="m-0 grid min-w-0 gap-3 border-0 p-0">
  <legend class="mb-2 text-sm font-medium text-ink-body">{t(m.legend)}</legend>
  <div class="flex flex-wrap gap-2">
    {#each OPTIONS as option (option)}
      <label
        class="inline-flex min-h-11 min-w-11 cursor-pointer items-center justify-center gap-1.5 rounded-tile border border-rim bg-surface-solid/70 px-3 text-sm text-ink-body has-[:checked]:border-ink-strong has-[:checked]:font-semibold has-[:checked]:text-ink-strong has-[:focus-visible]:outline-2"
        title={t(option ? m[option] : m.none)}
      >
        <input
          type="radio"
          name="{idPrefix}-icon"
          value={option}
          bind:group={key}
          class="sr-only"
        />
        {#if option}
          <AppIcon name={option} size={22} />
          <span class="sr-only">{t(m[option])}</span>
        {:else}
          <AppIcon name="x" size={16} />
          {t(m.none)}
        {/if}
      </label>
    {/each}
    <button
      type="button"
      class="inline-flex min-h-11 min-w-11 cursor-pointer items-center justify-center gap-1.5 rounded-tile border border-rim bg-surface-solid/70 px-3 text-sm text-ink-body focus-visible:outline-2"
      onclick={() => input?.click()}
    >
      {#if file && previewUrl}
        <img
          src={previewUrl}
          alt=""
          class="size-[22px] rounded-sm object-cover"
        />
      {:else if hasImage && !clearImage && existingUrl}
        <img
          src={existingUrl}
          alt=""
          class="size-[22px] rounded-sm object-cover"
        />
      {:else}
        <AppIcon name="upload-simple" size={22} />
      {/if}
      {t(m.uploadTile)}
    </button>
  </div>
  <div class="grid gap-2">
    <Input
      id="{idPrefix}-icon-file"
      type="file"
      bind:ref={input}
      accept="image/png,image/jpeg,image/webp"
      onchange={choose}
      class="sr-only"
      tabindex={-1}
      aria-label={t(m.uploadTile)}
    />
    <p class="text-sm text-ink-muted">{t(m.imageHint)}</p>
    {#if problem}
      <InlineMessage tone="error" role="alert">{t(m[problem])}</InlineMessage>
    {/if}
    {#if showsImage}
      <div class="flex flex-wrap items-center gap-3">
        {#if !file}
          <span class="text-sm text-ink-body">{t(m.currentImage)}</span>
        {/if}
        <Button variant="secondary" size="sm" onclick={removeImage}>
          <AppIcon name="trash" size={16} />
          {t(m.clearImage)}
        </Button>
      </div>
    {/if}
  </div>
</fieldset>
