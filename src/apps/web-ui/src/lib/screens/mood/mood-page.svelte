<!-- Logic-free wrapper (plan D1): the shell gates, MoodContent loads. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import MoodContent from "./mood-content.svelte";
</script>

<AppShell screen="mood">
  {#snippet children()}<MoodContent />{/snippet}
</AppShell>
