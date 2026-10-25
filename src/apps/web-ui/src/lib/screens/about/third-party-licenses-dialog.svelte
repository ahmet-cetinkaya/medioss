<!--
  @component
  Modal listing the works bundled with MediOSS that the project license does
  not cover: the built-in sounds (one row per entry of the sound manifest) and
  the Poppins font. Built on the shared Dialog, so focus is trapped, Escape
  closes and the title labels the dialog.
-->
<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import { aboutMessages as m } from "./about.messages";
  import { soundGroups } from "./third-party-sounds";

  let { open = $bindable(false) }: { open?: boolean } = $props();

  const linkClass =
    "focus-ring rounded-sm text-accent-strong underline underline-offset-2";
  const MOODIST_URL = "https://github.com/remvze/moodist";
  const PIXABAY_LICENSE_URL = "https://pixabay.com/service/license-summary/";
  const CC0_URL = "https://creativecommons.org/publicdomain/zero/1.0/";
  const OFL_URL =
    "https://openfontlicense.org/open-font-license-official-text/";
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-w-2xl overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>{t(m.thirdPartyOpen)}</Dialog.Title>
      <Dialog.Description>{t(m.thirdPartyIntro)}</Dialog.Description>
    </Dialog.Header>
    <section class="grid gap-3" aria-labelledby="third-party-sounds-heading">
      <h3 id="third-party-sounds-heading" class="text-heading text-ink-strong">
        {t(m.soundsHeading)}
      </h3>
      <p class="text-sm text-ink-body">
        {t(m.soundsLicense)}
        <a
          class={linkClass}
          href={MOODIST_URL}
          rel="noopener noreferrer"
          target="_blank">{t(m.linkMoodist)}</a
        >,
        <a
          class={linkClass}
          href={PIXABAY_LICENSE_URL}
          rel="noopener noreferrer"
          target="_blank">{t(m.linkPixabay)}</a
        >,
        <a
          class={linkClass}
          href={CC0_URL}
          rel="noopener noreferrer"
          target="_blank">{t(m.linkCc0)}</a
        >
      </p>
      {#each soundGroups as group (group.category)}
        <h4 class="text-sm font-medium text-ink-strong">{group.category}</h4>
        <ul class="grid gap-1 text-sm text-ink-body">
          {#each group.sounds as sound (sound.title)}
            <li class="flex flex-wrap gap-x-2">
              <span class="capitalize">{sound.title}</span>
              {#if sound.sourceUrl}
                <a
                  class={linkClass}
                  href={sound.sourceUrl}
                  rel="noopener noreferrer"
                  target="_blank">{t(m.soundSource)}</a
                >
              {/if}
            </li>
          {/each}
        </ul>
      {/each}
    </section>
    <section class="grid gap-2" aria-labelledby="third-party-fonts-heading">
      <h3 id="third-party-fonts-heading" class="text-heading text-ink-strong">
        {t(m.fontsHeading)}
      </h3>
      <p class="text-sm text-ink-body">
        {t(m.fontLicense)}
        <a
          class={linkClass}
          href={OFL_URL}
          rel="noopener noreferrer"
          target="_blank">{t(m.linkOfl)}</a
        >
      </p>
    </section>
  </Dialog.Content>
</Dialog.Root>
