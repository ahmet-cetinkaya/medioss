<!-- @component Programs panel: category-grouped program cards with an overflow menu. -->
<script lang="ts">
  import { programCoverUrl, type Program } from "$lib/api/meditation";
  import ContentCard from "$lib/components/content/content-card.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import type { groupPrograms } from "$lib/screens/library/library-groups";
  import { meditationAdminMessages as m } from "./admin.messages";

  let {
    id,
    groups,
    hasVisible,
    hasAny,
    selectedId,
    coverVersion,
    headingElement = $bindable(null),
    onAdd,
    onSelect,
    onEdit,
    onAddMeditation,
    onUpload,
  }: {
    id: string;
    groups: ReturnType<typeof groupPrograms>;
    hasVisible: boolean;
    hasAny: boolean;
    selectedId: number | null;
    coverVersion: (programId: number) => number;
    headingElement: HTMLElement | null;
    onAdd: () => void;
    onSelect: (program: Program) => void;
    onEdit: (program: Program) => void;
    onAddMeditation: (program: Program) => void;
    onUpload: (program: Program) => void;
  } = $props();
</script>

<Panel aria-labelledby="{id}-programs-heading">
  {#snippet heading()}
    <div class="flex flex-wrap items-center justify-between gap-3">
      <h2
        id="{id}-programs-heading"
        bind:this={headingElement}
        tabindex="-1"
        class="text-heading text-ink-strong outline-none"
      >
        {t(m.programsHeading)}
      </h2>
      <Button variant="secondary" onclick={onAdd}>
        <AppIcon name="plus" size={20} />
        {t(m.addProgram)}
      </Button>
    </div>
  {/snippet}
  {#if hasVisible}
    <!-- Library-sized cards (w-44/sm:w-48, 3:4 artwork) grouped by
         category like the Library screen; the overflow menu sits at
         the card's top-end corner. -->
    <div class="flex flex-col gap-6">
      {#each groups as group (group.key)}
        <section
          aria-labelledby="{id}-group-{group.key}"
          class="flex flex-col gap-3"
        >
          <h3 id="{id}-group-{group.key}" class="font-medium text-ink-strong">
            {group.category?.name ?? t(m.uncategorized)}
          </h3>
          <ul class="flex flex-wrap gap-4">
            {#each group.programs as program (program.id)}
              <li class="relative flex w-44 flex-col sm:w-48">
                <ContentCard
                  aspect="3/4"
                  headingLevel={4}
                  ariaLabel={t(m.selectProgram, { title: program.title })}
                  current={program.id === selectedId}
                  onselect={() => onSelect(program)}
                  title={program.title}
                  meta={[program.description || t(m.noProgramDescription)]}
                  coverUrl={program.has_cover
                    ? programCoverUrl(program.id, coverVersion(program.id))
                    : null}
                  artworkSeed={program.id}
                  artworkCategory="meditation"
                />
                <div class="absolute end-0 top-0">
                  <OverflowMenu
                    label={t(m.programActions, {
                      title: program.title,
                    })}
                  >
                    <OverflowItem
                      label={t(m.editProgram)}
                      icon="pencil-simple"
                      onclick={() => onEdit(program)}
                    />
                    <OverflowItem
                      label={t(m.addMeditationItem)}
                      icon="plus"
                      onclick={() => onAddMeditation(program)}
                    />
                    <OverflowItem
                      label={t(m.uploadItem)}
                      icon="upload-simple"
                      onclick={() => onUpload(program)}
                    />
                  </OverflowMenu>
                </div>
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  {:else}
    <p class="text-sm text-ink-muted">
      {t(hasAny ? m.programsFilteredEmpty : m.noPrograms)}
    </p>
  {/if}
</Panel>
