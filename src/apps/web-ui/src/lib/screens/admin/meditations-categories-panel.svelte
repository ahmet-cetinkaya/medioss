<!-- @component Categories panel: filter chips with a rename/delete overflow menu. -->
<script lang="ts">
  import type { Category } from "$lib/api/meditation";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import { meditationAdminMessages as m } from "./admin.messages";
  import type { CategoryFilter } from "./meditations-logic";

  let {
    id,
    categories,
    filter = $bindable(),
    countIn,
    headingElement = $bindable(null),
    onAdd,
    onRename,
    onDelete,
  }: {
    id: string;
    categories: Category[];
    filter: CategoryFilter;
    countIn: (categoryId: number | null) => number;
    headingElement: HTMLElement | null;
    onAdd: () => void;
    onRename: (category: Category) => void;
    onDelete: (category: Category) => void;
  } = $props();
</script>

<Panel aria-labelledby="{id}-category-heading">
  {#snippet heading()}
    <div class="flex items-center justify-between gap-3">
      <h2
        id="{id}-category-heading"
        bind:this={headingElement}
        tabindex="-1"
        class="text-heading text-ink-strong"
      >
        {t(m.categoriesPanelHeading)}
      </h2>
      <Button variant="secondary" onclick={onAdd}>
        <AppIcon name="plus" size={20} />{t(m.addCategory)}
      </Button>
    </div>
  {/snippet}
  {#if categories.length}
    <ul class="grid grid-cols-1 gap-2">
      {#each categories as category (category.id)}
        <li
          class="flex items-center gap-1 rounded-tile ps-1 pe-1 {filter ===
          category.id
            ? 'bg-accent-fill/15'
            : 'bg-surface-solid/50'}"
        >
          <button
            type="button"
            class="focus-ring min-h-11 min-w-0 flex-1 rounded-tile px-3 py-2 text-start font-medium break-words {filter ===
            category.id
              ? 'text-accent-strong'
              : 'text-ink-strong'}"
            aria-pressed={filter === category.id}
            onclick={() =>
              (filter = filter === category.id ? "all" : category.id)}
          >
            {t(m.categoryFilterNamed, {
              name: category.name,
              count: countIn(category.id),
            })}
          </button>
          <OverflowMenu label={t(m.categoryActions, { name: category.name })}>
            <OverflowItem
              label={t(m.renameCategory)}
              icon="pencil-simple"
              onclick={() => onRename(category)}
            />
            <OverflowItem
              label={t(m.deleteCategoryItem)}
              icon="trash"
              destructive
              onclick={() => onDelete(category)}
            />
          </OverflowMenu>
        </li>
      {/each}
      <li
        class="flex items-center gap-1 rounded-tile ps-1 pe-1 {filter === 'none'
          ? 'bg-accent-fill/15'
          : 'bg-surface-solid/50'}"
      >
        <button
          type="button"
          class="focus-ring min-h-11 min-w-0 flex-1 rounded-tile px-3 py-2 text-start font-medium break-words {filter ===
          'none'
            ? 'text-accent-strong'
            : 'text-ink-strong'}"
          aria-pressed={filter === "none"}
          onclick={() => (filter = filter === "none" ? "all" : "none")}
        >
          {t(m.categoryFilterNamed, {
            name: t(m.uncategorized),
            count: countIn(null),
          })}
        </button>
      </li>
    </ul>
  {:else}
    <p class="text-sm text-ink-muted">{t(m.noCategories)}</p>
  {/if}
</Panel>
