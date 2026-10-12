<!-- Logic-free wrapper (plan D1): the shell gates, StatsContent loads. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import StatsContent from "./stats-content.svelte";
</script>

<AppShell screen="stats">
  {#snippet children()}<StatsContent />{/snippet}
</AppShell>
