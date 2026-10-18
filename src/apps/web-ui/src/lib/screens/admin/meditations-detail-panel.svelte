<!-- @component Selected program's detail: summary, actions and the reorderable member list. -->
<script lang="ts">
  import {
    meditationDownloadUrl,
    programCoverUrl,
    type Category,
    type Meditation,
    type Program,
  } from "$lib/api/meditation";
  import { type Failure } from "$lib/components/dialogs/form-dialog.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { formatDuration } from "$lib/i18n/formatting/duration-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { playerHref } from "$lib/navigation/routes";
  import { meditationAdminMessages as m } from "./admin.messages";

  let {
    id,
    selected,
    categories,
    members,
    membersFailure,
    reordering,
    reorderFailure,
    memberFailure,
    coverVersion,
    headingElement = $bindable(null),
    onBack,
    onEdit,
    onUpload,
    onAddMeditation,
    onRetryMembers,
    onMove,
    onEditDescription,
    onRemove,
  }: {
    id: string;
    selected: Program | null;
    categories: Category[];
    members: Meditation[] | null;
    membersFailure: boolean;
    reordering: boolean;
    reorderFailure: Failure | null;
    memberFailure: Failure | null;
    coverVersion: (programId: number) => number;
    headingElement: HTMLElement | null;
    onBack: () => void;
    onEdit: (program: Program) => void;
    onUpload: (program: Program) => void;
    onAddMeditation: (program: Program) => void;
    onRetryMembers: () => void;
    onMove: (index: number, delta: -1 | 1) => void;
    onEditDescription: (meditation: Meditation) => void;
    onRemove: (meditation: Meditation) => void;
  } = $props();
</script>

{#snippet failureMessage(failure: Failure | null)}
  {#if failure}
    <InlineMessage tone="error" role="alert" class="mb-3">
      {t(failure.message, failure.params)}
    </InlineMessage>
  {/if}
{/snippet}

<Panel aria-labelledby="{id}-detail-heading">
  {#snippet heading()}
    {#if selected}
      <Button variant="ghost" size="sm" onclick={onBack}>
        <AppIcon name="arrow-left" size={16} />
        {t(m.backToPrograms)}
      </Button>
    {/if}
    <h2
      id="{id}-detail-heading"
      bind:this={headingElement}
      tabindex="-1"
      class="mt-1 text-heading text-ink-strong outline-none"
    >
      {selected
        ? t(m.meditationsOf, { title: selected.title })
        : t(m.libraryHeading)}
    </h2>
  {/snippet}
  {#if selected}
    <div class="mb-4 flex flex-wrap gap-2">
      <Button variant="secondary" onclick={() => onEdit(selected)}>
        <AppIcon name="pencil-simple" size={20} />
        {t(m.editProgram)}
      </Button>
      <Button variant="secondary" onclick={() => onUpload(selected)}>
        <AppIcon name="upload-simple" size={20} />
        {t(m.uploadItem)}
      </Button>
      <Button variant="secondary" onclick={() => onAddMeditation(selected)}>
        <AppIcon name="plus" size={20} />
        {t(m.addMeditationItem)}
      </Button>
    </div>
    <div class="mb-4 flex flex-wrap gap-4 rounded-tile bg-surface-solid/50 p-4">
      {#if selected.has_cover}
        <img
          src={programCoverUrl(selected.id, coverVersion(selected.id))}
          alt=""
          class="aspect-video w-40 rounded-tile object-cover"
        />
      {/if}
      <div class="min-w-0 flex-1">
        <p class="font-medium text-ink-strong">{selected.title}</p>
        <p class="mt-1 text-sm text-ink-muted">
          {selected.description || t(m.noProgramDescription)}
        </p>
        <p class="mt-2 text-sm text-ink-muted">
          {t(m.programCategory, {
            category:
              categories.find(
                (category) => category.id === selected.category_id,
              )?.name ?? t(m.uncategorized),
          })}
        </p>
      </div>
    </div>
    {@render failureMessage(reorderFailure)}
    {@render failureMessage(memberFailure)}
    <RegionState
      status={membersFailure
        ? "error"
        : !members
          ? "loading"
          : members.length
            ? "ready"
            : "empty"}
      minHeight="8rem"
      errorMessage={t(m.membersLoadFailed)}
      onRetry={onRetryMembers}
      emptyTitle={t(m.noMembers)}
      emptyText={t(m.noMembersText)}
    >
      <ol class="grid grid-cols-1 gap-3">
        {#each members ?? [] as meditation, index (meditation.id)}
          <li
            id="{id}-row-{meditation.id}"
            class="flex flex-wrap items-center gap-2 rounded-tile bg-surface-solid/50 p-2 ps-4"
          >
            <div class="grid min-w-0 flex-[1_1_12rem] grid-cols-1 gap-0.5">
              <span class="font-medium break-words text-ink-strong">
                {meditation.name}
              </span>
              <span class="text-sm text-ink-muted">
                {formatDuration(
                  meditation.duration_ms,
                  locale.current,
                  "short",
                )}
                · {t(m.positionOf, {
                  position: index + 1,
                  total: members?.length ?? 0,
                })}
              </span>
              <span class="text-sm break-words text-ink-body">
                {meditation.description || t(m.noMeditationDescription)}
              </span>
            </div>
            <div class="flex items-center">
              <Button
                variant="ghost"
                size="icon-sm"
                data-move="up"
                disabled={index === 0 || reordering}
                aria-label={t(m.moveUp, { name: meditation.name })}
                onclick={() => onMove(index, -1)}
              >
                <AppIcon name="arrow-up" size={20} />
              </Button>
              <Button
                variant="ghost"
                size="icon-sm"
                data-move="down"
                disabled={index === (members?.length ?? 0) - 1 || reordering}
                aria-label={t(m.moveDown, { name: meditation.name })}
                onclick={() => onMove(index, 1)}
              >
                <AppIcon name="arrow-down" size={20} />
              </Button>
              <OverflowMenu
                label={t(m.meditationActions, {
                  name: meditation.name,
                })}
              >
                <OverflowItem
                  label={t(actionMessages.play)}
                  icon="play"
                  onclick={() =>
                    (window.location.href = playerHref(meditation.id))}
                />
                <OverflowItem
                  label={t(actionMessages.edit)}
                  icon="pencil-simple"
                  onclick={() => onEditDescription(meditation)}
                />
                <OverflowItem
                  label={t(actionMessages.download)}
                  icon="download-simple"
                  onclick={() =>
                    (window.location.href = meditationDownloadUrl(
                      meditation.id,
                    ))}
                />
                <OverflowItem
                  label={t(m.removeFromProgram)}
                  icon="trash"
                  destructive
                  onclick={() => onRemove(meditation)}
                />
              </OverflowMenu>
            </div>
          </li>
        {/each}
      </ol>
    </RegionState>
  {:else}
    <p class="text-sm text-ink-muted">{t(m.pickProgram)}</p>
  {/if}
</Panel>
