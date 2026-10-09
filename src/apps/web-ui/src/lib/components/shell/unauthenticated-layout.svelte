<!--
  @component
  The unauthenticated layout (FR-079, DESIGN_SYSTEM §10): canvas, brand
  lockup, one centered glass panel with the page's h1, an optional footer
  (e.g. Sign out on the change-password view, links on Login), and the
  language switcher (FR-112). No navigation. Mounts the page's live
  announcer and sets `<html lang>` and the document title (FR-005, FR-116).

  Used by Login, Setup, and the forced change-password view.

  Props: `title` (the screen title message; h1 and document title),
  `children` (panel content), `footer?` (snippet below the panel).
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import BrandLockup from "$lib/components/content/brand-lockup.svelte";
  import LiveAnnouncer from "$lib/components/feedback/live-announcer.svelte";
  import LanguageSwitcher from "$lib/components/i18n/language-switcher.svelte";
  import Panel from "$lib/components/ui/panel/panel.svelte";
  import { brandMessages } from "$lib/i18n/common.messages";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import type { StringMessage } from "$lib/i18n/message";
  import { shellMessages } from "./shell.messages";

  type Props = {
    title: StringMessage;
    children: Snippet;
    footer?: Snippet;
  };

  let { title, children, footer }: Props = $props();

  const headingId = $props.id();

  $effect(() => {
    document.documentElement.lang = locale.current;
    document.title = t(shellMessages.documentTitle, {
      screen: t(title),
      product: t(brandMessages.productName),
    });
  });
</script>

<div
  class="flex min-h-dvh flex-col items-center justify-center gap-6 px-4 py-10"
>
  <BrandLockup />
  <main id="main" tabindex="-1" class="w-full max-w-md outline-none">
    <Panel aria-labelledby={headingId} class="flex flex-col gap-5">
      <h1 id={headingId} class="text-2xl font-semibold text-ink-strong">
        {t(title)}
      </h1>
      {@render children()}
    </Panel>
  </main>
  {#if footer}
    <div class="flex w-full max-w-md flex-col items-center gap-3">
      {@render footer()}
    </div>
  {/if}
  <LanguageSwitcher class="flex flex-col items-center" />
</div>
<LiveAnnouncer />
