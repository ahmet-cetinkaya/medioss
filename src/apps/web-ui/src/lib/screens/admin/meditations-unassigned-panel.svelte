<!-- @component Meditations that belong to no program, with play / edit / assign / delete actions. -->
<script lang="ts">
  import type { Meditation } from "$lib/api/meditation";
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
    meditations,
    onAssign,
    onEditDescription,
    onRename,
    onDelete,
  }: {
    id: string;
    meditations: Meditation[];
    onAssign: (meditation: Meditation) => void;
    onEditDescription: (meditation: Meditation) => void;
    onRename: (meditation: Meditation) => void;
    onDelete: (meditation: Meditation) => void;
  } = $props();
</script>

<Panel aria-labelledby="{id}-unassigned-heading">
  {#snippet heading()}
    <h2 id="{id}-unassigned-heading" class="text-heading text-ink-strong">
      {t(m.unassignedHeading)}
    </h2>
  {/snippet}
  <ul class="grid grid-cols-1 gap-3">
    {#each meditations as meditation (meditation.id)}
      <li
        class="flex flex-wrap items-center gap-2 rounded-tile bg-surface-solid/50 p-2 ps-4"
      >
        <div class="grid min-w-0 flex-[1_1_12rem] grid-cols-1 gap-0.5">
          <span class="font-medium break-words text-ink-strong">
            {meditation.name}
          </span>
          <span class="text-sm text-ink-muted">
            {formatDuration(meditation.duration_ms, locale.current, "short")}
          </span>
        </div>
        <OverflowMenu label={t(m.meditationActions, { name: meditation.name })}>
          <OverflowItem
            label={t(actionMessages.play)}
            icon="play"
            onclick={() => (window.location.href = playerHref(meditation.id))}
          />
          <OverflowItem
            label={t(m.assignToProgram)}
            icon="plus"
            onclick={() => onAssign(meditation)}
          />
          <OverflowItem
            label={t(m.renameMeditationItem)}
            icon="pencil-simple"
            onclick={() => onRename(meditation)}
          />
          <OverflowItem
            label={t(m.editDescriptionTitle)}
            icon="pencil-simple"
            onclick={() => onEditDescription(meditation)}
          />
          <OverflowItem
            label={t(m.deleteMeditationItem)}
            icon="trash"
            destructive
            onclick={() => onDelete(meditation)}
          />
        </OverflowMenu>
      </li>
    {/each}
  </ul>
</Panel>
