<!--
  @component
  Language choice (FR-112, FR-113, FR-115): native radios in a fieldset with
  a translated "Language" legend. Each option is named in its own language
  ("English", "Türkçe") and carries a matching `lang`. Choosing one calls
  `locale.set`, which re-renders text in place: nothing remounts, so other
  inputs and open dialogs keep their state. Same look as ChipGroup (44 px
  pills; the selected one is solid, weight 600, with a check icon).

  Props: `legendHidden` (visually hide the legend; it stays the group's
  name), `class`.
-->
<script lang="ts">
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { languageMessages } from "$lib/components/shell/shell.messages";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { LOCALES, type Locale } from "$lib/i18n/message";

  type Props = { legendHidden?: boolean; class?: string };

  let { legendHidden = false, class: className = "" }: Props = $props();

  const name = $props.id();
  const NAMES = {
    en: languageMessages.english,
    tr: languageMessages.turkish,
  } as const;

  function choose(event: Event & { currentTarget: HTMLInputElement }) {
    locale.set(event.currentTarget.value as Locale);
  }
</script>

<fieldset class="m-0 min-w-0 border-0 p-0 {className}">
  <legend
    class={legendHidden ? "sr-only" : "mb-2 text-sm font-medium text-ink-body"}
  >
    {t(languageMessages.legend)}
  </legend>
  <div class="flex flex-wrap gap-2">
    {#each LOCALES as option (option)}
      {@const selected = locale.current === option}
      <label
        lang={option}
        class="relative inline-flex min-h-11 min-w-11 max-w-full cursor-pointer items-center justify-center gap-1.5 rounded-full border border-control-border px-4.5 py-1 text-center text-sm break-words [&_svg]:shrink-0 leading-tight select-none has-focus-visible:outline-2 has-focus-visible:outline-offset-2 has-focus-visible:outline-accent-strong motion-safe:transition-colors motion-safe:duration-(--duration-fast) {selected
          ? 'elevation-1 bg-surface-solid font-semibold text-ink-strong'
          : 'bg-transparent font-normal text-ink-body hover:bg-surface-solid/50'}"
      >
        <input
          type="radio"
          class="sr-only"
          {name}
          value={option}
          checked={selected}
          onchange={choose}
        />
        {#if selected}
          <AppIcon name="check" size={16} />
        {/if}
        {t(NAMES[option])}
      </label>
    {/each}
  </div>
</fieldset>
