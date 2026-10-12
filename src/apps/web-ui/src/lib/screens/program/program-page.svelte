<!-- Logic-free wrapper (plan D1): the shell gates, ProgramContent loads. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import ProgramContent from "./program-content.svelte";
</script>

<AppShell screen="program">
  {#snippet children()}<ProgramContent />{/snippet}
</AppShell>
