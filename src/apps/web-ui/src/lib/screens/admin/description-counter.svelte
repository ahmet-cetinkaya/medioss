<!-- @component Character counter for a meditation/program description field. -->
<script lang="ts">
  import { t } from "$lib/i18n/locale.svelte";
  import { meditationAdminMessages as m } from "./admin.messages";
  import { DESCRIPTION_MAX, descriptionLength } from "./meditations-logic";

  let {
    counterId,
    text,
    over,
  }: { counterId: string; text: string; over: boolean } = $props();
</script>

<p
  id={counterId}
  class={over ? "text-sm text-destructive" : "text-sm text-ink-muted"}
>
  {t(m.descriptionCount, {
    count: descriptionLength(text.trim()),
    max: DESCRIPTION_MAX,
  })}
  {#if over}<span role="alert"
      >{t(m.descriptionTooLong, { max: DESCRIPTION_MAX })}</span
    >{/if}
</p>
