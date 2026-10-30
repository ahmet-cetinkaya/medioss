<!--
  @component
  Settings "Your data" panel (FR-069, PB-12, PB-16).

  - Exports are real same-origin `<a href download>` links styled as buttons:
    JSON (all data) and CSV (mood entries only).
  - Import is a button that opens a FormDialog with the file field; it
    sends the chosen JSON or CSV file once per activation. Success closes the
    dialog and shows all five counts the server returns in the panel; a
    refusal stays in the dialog with the specific reason (never server text)
    and no report.
-->
<script lang="ts">
  import { failureMessage, reasonOf } from "$lib/api/failure";
  import {
    exportCsvUrl,
    exportJsonUrl,
    importData,
    type ImportReport,
  } from "$lib/api/portability";
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { dataMessages as m } from "./settings.messages";

  const id = $props.id();

  let file = $state<File | null>(null);
  let importOpen = $state(false);
  let report = $state<ImportReport | null>(null);

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

  function askImport() {
    file = null;
    importOpen = true;
  }

  async function runImport() {
    if (!file) return;
    report = null;
    report = await importData(file);
    announce(t(m.imported));
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
      <Button variant="secondary" onclick={askImport}>
        <AppIcon name="upload-simple" size={18} />
        {t(m.importAction)}
      </Button>
    </div>

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

<FormDialog
  bind:open={importOpen}
  title={t(m.importLabel)}
  submitLabel={t(m.importAction)}
  canSubmit={file !== null}
  onSubmit={runImport}
  errorMessage={(error) =>
    failureMessage(reasonOf(error), m.importFailed).message}
>
  <label for="{id}-file" class="sr-only">{t(m.importLabel)}</label>
  <Input
    id="{id}-file"
    type="file"
    accept=".json,.csv,application/json,text/csv"
    onchange={(event) => {
      file = event.currentTarget.files?.[0] ?? null;
    }}
  />
</FormDialog>
