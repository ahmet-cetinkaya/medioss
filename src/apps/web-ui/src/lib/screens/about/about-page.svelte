<!-- Logic-free wrapper (plan D1): the shell gates, AboutContent renders. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import AboutContent from "./about-content.svelte";
</script>

<AppShell screen="about">
  {#snippet children()}<AboutContent />{/snippet}
</AppShell>
