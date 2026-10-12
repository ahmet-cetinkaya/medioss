<!-- Logic-free wrapper (plan D1): the shell gates, PlayerContent loads and
     lifts the meditation name into `playerTitle` (FR-043). Once it is known the
     shell suppresses its h1 (the name is the in-content h1). -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import PlayerContent from "./player-content.svelte";
  import { playerTitle } from "./player-title.svelte";
</script>

<AppShell
  screen="player"
  title={playerTitle.name ? null : undefined}
  documentTitle={playerTitle.name}
>
  {#snippet children()}<PlayerContent />{/snippet}
</AppShell>
