<!--
  @component
  The overall volume row shared by every soundscape mix player (the
  Soundscape screen's current mix and the meditation Player's mixer dialog).
  It drives `mix.master`, which scales every sound. The level is not saved
  with a mix and lives only as long as the player instance.
-->
<script lang="ts">
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { t } from "$lib/i18n/translate";
  import type { MixState } from "./mix-state.svelte";
  import { soundscapeMessages as m } from "./soundscape.messages";

  type Props = { mix: MixState };

  let { mix }: Props = $props();
</script>

<div
  class="flex items-center gap-3 rounded-tile border border-rim bg-surface-solid/70 px-3 py-2"
>
  <AppIcon name="speaker-high" size={20} />
  <div class="min-w-0 flex-1">
    <p class="pt-1 text-sm font-medium text-ink-strong">
      {t(m.masterVolume)}
    </p>
    <Slider
      value={mix.master}
      oninput={(event) => mix.setMaster(event.currentTarget.valueAsNumber)}
      label={t(m.masterVolume)}
      valueText={t(m.volumeValue, { value: mix.master })}
    />
  </div>
</div>
