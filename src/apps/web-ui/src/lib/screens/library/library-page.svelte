<!-- Logic-free wrapper (plan D1): the shell gates and owns the header search. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import LibraryContent from "./library-content.svelte";
</script>

<AppShell screen="library" search={true}>
  {#snippet children()}<LibraryContent />{/snippet}
</AppShell>
