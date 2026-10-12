<!-- Logic-free wrapper (plan D1): the shell gates, HistoryContent loads. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import HistoryContent from "./history-content.svelte";
</script>

<AppShell screen="history">
  {#snippet children()}<HistoryContent />{/snippet}
</AppShell>
