<!-- Logic-free wrapper (plan D1): the shell gates, SoundscapeContent loads. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import SoundscapeContent from "./soundscape-content.svelte";
</script>

<AppShell screen="soundscape">
  {#snippet children()}<SoundscapeContent />{/snippet}
</AppShell>
