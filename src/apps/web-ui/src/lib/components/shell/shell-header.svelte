<!--
  @component
  The shell header row (DESIGN_SYSTEM §4.1, §5.3, §5.4; FR-005, FR-006): the
  page title as the page's only h1, the optional Library search, and the
  account button. Below 768 px it is the sticky compact top bar (4 rem tall)
  with the compact brand lockup, and the search moves to its own row beneath it.

  Props: `title` (translated screen name or runtime override; `null` renders no
  h1, only a spacer, for a screen whose content carries its own h1), `search`.
-->
<script lang="ts">
  import BrandLockup from "$lib/components/content/brand-lockup.svelte";
  import HeaderSearch from "./header-search.svelte";

  type Props = {
    title: string | null;
    search?: boolean;
  };

  let { title, search = false }: Props = $props();
</script>

<header
  class="sticky top-0 z-20 flex h-16 items-center gap-3 px-4 max-md:glass-shell max-md:rounded-b-panel max-md:border-t-0 md:static md:h-auto md:gap-4 md:px-8 md:pt-8"
>
  <BrandLockup compact href="/" class="md:hidden" />
  {#if title === null}
    <div class="flex-1"></div>
  {:else}
    <h1
      class="min-w-0 flex-1 truncate text-xl font-semibold text-ink-strong md:overflow-visible md:text-4xl md:break-words md:whitespace-normal xl:text-display"
    >
      {title}
    </h1>
  {/if}
  {#if search}
    <HeaderSearch class="hidden w-72 md:block lg:w-80" />
  {/if}
</header>
{#if search}
  <div class="px-4 pt-4 md:hidden">
    <HeaderSearch class="w-full" />
  </div>
{/if}
