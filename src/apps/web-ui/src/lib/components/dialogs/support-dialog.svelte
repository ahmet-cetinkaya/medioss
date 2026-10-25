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
    <Dialog.Header>
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
