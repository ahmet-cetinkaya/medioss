<!-- @component Sound categories panel: select a category, rename/delete from its overflow menu, add. -->
<script lang="ts">
  import type { SoundscapeCategory } from "$lib/api/soundscape";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import { soundAdminMessages as m } from "./admin.messages";

  let {
    id,
    categories,
    selectedId,
    countIn,
    headingElement = $bindable(null),
    onSelect,
    onAdd,
    onRename,
    onDelete,
  }: {
    id: string;
    categories: SoundscapeCategory[];
    selectedId: number | undefined;
    countIn: (categoryId: number) => number;
    headingElement: HTMLElement | null;
    onSelect: (categoryId: number) => void;
    onAdd: () => void;
    onRename: (category: SoundscapeCategory) => void;
    onDelete: (category: SoundscapeCategory) => void;
  } = $props();
</script>

<Panel class="lg:col-span-2" aria-labelledby="{id}-categories-heading">
  {#snippet heading()}
    <h2
      id="{id}-categories-heading"
      bind:this={headingElement}
      tabindex="-1"
      class="text-heading text-ink-strong outline-none"
    >
      {t(m.categoriesHeading)}
    </h2>
  {/snippet}
  {#if categories.length}
    <ul class="grid grid-cols-1 gap-2">
      {#each categories as category (category.id)}
        <li
          class="flex items-center gap-1 rounded-tile ps-1 pe-1 {category.id ===
          selectedId
            ? 'bg-panel-tint'
            : 'bg-surface-solid/50'}"
        >
          <button
            type="button"
            aria-label={t(m.selectCategory, { name: category.name })}
            aria-current={category.id === selectedId ? "true" : undefined}
            class="focus-ring min-h-11 min-w-0 flex-1 rounded-tile px-3 py-2 text-start font-medium break-words text-ink-strong"
            onclick={() => onSelect(category.id)}
          >
            {t(m.categoryCount, {
              name: category.name,
              count: countIn(category.id),
            })}
          </button>
          <OverflowMenu label={t(m.categoryActions, { name: category.name })}>
            <OverflowItem
              label={t(actionMessages.rename)}
              icon="pencil-simple"
              onclick={() => onRename(category)}
            />
            <OverflowItem
              label={t(actionMessages.delete)}
              icon="trash"
              destructive
              onclick={() => onDelete(category)}
            />
          </OverflowMenu>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="text-sm text-ink-muted">{t(m.noCategories)}</p>
  {/if}
  <div class="mt-4">
    <Button variant="secondary" onclick={onAdd}>
      <AppIcon name="plus" size={20} />
      {t(m.addCategory)}
    </Button>
  </div>
</Panel>
