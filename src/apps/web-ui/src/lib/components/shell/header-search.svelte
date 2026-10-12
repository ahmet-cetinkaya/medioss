<!--
  @component
  Library search in the shell header (DESIGN_SYSTEM §5.3; spec: Library
  only). Submitting pushes `?q=` through `libraryQuery` (a whitespace-only
  query clears it); back/forward and Clear search update the field.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { t } from "$lib/i18n/locale.svelte";
  import { libraryQuery } from "$lib/library/library-query.svelte";
  import { shellMessages } from "./shell.messages";

  let { class: className = "" }: { class?: string } = $props();

  const id = $props.id();
  let value = $state("");

  onMount(() => libraryQuery.connect());

  // Follows the address (connect, popstate, clear); typing does not re-run it.
  $effect(() => {
    value = libraryQuery.query;
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    libraryQuery.submit(value);
  }
</script>

<form role="search" class="relative {className}" onsubmit={submit}>
  <label for={id} class="sr-only">{t(shellMessages.searchLabel)}</label>
  <AppIcon
    name="magnifying-glass"
    size={20}
    class="pointer-events-none absolute start-4 top-1/2 -translate-y-1/2 text-ink-muted"
  />
  <input
    {id}
    type="search"
    enterkeyhint="search"
    autocomplete="off"
    bind:value
    placeholder={t(shellMessages.searchPlaceholder)}
    class="focus-ring h-12 w-full min-w-0 rounded-full border border-rim bg-surface-solid ps-12 pe-4 text-base text-ink-strong placeholder:text-ink-muted"
  />
</form>
