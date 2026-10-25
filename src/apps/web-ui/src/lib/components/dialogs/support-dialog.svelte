<!--
  @component
  Support dialog: a short, honest invitation to support MediOSS, with the
  donation link. Opened by every Support button (sidebar, More sheet, About).
  Props: `open` (bindable).
-->
<script lang="ts">
  import CoffeeIcon from "phosphor-svelte/lib/CoffeeIcon";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { shellMessages as m } from "$lib/components/shell/shell.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { DONATE_URL } from "$lib/navigation/support";

  let { open = $bindable(false) }: { open?: boolean } = $props();
</script>

<Dialog.Root bind:open>
  <Dialog.Content>
    <span aria-hidden="true" class="heart-glow mx-auto block size-17">
      <span class="heart block size-full"></span>
    </span>
    <Dialog.Header class="text-center">
      <Dialog.Title>{t(m.supportTitle)}</Dialog.Title>
      <Dialog.Description>{t(m.supportStory)}</Dialog.Description>
    </Dialog.Header>
    <p class="text-body-md text-ink-body">{t(m.supportFunds)}</p>
    <Dialog.Footer>
      <Button
        variant="secondary"
        size="lg"
        class="border-transparent bg-[#ffdd00] font-semibold text-[#1a1a1a] hover:bg-[#ffe84d]"
        href={DONATE_URL}
        target="_blank"
        rel="noopener noreferrer"
      >
        <CoffeeIcon weight="fill" aria-hidden="true" />
        {t(m.supportAction)}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<style>
  /* An orb-styled heart: the brand orb gradient and a soft glass highlight,
     cut to a heart shape. The thin edge and the glow sit on a wrapper because a mask clips shadows. */
  .heart-glow {
    filter: drop-shadow(0 0 0.75px rgb(110 58 120 / 0.9))
      drop-shadow(0 10px 14px rgb(126 83 173 / 0.45));
  }

  .heart {
    background-image:
      radial-gradient(
        circle at 32% 26%,
        rgb(255 255 255 / 0.65),
        transparent 45%
      ),
      var(--gradient-orb);
    -webkit-mask: var(--heart-shape) center / contain no-repeat;
    mask: var(--heart-shape) center / contain no-repeat;
    --heart-shape: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'%3E%3Cpath d='M240,102c0,70-103.79,126.66-108.21,129a8,8,0,0,1-7.58,0C119.79,228.66,16,172,16,102A62.07,62.07,0,0,1,78,40c20.65,0,38.73,8.88,50,23.89C139.27,48.88,157.35,40,178,40A62.07,62.07,0,0,1,240,102Z'/%3E%3C/svg%3E");
  }
</style>
