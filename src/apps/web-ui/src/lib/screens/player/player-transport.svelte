<!--
  @component
  The transport controls: seek slider with elapsed and total time, play/pause and ±10 s on the first row; restart, the speed button and the
  icon-only Soundscape button on the second.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { IconButton } from "$lib/components/ui/icon-button/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import {
    formatClock,
    formatClockSpoken,
  } from "$lib/i18n/formatting/clock-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { SKIP_SECONDS } from "./player-controls-logic";
  import { playerMessages as m } from "./player.messages";

  type Props = {
    position: number;
    duration: number;
    playing: boolean;
    playbackError: boolean;
    speedText: string;
    skipLabel: string;
    /** True while the background soundscape is actually playing. */
    mixPlaying: boolean;
    onSeek: (seconds: number) => void;
    onRestart: () => void;
    onSkip: (delta: number) => void;
    onTogglePlay: () => void;
    onCycleSpeed: () => void;
    onOpenMix: () => void;
  };

  let {
    position = $bindable(),
    duration,
    playing,
    playbackError,
    speedText,
    skipLabel,
    mixPlaying,
    onSeek,
    onRestart,
    onSkip,
    onTogglePlay,
    onCycleSpeed,
    onOpenMix,
  }: Props = $props();
</script>

{#snippet skipHint()}
  <span
    aria-hidden="true"
    class="pointer-events-none absolute inset-0 flex items-center justify-center pt-px text-[0.625rem] font-bold text-ink-strong"
  >
    {skipLabel}
  </span>
{/snippet}

<div class="flex flex-col gap-1">
  <Slider
    bind:value={position}
    min={0}
    max={duration}
    step={1}
    label={t(m.seek)}
    valueText={t(m.seekValue, {
      elapsed: formatClockSpoken(position, locale.current),
      total: formatClockSpoken(duration, locale.current),
    })}
    disabled={playbackError}
    oninput={(event) => onSeek(event.currentTarget.valueAsNumber)}
  />
  <div
    class="flex justify-between text-body-md font-medium text-ink-strong tabular-nums"
  >
    <span>
      <span class="sr-only">{t(m.elapsed)}</span>
      {formatClock(position)}
    </span>
    <span>
      <span class="sr-only">{t(m.total)}</span>
      {formatClock(duration)}
    </span>
  </div>
</div>

<div class="flex flex-col items-center gap-4">
  <div class="flex items-center justify-center gap-4">
    <div class="relative">
      <IconButton
        label={t(m.skipBack)}
        icon="arrow-counter-clockwise"
        disabled={playbackError}
        onclick={() => onSkip(-SKIP_SECONDS)}
      />
      {@render skipHint()}
    </div>
    <IconButton
      label={t(playing ? actionMessages.pause : actionMessages.play)}
      icon={playing ? "pause" : "play"}
      filled
      disabled={playbackError}
      class="size-16 [&_svg]:size-7"
      onclick={onTogglePlay}
    />
    <div class="relative">
      <IconButton
        label={t(m.skipForward)}
        icon="arrow-clockwise"
        disabled={playbackError}
        onclick={() => onSkip(SKIP_SECONDS)}
      />
      {@render skipHint()}
    </div>
  </div>
  <div class="flex items-center justify-center gap-4">
    <IconButton
      label={t(m.restart)}
      icon="skip-back"
      disabled={playbackError}
      onclick={onRestart}
    />
    <Button
      variant="secondary"
      aria-label={t(m.speedButton, { value: speedText })}
      onclick={onCycleSpeed}
    >
      {speedText}
    </Button>
    <!-- The button opens the mixer dialog, so it is not a toggle (no
         aria-pressed). While a soundscape plays it shows the active look and
         the accessible name says so; the icon is decorative. -->
    <IconButton
      label={mixPlaying
        ? `${t(m.mixToggle)} ${t(m.mixPlaying)}`
        : t(m.mixToggle)}
      icon="speaker-high"
      class={mixPlaying
        ? "border-accent-fill bg-accent-soft text-accent-strong hover:bg-accent-soft"
        : undefined}
      onclick={onOpenMix}
    />
  </div>
</div>
