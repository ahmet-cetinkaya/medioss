<!--
  @component
  The playback panel (FR-043–FR-045, FR-048, PB-1–PB-5, PB-16): a compact
  full-width cover linking to the program with the meditation name (the
  in-content h1) as a label at its bottom-left, then custom controls over a hidden but present `<audio>`: restart,
  play/pause, ±10 s, one speed button cycling 1×, 1.25×, 1.5× (not
  remembered), seek with elapsed and total time (spoken `aria-valuetext`),
  the Soundscape toggle opening the mixer dialog, the background mix
  section, and download. The background mix follows play and pause.

  - Resume (PB-1): on the first `loadedmetadata`, the saved position unless
    the meditation is completed.
  - Position (PB-2): `position-reporter.ts`; hide reports go through
    `sendBeacon` on `window` `pagehide` and `document` `visibilitychange`.
  - Completion (PB-3): `completion-tracker.ts` on `ended`; once recorded,
    `onCompleted` opens the completion dialog.
  - Media Session (PB-5): title, play, pause, ±10 s, seekto, stop.
  - Audio `error` (plan D2): re-probe `/me`; a 401 goes to Login, otherwise
    a playback error shows while completion, download, and notes still work.

  Focus overlay (owner feedback round 2): while PLAYING, this same panel —
  one DOM tree, the very controls below, never a copy — floats above an
  opaque color-wash backdrop that hides the page UI completely, centered
  and narrow on desktop, with a static circular cover. Pausing or ending returns
  the page. It is not a dialog: no focus trap, Escape moves through the
  page normally, and the page scroll is locked while it is open.
  Implementation notes: the app shell paints a backdrop-filter, which makes
  it the containing block of `position: fixed` descendants — so the stage
  div (backdrop + floating panel wrapper) is moved under <body> while
  playing and back into its inline place on pause. Svelte 5 delegated
  events keep working across the move (verified on this version).
-->
<script lang="ts">
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import { downloadFile } from "$lib/download-file";
  import { onMount } from "svelte";
  import { ApiError } from "$lib/api/api-error";
  import { fetchMeProfile } from "$lib/api/identity";
  import {
    meditationAudioUrl,
    meditationDownloadUrl,
    postCompletion,
    postPosition,
    type Meditation,
    type PlaybackState,
    type ProgramSummary,
  } from "$lib/api/meditation";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { formatClock } from "$lib/i18n/formatting/clock-format";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import type { StringMessage } from "$lib/i18n/message";
  import { redirectToLogin } from "$lib/navigation/login-redirect";
  import { createCompletionTracker } from "./completion-tracker";
  import {
    clampSeek,
    isUsableDuration,
    nextSpeedIndex,
    SKIP_SECONDS,
    SPEEDS,
  } from "./player-controls-logic";
  import PlayerCover from "./player-cover.svelte";
  import PlayerFocusBackdrop from "./player-focus-backdrop.svelte";
  import { enterFocusOverlay, watchIdle } from "./player-focus-overlay";
  import { bindMediaSession, syncSessionState } from "./player-media-session";
  import type { PlayerMix } from "./player-mix.svelte";
  import PlayerMixDialog from "./player-mix-dialog.svelte";
  import { bindPageVisibility } from "./player-page-visibility";
  import PlayerTransport from "./player-transport.svelte";
  import { createPositionReporter } from "./position-reporter";
  import { playerMessages as m } from "./player.messages";

  type Props = {
    meditation: Meditation;
    playback: PlaybackState;
    /** undefined while the program is looked up; null for none. */
    program: ProgramSummary | null | undefined;
    mix: PlayerMix;
    /** The completion was recorded (end of the meditation). */
    onCompleted: () => void;
  };

  let { meditation, playback, program, mix, onCompleted }: Props = $props();

  const IDLE_MS = 5000;

  let audio = $state<HTMLAudioElement | null>(null);
  let playing = $state(false);
  let position = $state(0);
  let reportedDuration = $state<number | null>(null);
  let speedIndex = $state(0);
  let coverFailed = $state(false);
  let playbackError = $state(false);
  let notice = $state<StringMessage | null>(null);
  let resumed = false;
  let mixOpen = $state(false);

  // Focus overlay: the stage div (backdrop + panel wrapper) that moves
  // under <body> while playing and back on pause.
  let stage = $state<HTMLElement | null>(null);

  const duration = $derived(reportedDuration ?? meditation.duration_ms / 1000);
  const showCover = $derived(!!program?.has_cover && !coverFailed);
  const speed = $derived(SPEEDS[speedIndex]);
  const speedText = $derived(
    t(m.speedValue, { value: formatNumber(speed, locale.current) }),
  );
  const skipLabel = $derived(formatNumber(SKIP_SECONDS, locale.current));

  const reporter = createPositionReporter({
    mediaTimeMs: () => (audio?.currentTime ?? 0) * 1000,
    send: (positionMs) =>
      void postPosition(meditation.id, positionMs).then(
        () => {
          if (notice === m.positionFailed) notice = null;
        },
        () => (notice = m.positionFailed),
      ),
    beacon: (positionMs) =>
      navigator.sendBeacon(
        `/api/meditation/${meditation.id}/playback-state`,
        new Blob([JSON.stringify({ position_ms: positionMs })], {
          type: "application/json",
        }),
      ),
  });

  const tracker = createCompletionTracker({
    now: () => Date.now(),
    send: (startedAt, finishedAt) =>
      void postCompletion(meditation.id, startedAt, finishedAt).then(
        onCompleted,
        () => (notice = m.completionFailed),
      ),
  });

  function seekTo(seconds: number) {
    if (!audio) return;
    audio.currentTime = clampSeek(seconds, duration);
    position = audio.currentTime;
  }

  const skip = (delta: number) =>
    seekTo((audio?.currentTime ?? position) + delta);

  const restart = () => seekTo(0);

  function togglePlay() {
    if (!audio) return;
    if (audio.paused) void audio.play().catch(() => {});
    else audio.pause();
  }

  function onLoadedMetadata() {
    if (!audio) return;
    onDurationChange();
    if (!resumed) {
      resumed = true;
      if (!playback.completed && playback.position_ms > 0)
        audio.currentTime = playback.position_ms / 1000;
      reporter.reset(audio.currentTime * 1000);
    }
    position = audio.currentTime;
  }

  function onDurationChange() {
    if (audio && isUsableDuration(audio.duration))
      reportedDuration = audio.duration;
  }

  function onPlay() {
    playing = true;
    reporter.play();
    tracker.play();
    syncSessionState(audio);
    void mix.start(() => playing);
  }

  function onPause() {
    playing = false;
    reporter.pause();
    syncSessionState(audio);
    mix.pause();
  }

  function onEnded() {
    playing = false;
    mix.pause();
    tracker.ended(duration * 1000);
  }

  function onTimeUpdate() {
    if (!audio) return;
    position = audio.currentTime;
    reporter.progress();
  }

  async function onAudioError() {
    if (playbackError) return;
    try {
      await fetchMeProfile();
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) {
        redirectToLogin();
        return;
      }
    }
    playing = false;
    playbackError = true;
  }

  // Playback speed is per visit only (PB-4).
  $effect(() => {
    if (!audio) return;
    audio.defaultPlaybackRate = speed;
    audio.playbackRate = speed;
  });

  // Focus overlay: while playing, the panel floats (CSS: .overlay-floating
  // and .focus-overlay-backdrop are position: fixed) and the page behind
  // cannot scroll and is fully hidden behind the opaque color wash. The
  // controls are never duplicated, so single-match locators keep working.
  $effect(() => {
    if (!playing || !stage) return;
    return enterFocusOverlay(stage);
  });

  // Idle auto-hide: after IDLE_MS without input while playing, the panel
  // fades out and only the big elapsed clock remains. Any input restores it.
  let idle = $state(false);
  $effect(() => {
    if (!playing) {
      idle = false;
      return;
    }
    return watchIdle(IDLE_MS, (value) => (idle = value));
  });

  // Media Session (PB-5).
  $effect(() => {
    const element = audio;
    if (!element) return;
    return bindMediaSession(meditation.name, SKIP_SECONDS, {
      play: () => void element.play().catch(() => {}),
      pause: () => element.pause(),
      skip,
      seekTo,
      stop: () => {
        element.pause();
        seekTo(0);
        syncSessionState(element);
      },
    });
  });

  onMount(() => {
    return bindPageVisibility({
      hide: () => reporter.hide(),
      show: () => reporter.show(),
    });
  });
</script>

<!-- The overlay stage: while playing the $effect above moves this whole
     div (backdrop + floating panel) under <body>, so `position: fixed`
     escapes the glass shell's backdrop-filter containing block. Paused, the
     panel renders inline here and the backdrop is unmounted. The same
     $effect toggles .player-focus-on on <body>, hiding the rest of the
     page UI behind the opaque wash. -->
<div bind:this={stage} data-player-floating="" class="min-w-0">
  {#if playing}
    <PlayerFocusBackdrop
      {program}
      {showCover}
      onCoverError={() => (coverFailed = true)}
    />
  {/if}
  {#if playing && idle}
    <div
      class="pointer-events-none fixed inset-0 z-50 flex items-center justify-center"
      data-player-idle-clock=""
    >
      <span class="sr-only">{t(m.elapsed)}</span>
      <span class="text-display text-ink-strong tabular-nums"
        >{formatClock(position)}</span
      >
    </div>
  {/if}
  <Panel
    aria-labelledby="player-controls-heading"
    data-elevation="featured"
    class="elevation-3 flex flex-col gap-6 {playing
      ? 'overlay-floating'
      : ''} {playing && idle
      ? 'pointer-events-none opacity-0'
      : ''} motion-safe:transition-opacity motion-safe:duration-700"
  >
    <!-- sr-only (absolute), so the flex gap ignores it -->
    <h2 id="player-controls-heading" class="sr-only">
      {t(m.controlsHeading)}
    </h2>

    <PlayerCover
      {meditation}
      {program}
      {playing}
      {showCover}
      onCoverError={() => (coverFailed = true)}
    />

    <audio
      bind:this={audio}
      src={meditationAudioUrl(meditation.id)}
      preload="metadata"
      class="hidden"
      onloadedmetadata={onLoadedMetadata}
      ondurationchange={onDurationChange}
      ontimeupdate={onTimeUpdate}
      onseeked={() => audio && (position = audio.currentTime)}
      onplay={onPlay}
      onpause={onPause}
      onended={onEnded}
      onerror={() => void onAudioError()}
    ></audio>

    {#if playbackError}
      <InlineMessage tone="error" role="alert"
        >{t(m.playbackError)}</InlineMessage
      >
    {/if}

    <PlayerTransport
      bind:position
      {duration}
      {playing}
      {playbackError}
      {speedText}
      {skipLabel}
      mixPlaying={mix.state.playing}
      onSeek={seekTo}
      onRestart={restart}
      onSkip={skip}
      onTogglePlay={togglePlay}
      onCycleSpeed={() => (speedIndex = nextSpeedIndex(speedIndex))}
      onOpenMix={() => (mixOpen = true)}
    />

    {#if notice}
      <InlineMessage tone="error" role="status">{t(notice)}</InlineMessage>
    {/if}

    <div class="flex flex-wrap items-center justify-end gap-4">
      <OverflowMenu label={t(m.moreActionsFor, { name: meditation.name })}>
        <OverflowItem
          icon="download-simple"
          label={t(m.download)}
          onclick={() => downloadFile(meditationDownloadUrl(meditation.id))}
        />
      </OverflowMenu>
    </div>
  </Panel>
</div>

<PlayerMixDialog bind:open={mixOpen} {mix} {playing} />
