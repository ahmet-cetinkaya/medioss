<!--
  @component
  Settings "Your data" panel (FR-069, PB-12, PB-16).

  - Exports are real same-origin `<a href download>` links styled as buttons:
    JSON (all data) and CSV (mood entries only).
  - Import sends the chosen JSON or CSV file once per activation; success
    shows all five counts the server returns, a refusal shows the specific
    reason (never server text) and no report.
-->
<script lang="ts">
  import { failureMessage, reasonOf } from "$lib/api/failure";
  import {
    exportCsvUrl,
    exportJsonUrl,
    importData,
    type ImportReport,
  } from "$lib/api/portability";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { dataMessages as m } from "./settings.messages";

  const id = $props.id();

  let file = $state<File | null>(null);
  let busy = $state(false);
  let report = $state<ImportReport | null>(null);
  let failure = $state<ReturnType<typeof reasonOf> | null>(null);

  const counts = $derived(
    report
      ? [
          { label: m.moodCreated, value: report.mood_created },
          { label: m.moodDuplicates, value: report.mood_duplicates_skipped },
          { label: m.practiceLinked, value: report.practice_linked },
          { label: m.practiceSkipped, value: report.practice_skipped },
          {
            label: m.practiceDuplicates,
            value: report.practice_duplicates_skipped,
          },
        ]
      : [],
  );

  async function runImport(event: SubmitEvent) {
    event.preventDefault();
    if (!file || busy) return;
    busy = true;
    report = null;
    failure = null;
    try {
      report = await importData(file);
      announce(t(m.imported));
    } catch (error) {
      failure = reasonOf(error);
    } finally {
      busy = false;
    }
  }
</script>

<Panel aria-labelledby="{id}-heading">
  {#snippet heading()}
    <h2 id="{id}-heading" class="text-heading text-ink-strong">
      {t(m.heading)}
    </h2>
    <p class="mt-1 text-sm text-ink-muted">{t(m.intro)}</p>
  {/snippet}

  <div class="flex flex-col gap-6">
    <div class="flex flex-wrap gap-2">
      <Button
        variant="secondary"
        href={exportJsonUrl()}
        download=""
        class="h-auto max-w-full py-2 text-left whitespace-normal"
      >
        <AppIcon name="download-simple" size={18} />
        {t(m.exportJson)}
      </Button>
      <Button
        variant="secondary"
        href={exportCsvUrl()}
        download=""
        class="h-auto max-w-full py-2 text-left whitespace-normal"
      >
        <AppIcon name="download-simple" size={18} />
        {t(m.exportCsv)}
      </Button>
    </div>

    <form class="flex flex-col gap-2" novalidate onsubmit={runImport}>
      <label for="{id}-file" class="text-sm font-medium text-ink-strong">
        {t(m.importLabel)}
      </label>
      <Input
        id="{id}-file"
        type="file"
        accept=".json,.csv,application/json,text/csv"
        onchange={(event) => {
          file = event.currentTarget.files?.[0] ?? null;
          failure = null;
        }}
      />
      {#if failure}
        <InlineMessage tone="error" role="alert">
          {t(failureMessage(failure, m.importFailed).message)}
        </InlineMessage>
      {/if}
      <div class="mt-2">
        <Button type="submit" {busy} disabled={!file}>
          <AppIcon name="upload-simple" size={18} />
          {t(m.importAction)}
        </Button>
      </div>
    </form>

    {#if report}
      <section aria-labelledby="{id}-report" class="flex flex-col gap-3">
        <h3 id="{id}-report" class="text-base font-semibold text-ink-strong">
          {t(m.reportHeading)}
        </h3>
        <dl class="grid grid-cols-1 gap-2 sm:grid-cols-2">
          {#each counts as count (count.label)}
            <div
              class="flex items-baseline justify-between gap-4 rounded-tile bg-surface-solid/50 px-4 py-3"
            >
              <dt class="text-sm text-ink-body">{t(count.label)}</dt>
              <dd class="text-lg font-semibold text-ink-strong">
                {formatNumber(count.value, locale.current)}
              </dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}
  </div>
</Panel>
