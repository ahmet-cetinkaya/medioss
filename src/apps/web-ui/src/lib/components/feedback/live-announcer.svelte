<!--
  @component
  The page's single polite live region (FR-021). Mount once per page (the
  shell does); announce through `announce()` from `announcer.svelte.ts`.
-->
<script lang="ts">
  import { announcement } from "./announcer.svelte";
</script>

<div
  data-live-announcer
  class="sr-only"
  role="status"
  aria-live="polite"
  aria-atomic="true"
>
  {announcement.message}
</div>
