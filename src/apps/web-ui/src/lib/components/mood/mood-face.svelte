<!--
  @component
  The face of a mood score on its score colour: the same badge in every list
  that shows a mood entry. Decorative; the score is always stated in text.

  Props: `score` (1-5), `size` ("sm" 20 px | "md" 40 px, default "md").
-->
<script lang="ts">
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { MOOD_FACE, type MoodScore } from "./score-selector.svelte";

  type Props = { score: number; size?: "sm" | "md" };

  let { score, size = "md" }: Props = $props();

  const MOOD_BG: Record<MoodScore, string> = {
    1: "bg-mood-1",
    2: "bg-mood-2",
    3: "bg-mood-3",
    4: "bg-mood-4",
    5: "bg-mood-5",
  };
  const BADGE_CLASS = { sm: "size-5", md: "size-10" };
  const ICON_SIZE = { sm: 14, md: 22 };

  const moodScore = $derived(score as MoodScore);
</script>

<span
  data-mood-dot={score}
  aria-hidden="true"
  class={`grid shrink-0 place-items-center rounded-full text-primary-foreground ${BADGE_CLASS[size]} ${MOOD_BG[moodScore]}`}
>
  <AppIcon name={MOOD_FACE[moodScore]} size={ICON_SIZE[size]} filled />
</span>
