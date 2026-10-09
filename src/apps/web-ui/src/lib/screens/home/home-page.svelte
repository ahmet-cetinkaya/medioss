<!-- Logic-free wrapper (plan D1): the shell gates; HomeContent loads. A null title suppresses the shell h1: the greeting is Home's h1. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import HomeContent from "./home-content.svelte";
</script>

<AppShell screen="home" title={null}>
  {#snippet children()}<HomeContent />{/snippet}
</AppShell>
