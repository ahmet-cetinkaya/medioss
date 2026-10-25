<!--
  @component
  About panel: instance facts from the public `/api/meta` (name, version,
  license), a source link, and the donation link. Read-only, so it renders
  last in Settings and never blocks the panels above it: a failed load shows
  a retry row instead of version text.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { fetchAppMeta, type AppMeta } from "$lib/api/meta";
  import CoffeeIcon from "phosphor-svelte/lib/CoffeeIcon";
  import GithubLogoIcon from "phosphor-svelte/lib/GithubLogoIcon";
  import PackageIcon from "phosphor-svelte/lib/PackageIcon";
  import ScalesIcon from "phosphor-svelte/lib/ScalesIcon";
  import BrandLockup from "$lib/components/content/brand-lockup.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import SupportDialog from "$lib/components/dialogs/support-dialog.svelte";
  import { shellMessages } from "$lib/components/shell/shell.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { aboutMessages as m } from "./about.messages";

  let supportOpen = $state(false);
  let meta = $state<AppMeta | null>(null);
  let failed = $state(false);
  let request = 0;

  async function load() {
    const current = ++request;
    failed = false;
    try {
      const loaded = await fetchAppMeta();
      if (current === request) meta = loaded;
    } catch {
      if (current === request) failed = true;
    }
  }

  onMount(() => void load());
</script>

<div class="mx-auto w-full max-w-xl text-center">
  <Panel aria-labelledby="about-heading">
    {#snippet heading()}
      <h2
        id="about-heading"
        class="flex flex-wrap items-center justify-center gap-3"
      >
        <BrandLockup />
        {#if meta}
          <span
            class="rounded-full bg-surface-solid px-3 py-1 text-xs font-medium text-ink-strong"
            >{`v${meta.version}`}</span
          >
        {/if}
      </h2>
    {/snippet}
    <div class="flex flex-col gap-4">
      <p class="text-body-md text-ink-body">{t(m.description)}</p>
      {#if failed}
        <RegionState
          status="error"
          minHeight="4rem"
          errorMessage={t(m.loadFailed)}
          onRetry={() => void load()}
        >
          <span class="sr-only">{t(m.loadFailed)}</span>
        </RegionState>
      {:else if !meta}
        <p class="text-sm text-ink-muted" aria-busy="true">…</p>
      {/if}
      <div class="flex flex-wrap items-center justify-center gap-3">
        {#if meta}
          <Button
            variant="secondary"
            href={meta.source}
            target="_blank"
            rel="noopener noreferrer"
          >
            <GithubLogoIcon aria-hidden="true" />
            {t(m.openSource)}
          </Button>
          <Button
            variant="secondary"
            href={`${meta.source}/blob/main/LICENSE`}
            target="_blank"
            rel="noopener noreferrer"
          >
            <ScalesIcon aria-hidden="true" />
            {t(m.licenseFullText)}
          </Button>
          <Button
            variant="secondary"
            href={`${meta.source}/blob/main/NOTICE`}
            target="_blank"
            rel="noopener noreferrer"
          >
            <PackageIcon aria-hidden="true" />
            {t(m.thirdPartyOpen)}
          </Button>
        {/if}
        <Button
          variant="secondary"
          size="lg"
          class="border-transparent bg-[#ffdd00] font-semibold text-[#1a1a1a] hover:bg-[#ffe84d]"
          onclick={() => (supportOpen = true)}
        >
          <CoffeeIcon weight="fill" aria-hidden="true" />
          {t(shellMessages.support)}
        </Button>
      </div>
      <SupportDialog bind:open={supportOpen} />
    </div>
  </Panel>
</div>
