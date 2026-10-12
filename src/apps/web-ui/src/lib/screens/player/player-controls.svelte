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
    programCoverUrl,
    type Meditation,
    type PlaybackState,
    type ProgramSummary,
  } from "$lib/api/meditation";
  import GeneratedArtwork from "$lib/components/content/generated-artwork.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { IconButton } from "$lib/components/ui/icon-button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import {
    formatClock,
    formatClockSpoken,
  } from "$lib/i18n/formatting/clock-format";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import type { StringMessage } from "$lib/i18n/message";
  import { redirectToLogin } from "$lib/navigation/login-redirect";
  import { ROUTES } from "$lib/navigation/routes";
  import { createCompletionTracker } from "./completion-tracker";
  import { createPositionReporter } from "./position-reporter";
  import type { PlayerMix } from "./player-mix.svelte";
  import PlayerMixDialog from "./player-mix-dialog.svelte";
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

  const SKIP_SECONDS = 10;
  const IDLE_MS = 5000;
  const SPEEDS = [1, 1.25, 1.5] as const;

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

  const mediaSession = () =>
    typeof navigator !== "undefined" && "mediaSession" in navigator
      ? navigator.mediaSession
      : null;

  function syncSessionState() {
    const session = mediaSession();
    if (session && audio)
      session.playbackState = audio.paused ? "paused" : "playing";
  }

  function seekTo(seconds: number) {
    if (!audio) return;
    audio.currentTime = Math.min(Math.max(0, seconds), duration);
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
    if (audio && Number.isFinite(audio.duration) && audio.duration > 0)
      reportedDuration = audio.duration;
  }

  function onPlay() {
    playing = true;
    reporter.play();
    tracker.play();
    syncSessionState();
    void mix.start(() => playing);
  }

  function onPause() {
    playing = false;
    reporter.pause();
    syncSessionState();
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
    const element = stage;
    const parent = element.parentElement;
    const next = element.nextSibling;
    document.body.append(element);
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    // Hide the rest of the page UI: the backdrop is an opaque color wash,
    // so only the floating panel and open dialogs remain (see global.css).
    document.body.classList.add("player-focus-on");
    return () => {
      document.body.style.overflow = previousOverflow;
      document.body.classList.remove("player-focus-on");
      parent?.insertBefore(element, next);
    };
  });

  // Idle auto-hide: after IDLE_MS without input while playing, the panel
  // fades out and only the big elapsed clock remains. Any input restores it.
  let idle = $state(false);
  $effect(() => {
    if (!playing) {
      idle = false;
      return;
    }
    let timer: ReturnType<typeof setTimeout>;
    const wake = () => {
      idle = false;
      clearTimeout(timer);
      timer = setTimeout(() => (idle = true), IDLE_MS);
    };
    const events = ["pointermove", "pointerdown", "keydown"] as const;
    for (const name of events) window.addEventListener(name, wake, true);
    wake();
    return () => {
      clearTimeout(timer);
      for (const name of events) window.removeEventListener(name, wake, true);
    };
  });

  // Media Session (PB-5).
  $effect(() => {
    const session = mediaSession();
    const element = audio;
    if (!session || !element) return;
    session.metadata = new MediaMetadata({ title: meditation.name });
    const handlers: [MediaSessionAction, MediaSessionActionHandler][] = [
      ["play", () => void element.play().catch(() => {})],
      ["pause", () => element.pause()],
      ["seekbackward", () => skip(-SKIP_SECONDS)],
      ["seekforward", () => skip(SKIP_SECONDS)],
      [
        "seekto",
        (details) => {
          if (details.seekTime !== undefined) seekTo(details.seekTime);
        },
      ],
      [
        "stop",
        () => {
          element.pause();
          seekTo(0);
          syncSessionState();
        },
      ],
    ];
    for (const [action, handler] of handlers) {
      try {
        session.setActionHandler(action, handler);
      } catch {
        // Unsupported action in this browser.
      }
    }
    return () => {
      for (const [action] of handlers) {
        try {
          session.setActionHandler(action, null);
        } catch {
          // Unsupported action in this browser.
        }
      }
      session.metadata = null;
    };
  });

  // Hide reports (PB-2): `pagehide` fires on window, not document.
  onMount(() => {
    const hide = () => reporter.hide();
    const show = () => reporter.show();
    const visibility = () =>
      document.visibilityState === "hidden" ? hide() : show();
    window.addEventListener("pagehide", hide);
    window.addEventListener("pageshow", show);
    document.addEventListener("visibilitychange", visibility);
    return () => {
      window.removeEventListener("pagehide", hide);
      window.removeEventListener("pageshow", show);
      document.removeEventListener("visibilitychange", visibility);
    };
  });
</script>

{#snippet coverTitle()}
  <div
    class="absolute inset-x-0 bottom-0 z-10 flex flex-col items-start gap-1 p-5 sm:p-6"
  >
    <h1
      class="max-w-full rounded-full bg-surface-solid/85 px-4 py-2 text-title-lg break-words text-ink-strong backdrop-blur-sm"
    >
      {meditation.name}
    </h1>
  </div>
{/snippet}

{#snippet artwork()}
  {#if program && showCover}
    <img
      src={programCoverUrl(program.id)}
      alt=""
      class="absolute inset-0 size-full object-cover"
      onerror={() => (coverFailed = true)}
    />
    <span
      aria-hidden="true"
      class="absolute inset-0 bg-(image:--gradient-image-wash)"
    ></span>
  {:else if program !== undefined}
    <GeneratedArtwork
      seed={program?.id ?? meditation.id}
      category="meditation"
      wash
    />
  {/if}
{/snippet}

<!-- The overlay stage: while playing the $effect above moves this whole
     div (backdrop + floating panel) under <body>, so `position: fixed`
     escapes the glass shell's backdrop-filter containing block. Paused, the
     panel renders inline here and the backdrop is unmounted. The same
     $effect toggles .player-focus-on on <body>, hiding the rest of the
     page UI behind the opaque wash. -->
<div bind:this={stage} data-player-floating="" class="min-w-0">
  {#if playing}
    <div
      aria-hidden="true"
      class="focus-overlay-backdrop fixed inset-0 z-40"
      data-player-focus-backdrop=""
    >
      {#if program && showCover}
        <img
          src={programCoverUrl(program.id)}
          alt=""
          class="focus-overlay-cover absolute inset-0 size-full object-cover"
          onerror={() => (coverFailed = true)}
        />
      {/if}
      <div
        class="focus-overlay-wash absolute inset-0"
        data-over-cover={program && showCover ? "" : undefined}
      ></div>
    </div>
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

    <!-- Inline: a wide cover with the name labelled at its bottom-left.
         Floating: a static centered circular cover with the name below. -->
    {#if playing}
      <div class="flex flex-col items-center gap-4">
        {#if program}
          <a
            href={`${ROUTES.program.path}?id=${program.id}`}
            aria-label={t(m.programLink, { name: program.title })}
            class="focus-ring relative block size-40 overflow-hidden rounded-full bg-surface-solid/50"
          >
            {@render artwork()}
          </a>
        {:else}
          <div
            class="relative block size-40 overflow-hidden rounded-full bg-surface-solid/50"
          >
            {@render artwork()}
          </div>
        {/if}
        <div class="flex min-w-0 flex-col items-center gap-2">
          <h1
            class="min-w-0 text-title-lg text-center break-words text-ink-strong"
          >
            {meditation.name}
          </h1>
          {#if meditation.description}
            <p
              class="max-w-prose text-body-md text-center break-words text-ink-body"
            >
              {meditation.description}
            </p>
          {/if}
        </div>
      </div>
    {:else}
      <div class="flex min-w-0 flex-col gap-3">
        {#if program}
          <a
            href={`${ROUTES.program.path}?id=${program.id}`}
            aria-label={t(m.programLink, { name: program.title })}
            class="focus-ring relative block aspect-video w-full overflow-hidden lg:aspect-auto lg:h-72 lg:max-h-72 rounded-tile bg-surface-solid/50"
          >
            {@render artwork()}
            {@render coverTitle()}
          </a>
        {:else}
          <div
            class="relative block aspect-video w-full overflow-hidden lg:aspect-auto lg:h-72 lg:max-h-72 rounded-tile bg-surface-solid/50"
          >
            {@render artwork()}
            {@render coverTitle()}
          </div>
        {/if}
        {#if meditation.description}
          <p class="text-body-md break-words text-ink-body">
            {meditation.description}
          </p>
        {/if}
      </div>
    {/if}

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
        oninput={(event) => seekTo(event.currentTarget.valueAsNumber)}
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

    <div class="flex flex-wrap items-center justify-center gap-4">
      <IconButton
        label={t(m.restart)}
        icon="skip-back"
        disabled={playbackError}
        onclick={restart}
      />
      <div class="relative">
        <IconButton
          label={t(m.skipBack)}
          icon="arrow-counter-clockwise"
          disabled={playbackError}
          onclick={() => skip(-SKIP_SECONDS)}
        />
        <span
          aria-hidden="true"
          class="pointer-events-none absolute inset-0 flex items-center justify-center pt-px text-[0.625rem] font-bold text-ink-strong"
        >
          {skipLabel}
        </span>
      </div>
      <IconButton
        label={t(playing ? actionMessages.pause : actionMessages.play)}
        icon={playing ? "pause" : "play"}
        filled
        disabled={playbackError}
        class="size-16 [&_svg]:size-7"
        onclick={togglePlay}
      />
      <div class="relative">
        <IconButton
          label={t(m.skipForward)}
          icon="arrow-clockwise"
          disabled={playbackError}
          onclick={() => skip(SKIP_SECONDS)}
        />
        <span
          aria-hidden="true"
          class="pointer-events-none absolute inset-0 flex items-center justify-center pt-px text-[0.625rem] font-bold text-ink-strong"
        >
          {skipLabel}
        </span>
      </div>
      <Button
        variant="secondary"
        aria-label={t(m.speedButton, { value: speedText })}
        onclick={() => (speedIndex = (speedIndex + 1) % SPEEDS.length)}
      >
        {speedText}
      </Button>
      <Button
        variant="secondary"
        aria-pressed={mixOpen || undefined}
        onclick={() => (mixOpen = true)}
      >
        <AppIcon name="speaker-high" size={20} />
        {t(m.mixToggle)}
      </Button>
    </div>

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
