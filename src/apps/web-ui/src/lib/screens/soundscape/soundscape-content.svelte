<!--
  @component
  Soundscape content (FR-049–FR-054, PB-6/PB-7/PB-8/PB-16). Loads the sound
  library (categories + sounds) and the saved mixes as two regions; the
  current mix lives in one MixState (plan D10) shared by the panels.
-->
<script lang="ts">
  import { type ServerReason } from "$lib/api/server-reason";
  import {
    failureMessage,
    NETWORK_OR_SERVER_REASONS,
    reasonOf,
  } from "$lib/api/failure";
  import {
    fetchSoundscapeCategories,
    fetchSoundscapeMixes,
    fetchSounds,
    type SoundscapeCategory,
    type SoundscapeMix,
    type SoundscapeSound,
  } from "$lib/api/soundscape";
  import { t } from "$lib/i18n/translate";
  import MixPanel from "./mix-panel.svelte";
  import { MixState } from "./mix-state.svelte";
  import SavedMixes from "./saved-mixes.svelte";
  import SoundLibrary from "./sound-library.svelte";
  import { soundscapeMessages as m } from "./soundscape.messages";

  const mix = new MixState();

  let library = $state<{
    categories: SoundscapeCategory[];
    sounds: SoundscapeSound[];
  } | null>(null);
  let libraryFailure = $state<ServerReason | null>(null);
  let mixes = $state<SoundscapeMix[] | null>(null);
  let mixesFailure = $state<ServerReason | null>(null);
  let libraryRequest = 0;
  let mixesRequest = 0;

  async function loadLibrary() {
    const id = ++libraryRequest;
    library = null;
    libraryFailure = null;
    try {
      const [categories, sounds] = await Promise.all([
        fetchSoundscapeCategories(),
        fetchSounds(),
      ]);
      if (id === libraryRequest) library = { categories, sounds };
    } catch (error) {
      if (id === libraryRequest) libraryFailure = reasonOf(error);
    }
  }

  async function loadMixes() {
    const id = ++mixesRequest;
    mixes = null;
    mixesFailure = null;
    try {
      const result = await fetchSoundscapeMixes();
      if (id === mixesRequest) mixes = result;
    } catch (error) {
      if (id === mixesRequest) mixesFailure = reasonOf(error);
    }
  }

  $effect(() => {
    void loadLibrary();
    void loadMixes();
    return () => mix.dispose();
  });
</script>

<div class="grid grid-cols-1 items-start gap-6">
  <div class="flex min-w-0 flex-col gap-6">
    <MixPanel
      {mix}
      sounds={library?.sounds ?? []}
      categories={library?.categories ?? []}
      onSaved={(saved) => (mixes = [saved, ...(mixes ?? [])])}
    />
  </div>

  <div
    class="grid min-w-0 grid-cols-1 items-start gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,3fr)]"
  >
    <SavedMixes
      {mix}
      bind:mixes
      sounds={library?.sounds ?? null}
      status={mixesFailure
        ? "error"
        : mixes
          ? mixes.length
            ? "ready"
            : "empty"
          : "loading"}
      errorMessage={mixesFailure
        ? t(
            failureMessage(mixesFailure, m.mixesLoadFailed, {
              specificReasons: NETWORK_OR_SERVER_REASONS,
            }).message,
          )
        : undefined}
      onReload={() => void loadMixes()}
    />
    <SoundLibrary
      {mix}
      {library}
      status={libraryFailure
        ? "error"
        : library
          ? library.sounds.length
            ? "ready"
            : "empty"
          : "loading"}
      errorMessage={libraryFailure
        ? t(
            failureMessage(libraryFailure, m.soundsLoadFailed, {
              specificReasons: NETWORK_OR_SERVER_REASONS,
            }).message,
          )
        : undefined}
      onRetry={() => void loadLibrary()}
    />
  </div>
</div>
