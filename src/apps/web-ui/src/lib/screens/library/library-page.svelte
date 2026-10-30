<!-- Logic-free wrapper (plan D1): the shell gates and owns the header search;
     the filter dropdown rides the header row next to it (one row on desktop). -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import LibraryHeaderFilter from "./library-header-filter.svelte";
  import LibraryContent from "./library-content.svelte";
</script>

<AppShell screen="library" search={true}>
  {#snippet headerActions()}
    <LibraryHeaderFilter />
  {/snippet}
  {#snippet children()}<LibraryContent />{/snippet}
</AppShell>
