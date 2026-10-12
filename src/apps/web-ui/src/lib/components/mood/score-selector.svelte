<!--
  @component
  Mood score selector (FR-055, FR-108): five native radios (1–5) in a
  fieldset, so arrow keys move the selection and Tab enters and leaves the
  group. Each option shows its number and a short text label in its score's
  mood color (--mood-1..5, red → teal). The selected option fills with that
  color and flips to a white ink, and still adds weight 600 and a filled
  icon, so selection is never color alone (FR-092). Both color roles pass
  >= 4.5:1 (see global-css-contrast.test.ts). 44 px targets; the focus ring
  shows on the option label.

  Props: `bind:value` (1–5; the caller sets the default, e.g. 3), `name`
  (radio group name, unique per page), `legend` (translated), `legendHidden`
  (visually hide the legend; it stays the group's name), `disabled`, `class`, `size` ("default" | "large": large renders big
  round Daylio-style faces for check-in contexts).
  Instance method: `focus()` focuses the selected radio (or the first), for
  actions such as "focus the score selector" (FR-059); use `bind:this`.
-->
<script lang="ts" module>
  export type MoodScore = 1 | 2 | 3 | 4 | 5;
</script>

<script lang="ts">
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { t } from "$lib/i18n/translate";
  import { cn } from "$lib/utils.js";
  import { moodScoreMessages } from "./mood-score.messages";

  type Props = {
    value?: MoodScore;
    name: string;
    legend: string;
    legendHidden?: boolean;
    disabled?: boolean;
    size?: "default" | "large";
    class?: string;
  };

  let {
    value = $bindable(),
    name,
    legend,
    legendHidden = false,
    disabled = false,
    size = "default",
    class: className,
  }: Props = $props();

  const SCORES: readonly MoodScore[] = [1, 2, 3, 4, 5];

  /* Static class names so Tailwind sees every mood utility (never
     interpolated); the label sets the color and the children inherit it. */
  const MOOD_FILL: Record<MoodScore, string> = {
    1: "bg-mood-1",
    2: "bg-mood-2",
    3: "bg-mood-3",
    4: "bg-mood-4",
    5: "bg-mood-5",
  };
  const MOOD_TEXT: Record<MoodScore, string> = {
    1: "text-mood-1",
    2: "text-mood-2",
    3: "text-mood-3",
    4: "text-mood-4",
    5: "text-mood-5",
  };

  let fieldset: HTMLFieldSetElement | undefined = $state();

  export function focus(): void {
    const radio =
      fieldset?.querySelector<HTMLInputElement>("input:checked") ??
      fieldset?.querySelector<HTMLInputElement>("input");
    radio?.focus();
  }
</script>

<fieldset
  bind:this={fieldset}
  {disabled}
  class={cn("m-0 min-w-0 border-0 p-0", className)}
>
  <legend
    class={legendHidden ? "sr-only" : "mb-2 text-sm font-medium text-ink-body"}
  >
    {legend}
  </legend>
  <div
    class={size === "large"
      ? "flex flex-wrap justify-center gap-x-3 gap-y-4"
      : "grid grid-cols-[repeat(auto-fit,minmax(2.75rem,1fr))] gap-2"}
  >
    {#each SCORES as score (score)}
      {@const selected = value === score}
      {#if size === "large"}
        <label
          class="group relative flex w-16 cursor-pointer flex-col items-center gap-1.5 text-center select-none has-disabled:cursor-not-allowed has-disabled:opacity-60"
        >
          <input
            type="radio"
            class="peer sr-only"
            {name}
            value={score}
            bind:group={value}
          />
          <span
            class={cn(
              "grid size-14 place-items-center rounded-full border-2 peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-accent-strong motion-safe:transition-transform motion-safe:duration-(--duration-fast) motion-safe:group-hover:scale-105",
              selected
                ? cn(
                    "border-transparent text-primary-foreground motion-safe:scale-110",
                    MOOD_FILL[score],
                  )
                : cn("border-control-border bg-transparent", MOOD_TEXT[score]),
            )}
          >
            <AppIcon name="smiley" size={30} filled={selected} />
          </span>
          <span
            class={cn(
              "text-xs leading-tight break-words text-ink-body",
              selected && "font-semibold text-ink-strong",
            )}
          >
            {score} · {t(moodScoreMessages[score])}
          </span>
        </label>
      {:else}
        <label
          class={cn(
            "relative flex min-h-11 min-w-11 cursor-pointer flex-col items-center justify-center gap-1 rounded-panel border px-1 py-2 text-center select-none has-disabled:cursor-not-allowed has-disabled:opacity-60 has-focus-visible:outline-2 has-focus-visible:outline-offset-2 has-focus-visible:outline-accent-strong motion-safe:transition-colors motion-safe:duration-(--duration-fast)",
            selected
              ? cn(
                  "border-transparent font-semibold text-primary-foreground",
                  MOOD_FILL[score],
                )
              : cn(
                  "border-control-border bg-transparent font-normal hover:bg-surface-solid/50",
                  MOOD_TEXT[score],
                ),
          )}
        >
          <input
            type="radio"
            class="sr-only"
            {name}
            value={score}
            bind:group={value}
          />
          <AppIcon name="smiley" size={20} filled={selected} />
          <span class="text-lg leading-none">{score}</span>
          <span class="text-xs leading-tight break-words">
            {t(moodScoreMessages[score])}
          </span>
        </label>
      {/if}
    {/each}
  </div>
</fieldset>
