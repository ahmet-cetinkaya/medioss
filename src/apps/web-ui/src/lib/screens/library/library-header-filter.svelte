<!--
  @component
  The Library filter dropdown for the shell header row, next to the search
  (one row on desktop). Shares `libraryFilter` with LibraryContent.
-->
<script lang="ts">
  import FilterSelect from "$lib/components/filters/filter-select.svelte";
  import { t } from "$lib/i18n/locale.svelte";
  import { libraryMessages as m } from "./library.messages";
  import { libraryFilter } from "./library-filter.svelte";

  const options = $derived([
    { value: "all" as const, label: t(m.all) },
    { value: "completed" as const, label: t(m.filterCompleted) },
    { value: "in-progress" as const, label: t(m.filterInProgress) },
    { value: "not-started" as const, label: t(m.filterNotStarted) },
    { value: "favorites" as const, label: t(m.filterFavorites) },
  ]);
</script>

<FilterSelect
  {options}
  bind:value={libraryFilter.value}
  legend={t(m.filterLegend)}
/>
