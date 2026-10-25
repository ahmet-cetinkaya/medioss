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
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { t } from "$lib/i18n/locale.svelte";
  import notice from "$repo/NOTICE?raw";
  import { aboutMessages as m } from "./settings.messages";
  import ThirdPartyLicensesDialog from "./third-party-licenses-dialog.svelte";

  /** Canonical donation page; the only link the app advertises. */
  const DONATE_URL = "https://ahmetcetinkaya.me/donate";

  /** The NOTICE paragraph stating the project license, shown verbatim. */
  const LICENSE_STATEMENT = notice.split("\n\n")[1]?.replace(/\s+/g, " ");

  let thirdPartyOpen = $state(false);
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

<Panel aria-labelledby="settings-about-heading">
  {#snippet heading()}
    <h2 id="settings-about-heading" class="text-heading text-ink-strong">
      {t(m.heading)}
    </h2>
  {/snippet}
  <div class="flex flex-col gap-4">
    <p class="text-body-md text-ink-body">{t(m.donateIntro)}</p>
    {#if meta}
      <dl class="flex flex-col gap-2 text-sm">
        <div class="flex gap-2">
          <dt class="text-ink-muted">{t(m.version)}</dt>
          <dd class="font-medium text-ink-strong">{meta.version}</dd>
        </div>
        <div class="flex gap-2">
          <dt class="text-ink-muted">{t(m.license)}</dt>
          <dd class="font-medium text-ink-strong">{meta.license}</dd>
        </div>
        <div class="flex gap-2">
          <dt class="text-ink-muted">{t(m.source)}</dt>
          <dd>
            <a
              class="focus-ring rounded-sm text-accent-strong underline underline-offset-2"
              href={meta.source}
              rel="noopener noreferrer"
              target="_blank"
            >
              {meta.source}
            </a>
          </dd>
        </div>
      </dl>
      <p class="text-sm text-ink-body">{LICENSE_STATEMENT}</p>
      <div class="flex flex-wrap gap-3">
        <Button
          variant="secondary"
          href={`${meta.source}/blob/main/LICENSE`}
          target="_blank"
          rel="noopener noreferrer"
        >
          {t(m.licenseFullText)}
        </Button>
        <Button variant="secondary" onclick={() => (thirdPartyOpen = true)}>
          {t(m.thirdPartyOpen)}
        </Button>
      </div>
      <ThirdPartyLicensesDialog bind:open={thirdPartyOpen} />
    {:else if failed}
      <RegionState
        status="error"
        minHeight="4rem"
        errorMessage={t(m.loadFailed)}
        onRetry={() => void load()}
      >
        <span class="sr-only">{t(m.loadFailed)}</span>
      </RegionState>
    {:else}
      <p class="text-sm text-ink-muted" aria-busy="true">…</p>
    {/if}
    <div>
      <Button
        variant="secondary"
        size="lg"
        class="border-transparent bg-[#ffdd00] font-semibold text-[#1a1a1a] hover:bg-[#ffe84d]"
        href={DONATE_URL}
        target="_blank"
        rel="noopener noreferrer"
      >
        <CoffeeIcon weight="fill" aria-hidden="true" />
        {t(m.donate)}
      </Button>
    </div>
  </div>
</Panel>
