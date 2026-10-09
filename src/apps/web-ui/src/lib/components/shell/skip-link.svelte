<!--
  @component
  Skip link (FR-096): the first focusable element of every authenticated
  screen. Visually hidden until focused; moves focus to `#main`
  (`tabindex="-1"`).
-->
<script lang="ts">
  import { t } from "$lib/i18n/locale.svelte";
  import { shellMessages } from "./shell.messages";

  function skip(event: MouseEvent) {
    const main = document.getElementById("main");
    if (!main) return;
    event.preventDefault();
    main.focus();
  }
</script>

<a
  href="#main"
  onclick={skip}
  class="focus-ring glass-solid elevation-2 sr-only z-50 rounded-full px-5 py-3 font-medium text-ink-strong focus:not-sr-only focus:fixed focus:start-4 focus:top-4 focus:inline-flex focus:min-h-11 focus:items-center"
>
  {t(shellMessages.skipToContent)}
</a>
