<!--
  @component
  Admin Meditations section (PB-13, PB-16): a drill-down hierarchy. Left:
  categories (filter chips + add) and programs (add; each row selects it and
  has an overflow menu: set category, add existing meditation, upload audio).
  Right: the selected program's meditations with move up/down reordering
  (optimistic, reverted on failure) and per-meditation actions (play, edit
  description, download, remove from program). State is client-side only; on
  small screens the two panes stack and "Back to programs" returns to the list.
-->
<script lang="ts">
  import { onMount, tick } from "svelte";
  import { ApiError } from "$lib/api/api-error";
  import {
    addMember,
    assignCategory,
    createCategory,
    createProgram,
    deleteCategory,
    fetchCategories,
    fetchLibrary,
    fetchProgramMembers,
    fetchPrograms,
    meditationDownloadUrl,
    programCoverUrl,
    removeMember,
    renameCategory,
    reorderProgramMeditations,
    replaceProgramCover,
    titleCaseFromFile,
    updateMeditationDescription,
    updateProgram,
    uploadAudio,
    type Category,
    type Meditation,
    type Program,
  } from "$lib/api/meditation";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import FormDialog, {
    describeFailure,
    type Failure,
  } from "$lib/components/dialogs/form-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { formatDuration } from "$lib/i18n/formatting/duration-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
  import { groupPrograms } from "$lib/screens/library/library-groups";
  import { playerHref } from "$lib/navigation/routes";
  import {
    adminMessages,
    meditationAdminMessages as m,
  } from "./admin.messages";
  import NativeSelect from "./native-select.svelte";

  type Data = {
    programs: Program[];
    categories: Category[];
    library: Meditation[];
  };

  const id = $props.id();

  // `/admin/meditations/` → null; `/admin/meditations/12/` → 12.
  function parseProgramId(pathname: string): number | null {
    const match = pathname.match(/^\/admin\/meditations\/(\d+)\/?$/);
    return match ? Number(match[1]) : null;
  }
  const DESCRIPTION_MAX = 2000;
  const descriptionLength = (text: string) => [...text].length;
  const say = (
    message: Parameters<typeof t>[0],
    params?: Parameters<typeof t>[1],
  ) => announce(t(message, params));

  let data = $state<Data | null>(null);
  let loadFailure = $state<ServerReason | null>(null);
  let request = 0;

  const status = $derived(loadFailure ? "error" : data ? "ready" : "loading");
  const loadErrorText = $derived(
    loadFailure === "network" || loadFailure === "server"
      ? t(serverReasonMessages[loadFailure])
      : t(adminMessages.loadFailed),
  );

  async function load(reset = true) {
    const current = ++request;
    if (reset) {
      data = null;
      loadFailure = null;
    }
    try {
      const [programs, categories, library] = await Promise.all([
        fetchPrograms(),
        fetchCategories(),
        fetchLibrary(),
      ]);
      if (current !== request) return;
      data = { programs, categories, library };
      loadFailure = null;
    } catch (error) {
      if (current === request)
        loadFailure =
          error instanceof ApiError ? serverReason(error).reason : "generic";
    }
  }

  onMount(() => void load());

  // Selection: category filter ("all" | "none" | category id) and program.
  // The selected program lives in the address (/admin/meditations/<id>/),
  // so reload, Back and bookmarks land on the same view.
  let filter = $state<"all" | "none" | number>("all");
  let selectedId = $state<number | null>(parseProgramId(location.pathname));
  const selected = $derived(
    data?.programs.find((p) => p.id === selectedId) ?? null,
  );
  const visiblePrograms = $derived(
    (data?.programs ?? []).filter((p) =>
      filter === "all"
        ? true
        : filter === "none"
          ? p.category_id === null
          : p.category_id === filter,
    ),
  );
  // Library-style grouping: the server's category order, Uncategorized last.
  const groupedPrograms = $derived(
    groupPrograms(visiblePrograms, data?.categories ?? []),
  );
  // Meditations that belong to no program (library rows without program_id).
  const unassigned = $derived(
    (data?.library ?? []).filter((x) => x.program_id == null),
  );

  const countIn = (categoryId: number | null) =>
    (data?.programs ?? []).filter((p) => p.category_id === categoryId).length;

  // Members of the selected program.
  let members = $state<Meditation[] | null>(null);
  let membersFailure = $state(false);
  let membersRequest = 0;
  let detailHeading = $state<HTMLElement | null>(null);
  let programsHeading = $state<HTMLElement | null>(null);
  let categoriesHeading = $state<HTMLElement | null>(null);

  async function loadMembers(reset = true) {
    const programId = selectedId;
    if (programId === null) return;
    const current = ++membersRequest;
    if (reset) members = null;
    membersFailure = false;
    try {
      const list = await fetchProgramMembers(programId);
      if (current === membersRequest) members = list;
    } catch {
      if (current === membersRequest) membersFailure = true;
    }
  }

  async function selectProgram(program: Program) {
    selectedId = program.id;
    reorderFailure = null;
    memberFailure = null;
    history.pushState(history.state, "", `/admin/meditations/${program.id}/`);
    void loadMembers();
    await tick();
    detailHeading?.focus();
  }

  async function backToPrograms() {
    selectedId = null;
    members = null;
    membersRequest++;
    history.pushState(history.state, "", "/admin/meditations/");
    await tick();
    programsHeading?.focus();
  }

  // Browser Back/Forward between the list and a program detail.
  $effect(() => {
    const onPop = () => {
      const next = parseProgramId(location.pathname);
      if (next === selectedId) return;
      selectedId = next;
      reorderFailure = null;
      memberFailure = null;
      members = null;
      if (next !== null) void loadMembers();
    };
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  });

  // Reorder (optimistic; reverted on failure).
  let reordering = $state(false);
  let reorderFailure = $state<Failure | null>(null);

  async function move(index: number, delta: -1 | 1) {
    const programId = selectedId;
    const prev = members;
    const target = index + delta;
    if (programId === null || !prev || reordering) return;
    if (target < 0 || target >= prev.length) return;
    const next = [...prev];
    [next[index], next[target]] = [next[target], next[index]];
    const moved = next[target];
    members = next;
    reordering = true;
    reorderFailure = null;
    try {
      await reorderProgramMeditations(
        programId,
        next.map((x) => x.id),
      );
      say(m.moved, { name: moved.name, position: target + 1 });
    } catch (error) {
      if (selectedId === programId) members = prev;
      reorderFailure = describeFailure(error);
    } finally {
      reordering = false;
    }
    // Keep keyboard focus on the moved row's button (DOM moves can drop it).
    await tick();
    const dir = delta < 0 ? "up" : "down";
    const other = delta < 0 ? "down" : "up";
    const row = document.getElementById(`${id}-row-${moved.id}`);
    const button =
      row?.querySelector<HTMLButtonElement>(
        `[data-move="${dir}"]:not(:disabled)`,
      ) ??
      row?.querySelector<HTMLButtonElement>(
        `[data-move="${other}"]:not(:disabled)`,
      );
    button?.focus();
  }

  // Add category (dialog).
  let categoryOpen = $state(false);
  let categoryName = $state("");

  function askAddCategory() {
    categoryName = "";
    categoryOpen = true;
  }

  async function runAddCategory() {
    const category = await createCategory(categoryName.trim());
    say(m.categoryCreated, { name: category.name });
    await load(false);
  }

  // Rename category (dialog).
  let renameOpen = $state(false);
  let renameTarget = $state<Category | null>(null);
  let renameName = $state("");

  function askRenameCategory(category: Category) {
    renameTarget = category;
    renameName = category.name;
    renameOpen = true;
  }

  async function runRenameCategory() {
    const category = renameTarget;
    if (!category || renameName.trim() === "") return;
    await renameCategory(category.id, renameName.trim());
    say(m.categoryRenamed, { name: renameName.trim() });
    await load(false);
  }

  // Delete category (confirm dialog); its programs become uncategorized.
  let deleteCategoryOpen = $state(false);
  let deleteCategoryTarget = $state<Category | null>(null);

  function askDeleteCategory(category: Category) {
    deleteCategoryTarget = category;
    deleteCategoryOpen = true;
  }

  async function runDeleteCategory() {
    const category = deleteCategoryTarget;
    if (!category) return;
    if (filter === category.id) filter = "all";
    await deleteCategory(category.id);
    say(m.meditationCategoryDeleted, { name: category.name });
    deleteCategoryTarget = null;
    await load(false);
  }

  // Add program (dialog).
  let programOpen = $state(false);
  let programTitle = $state("");
  let programDescription = $state("");
  let cover = $state<File | null>(null);

  function askAddProgram() {
    programTitle = "";
    programDescription = "";
    cover = null;
    programOpen = true;
  }

  async function runAddProgram() {
    const program = await createProgram(
      programTitle.trim(),
      programDescription.trim() || null,
      cover,
    );
    say(m.programCreated, { title: program.title });
    await load(false);
  }

  // Edit program (dialog): title, description, category, cover.
  let editProgramOpen = $state(false);
  let editProgramTarget = $state<Program | null>(null);
  let editProgramTitle = $state("");
  let editProgramDescription = $state("");
  let editProgramCategory = $state("");
  let editProgramCover = $state<File | null>(null);
  let coverVersions = $state<Record<number, number>>({});
  const coverVersion = (programId: number) => coverVersions[programId] ?? 0;

  function askEditProgram(program: Program) {
    editProgramTarget = program;
    editProgramTitle = program.title;
    editProgramDescription = program.description ?? "";
    editProgramCategory =
      program.category_id === null ? "" : String(program.category_id);
    editProgramCover = null;
    editProgramOpen = true;
  }

  async function runEditProgram() {
    const program = editProgramTarget;
    if (!program || editProgramTitle.trim() === "") return;
    await updateProgram(
      program.id,
      editProgramTitle.trim(),
      editProgramDescription.trim() || null,
    );
    const categoryId =
      editProgramCategory === "" ? null : Number(editProgramCategory);
    if (categoryId !== program.category_id)
      await assignCategory(program.id, categoryId);
    if (editProgramCover) {
      await replaceProgramCover(program.id, editProgramCover);
      coverVersions[program.id] = (coverVersions[program.id] ?? 0) + 1;
    }
    say(m.programSaved, { title: editProgramTitle.trim() });
    await load(false);
    if (program.id === selectedId) await loadMembers(false);
  }

  // Program actions: add existing meditation, upload. Category changes go
  // through Edit program.
  let programTarget = $state<Program | null>(null);

  let addOpen = $state(false);
  let addPick = $state("");
  let addCandidates = $state<Meditation[]>([]);

  async function askAddMeditation(program: Program) {
    programTarget = program;
    addPick = "";
    const inProgram =
      program.id === selectedId && members
        ? members
        : await fetchProgramMembers(program.id).catch(() => []);
    const taken = new Set(inProgram.map((x) => x.id));
    addCandidates = (data?.library ?? []).filter((x) => !taken.has(x.id));
    addOpen = true;
  }

  async function runAddMeditation() {
    const program = programTarget;
    const meditation = addCandidates.find((x) => x.id === Number(addPick));
    if (!program || !meditation) return;
    await addMember(program.id, meditation.id);
    say(m.addedToProgram, { name: meditation.name, title: program.title });
    if (program.id === selectedId) await loadMembers(false);
  }

  // Assign an unassigned meditation to a program (dialog; program chosen).
  let assignOpen = $state(false);
  let assignTarget = $state<Meditation | null>(null);
  let assignPick = $state("");

  function askAssign(meditation: Meditation) {
    assignTarget = meditation;
    assignPick = "";
    assignOpen = true;
  }

  async function runAssign() {
    const meditation = assignTarget;
    const program = (data?.programs ?? []).find(
      (p) => p.id === Number(assignPick),
    );
    if (!meditation || !program) return;
    await addMember(program.id, meditation.id);
    say(m.addedToProgram, { name: meditation.name, title: program.title });
    await load(false);
  }

  // Upload meditation audio (dialog; program prefilled).
  let uploadOpen = $state(false);
  let uploadProgram = $state("");
  let audioFile = $state<File | null>(null);
  let meditationName = $state("");
  let uploadDescription = $state("");
  const uploadTooLong = $derived(
    descriptionLength(uploadDescription.trim()) > DESCRIPTION_MAX,
  );

  function askUpload(program: Program | null) {
    uploadProgram = program ? String(program.id) : "";
    audioFile = null;
    meditationName = "";
    uploadDescription = "";
    uploadOpen = true;
  }

  function chooseAudio(input: HTMLInputElement) {
    audioFile = input.files?.[0] ?? null;
    // Title Case prefill from the file name, still editable (FR-024).
    if (audioFile && !meditationName.trim())
      meditationName = titleCaseFromFile(audioFile.name);
  }

  async function runUpload() {
    const programId = Number(uploadProgram);
    if (!audioFile || !programId) return;
    const meditation = await uploadAudio(
      audioFile,
      meditationName.trim(),
      programId,
      uploadDescription.trim() || null,
    );
    say(m.uploaded, { name: meditation.name });
    await load(false);
    if (programId === selectedId) await loadMembers(false);
  }

  // Edit description.
  let editOpen = $state(false);
  let editTarget = $state<Meditation | null>(null);
  let editDescription = $state("");
  const editTooLong = $derived(
    descriptionLength(editDescription.trim()) > DESCRIPTION_MAX,
  );

  function openEdit(meditation: Meditation) {
    editTarget = meditation;
    editDescription = meditation.description ?? "";
    editOpen = true;
  }

  async function runEdit() {
    if (!editTarget || editTooLong) return;
    const updated = await updateMeditationDescription(
      editTarget.id,
      editDescription.trim() || null,
    );
    say(m.descriptionUpdated, { name: updated.name });
    await loadMembers(false);
  }

  // Remove from program.
  let removeOpen = $state(false);
  let memberFailure = $state<Failure | null>(null);

  function askRemove(meditation: Meditation) {
    editTarget = meditation;
    removeOpen = true;
  }

  async function runRemove() {
    const target = editTarget;
    if (!target || selectedId === null) return;
    await removeMember(selectedId, target.id);
    say(m.removed, { name: target.name });
    await loadMembers(false);
  }
</script>

{#snippet failureMessage(failure: Failure | null)}
  {#if failure}
    <InlineMessage tone="error" role="alert" class="mb-3">
      {t(failure.message, failure.params)}
    </InlineMessage>
  {/if}
{/snippet}

{#snippet counter(counterId: string, text: string, over: boolean)}
  <p
    id={counterId}
    class={over ? "text-sm text-destructive" : "text-sm text-ink-muted"}
  >
    {t(m.descriptionCount, {
      count: descriptionLength(text.trim()),
      max: DESCRIPTION_MAX,
    })}
    {#if over}<span role="alert"
        >{t(m.descriptionTooLong, { max: DESCRIPTION_MAX })}</span
      >{/if}
  </p>
{/snippet}

<RegionState
  {status}
  minHeight="24rem"
  errorMessage={loadErrorText}
  onRetry={() => void load()}
>
  {#if data}
    <div
      class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)] lg:items-start"
    >
      <div class={selected ? "hidden" : "block"}>
        <Panel aria-labelledby="{id}-programs-heading">
          {#snippet heading()}
            <div class="flex flex-wrap items-center justify-between gap-3">
              <h2
                id="{id}-programs-heading"
                bind:this={programsHeading}
                tabindex="-1"
                class="text-heading text-ink-strong outline-none"
              >
                {t(m.programsHeading)}
              </h2>
              <Button variant="secondary" onclick={askAddProgram}>
                <AppIcon name="plus" size={20} />
                {t(m.addProgram)}
              </Button>
            </div>
          {/snippet}
          {#if visiblePrograms.length}
            <!-- Library-sized cards (w-44/sm:w-48, 3:4 artwork) grouped by
                 category like the Library screen; the overflow menu sits at
                 the card's top-end corner. -->
            <div class="flex flex-col gap-6">
              {#each groupedPrograms as group (group.key)}
                <section
                  aria-labelledby="{id}-group-{group.key}"
                  class="flex flex-col gap-3"
                >
                  <h3
                    id="{id}-group-{group.key}"
                    class="font-medium text-ink-strong"
                  >
                    {group.category?.name ?? t(m.uncategorized)}
                  </h3>
                  <ul class="flex flex-wrap gap-4">
                    {#each group.programs as program (program.id)}
                      <li class="relative flex w-44 flex-col sm:w-48">
                        <button
                          type="button"
                          aria-label={t(m.selectProgram, {
                            title: program.title,
                          })}
                          aria-current={program.id === selectedId
                            ? "true"
                            : undefined}
                          class="focus-ring flex min-w-0 flex-col rounded-tile text-start"
                          onclick={() => void selectProgram(program)}
                        >
                          <span
                            class="relative block aspect-3/4 w-full overflow-hidden rounded-tile bg-panel-tint"
                          >
                            {#if program.has_cover}
                              <img
                                src={programCoverUrl(
                                  program.id,
                                  coverVersion(program.id),
                                )}
                                alt=""
                                loading="lazy"
                                class="absolute inset-0 size-full object-cover"
                              />
                            {/if}
                          </span>
                          <span class="grid gap-1 p-1 pt-2">
                            <span
                              class="line-clamp-2 text-sm font-medium break-words text-ink-strong"
                            >
                              {program.title}
                            </span>
                            <span class="line-clamp-2 text-sm text-ink-body">
                              {program.description || t(m.noProgramDescription)}
                            </span>
                          </span>
                        </button>
                        <div class="absolute end-0 top-0">
                          <OverflowMenu
                            label={t(m.programActions, {
                              title: program.title,
                            })}
                          >
                            <OverflowItem
                              label={t(m.editProgram)}
                              icon="pencil-simple"
                              onclick={() => askEditProgram(program)}
                            />
                            <OverflowItem
                              label={t(m.addMeditationItem)}
                              icon="plus"
                              onclick={() => void askAddMeditation(program)}
                            />
                            <OverflowItem
                              label={t(m.uploadItem)}
                              icon="upload-simple"
                              onclick={() => askUpload(program)}
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
              {t(data.programs.length ? m.programsFilteredEmpty : m.noPrograms)}
            </p>
          {/if}
        </Panel>
      </div>

      <div class={selected ? "hidden" : "block"}>
        <Panel aria-labelledby="{id}-category-heading">
          {#snippet heading()}
            <div class="flex items-center justify-between gap-3">
              <h2
                id="{id}-category-heading"
                bind:this={categoriesHeading}
                tabindex="-1"
                class="text-heading text-ink-strong"
              >
                {t(m.categoriesPanelHeading)}
              </h2>
              <Button variant="secondary" onclick={askAddCategory}>
                <AppIcon name="plus" size={20} />{t(m.addCategory)}
              </Button>
            </div>
          {/snippet}
          {#if data.categories.length}
            <ul class="grid grid-cols-1 gap-2">
              {#each data.categories as category (category.id)}
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
                  <OverflowMenu
                    label={t(m.categoryActions, { name: category.name })}
                  >
                    <OverflowItem
                      label={t(m.renameCategory)}
                      icon="pencil-simple"
                      onclick={() => askRenameCategory(category)}
                    />
                    <OverflowItem
                      label={t(m.deleteCategoryItem)}
                      icon="trash"
                      destructive
                      onclick={() => askDeleteCategory(category)}
                    />
                  </OverflowMenu>
                </li>
              {/each}
              <li
                class="flex items-center gap-1 rounded-tile ps-1 pe-1 {filter ===
                'none'
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
      </div>

      {#if !selected && unassigned.length}
        <div class="lg:col-span-2">
          <Panel aria-labelledby="{id}-unassigned-heading">
            {#snippet heading()}
              <h2
                id="{id}-unassigned-heading"
                class="text-heading text-ink-strong"
              >
                {t(m.unassignedHeading)}
              </h2>
            {/snippet}
            <ul class="grid grid-cols-1 gap-3">
              {#each unassigned as meditation (meditation.id)}
                <li
                  class="flex flex-wrap items-center gap-2 rounded-tile bg-surface-solid/50 p-2 ps-4"
                >
                  <div class="grid min-w-0 flex-1 grid-cols-1 gap-0.5">
                    <span class="font-medium break-words text-ink-strong">
                      {meditation.name}
                    </span>
                    <span class="text-sm text-ink-muted">
                      {formatDuration(
                        meditation.duration_ms,
                        locale.current,
                        "short",
                      )}
                    </span>
                  </div>
                  <OverflowMenu
                    label={t(m.meditationActions, { name: meditation.name })}
                  >
                    <OverflowItem
                      label={t(actionMessages.play)}
                      icon="play"
                      onclick={() =>
                        (window.location.href = playerHref(meditation.id))}
                    />
                    <OverflowItem
                      label={t(m.assignToProgram)}
                      icon="plus"
                      onclick={() => askAssign(meditation)}
                    />
                  </OverflowMenu>
                </li>
              {/each}
            </ul>
          </Panel>
        </div>
      {/if}

      <div class="lg:col-span-2 {selected ? 'block' : 'hidden'}">
        <Panel aria-labelledby="{id}-detail-heading">
          {#snippet heading()}
            {#if selected}
              <Button
                variant="ghost"
                size="sm"
                onclick={() => void backToPrograms()}
              >
                <AppIcon name="arrow-left" size={16} />
                {t(m.backToPrograms)}
              </Button>
            {/if}
            <h2
              id="{id}-detail-heading"
              bind:this={detailHeading}
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
              <Button
                variant="secondary"
                onclick={() => askEditProgram(selected)}
              >
                <AppIcon name="pencil-simple" size={20} />
                {t(m.editProgram)}
              </Button>
              <Button variant="secondary" onclick={() => askUpload(selected)}>
                <AppIcon name="upload-simple" size={20} />
                {t(m.uploadItem)}
              </Button>
              <Button
                variant="secondary"
                onclick={() => void askAddMeditation(selected)}
              >
                <AppIcon name="plus" size={20} />
                {t(m.addMeditationItem)}
              </Button>
            </div>
            <div
              class="mb-4 flex flex-wrap gap-4 rounded-tile bg-surface-solid/50 p-4"
            >
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
                      data.categories.find(
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
              onRetry={() => void loadMembers()}
              emptyTitle={t(m.noMembers)}
              emptyText={t(m.noMembersText)}
            >
              <ol class="grid grid-cols-1 gap-3">
                {#each members ?? [] as meditation, index (meditation.id)}
                  <li
                    id="{id}-row-{meditation.id}"
                    class="flex flex-wrap items-center gap-2 rounded-tile bg-surface-solid/50 p-2 ps-4"
                  >
                    <div class="grid min-w-0 flex-1 grid-cols-1 gap-0.5">
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
                        onclick={() => void move(index, -1)}
                      >
                        <AppIcon name="arrow-up" size={20} />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        data-move="down"
                        disabled={index === (members?.length ?? 0) - 1 ||
                          reordering}
                        aria-label={t(m.moveDown, { name: meditation.name })}
                        onclick={() => void move(index, 1)}
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
                          onclick={() => openEdit(meditation)}
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
                          onclick={() => askRemove(meditation)}
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
      </div>
    </div>
  {/if}
</RegionState>

<FormDialog
  bind:open={categoryOpen}
  title={t(m.addCategoryTitle)}
  submitLabel={t(m.addCategory)}
  canSubmit={categoryName.trim() !== ""}
  onSubmit={runAddCategory}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-category">{t(m.categoryName)}</Label>
    <Input id="{id}-category" bind:value={categoryName} required />
  </div>
</FormDialog>

<FormDialog
  bind:open={renameOpen}
  title={t(m.renameCategoryTitle)}
  description={renameTarget?.name}
  submitLabel={t(actionMessages.save)}
  canSubmit={renameName.trim() !== ""}
  onSubmit={runRenameCategory}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-rename-category">{t(m.categoryName)}</Label>
    <Input id="{id}-rename-category" bind:value={renameName} required />
  </div>
</FormDialog>

<ConfirmDialog
  bind:open={deleteCategoryOpen}
  title={t(m.deleteCategoryDialogTitle)}
  itemName={deleteCategoryTarget?.name}
  consequence={t(m.deleteCategoryDialogText)}
  confirmLabel={t(m.deleteCategoryDialogConfirm)}
  destructive
  onConfirm={runDeleteCategory}
/>

<FormDialog
  bind:open={programOpen}
  title={t(m.addProgramTitle)}
  submitLabel={t(m.createProgram)}
  canSubmit={programTitle.trim() !== ""}
  onSubmit={runAddProgram}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-title">{t(m.title)}</Label>
    <Input id="{id}-title" bind:value={programTitle} required />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-program-description">{t(m.description)}</Label>
    <Textarea id="{id}-program-description" bind:value={programDescription} />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-cover">{t(m.cover)}</Label>
    <Input
      id="{id}-cover"
      type="file"
      accept="image/png,image/jpeg,image/webp"
      aria-describedby="{id}-cover-hint"
      onchange={(event) => (cover = event.currentTarget.files?.[0] ?? null)}
    />
    <p id="{id}-cover-hint" class="text-sm text-ink-muted">{t(m.coverHint)}</p>
  </div>
</FormDialog>

<FormDialog
  bind:open={editProgramOpen}
  title={t(m.editProgramTitle)}
  description={editProgramTarget?.title}
  submitLabel={t(actionMessages.save)}
  canSubmit={editProgramTitle.trim() !== ""}
  onSubmit={runEditProgram}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-title">{t(m.title)}</Label>
    <Input id="{id}-edit-title" bind:value={editProgramTitle} required />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-program-description">{t(m.description)}</Label>
    <Textarea
      id="{id}-edit-program-description"
      bind:value={editProgramDescription}
      rows={3}
    />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-category">{t(m.categoryHeading)}</Label>
    <NativeSelect id="{id}-edit-category" bind:value={editProgramCategory}>
      <option value="">{t(m.uncategorized)}</option>
      {#each data?.categories ?? [] as category (category.id)}
        <option value={String(category.id)}>{category.name}</option>
      {/each}
    </NativeSelect>
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-cover">{t(m.cover)}</Label>
    <Input
      id="{id}-edit-cover"
      type="file"
      accept="image/png,image/jpeg,image/webp"
      aria-describedby="{id}-edit-cover-hint"
      onchange={(event) =>
        (editProgramCover = event.currentTarget.files?.[0] ?? null)}
    />
    <p id="{id}-edit-cover-hint" class="text-sm text-ink-muted">
      {t(m.editProgramHint)}
    </p>
  </div>
</FormDialog>

<FormDialog
  bind:open={addOpen}
  title={t(m.addMeditationTitle)}
  description={programTarget?.title}
  submitLabel={t(m.addMeditationSubmit)}
  canSubmit={addPick !== ""}
  onSubmit={runAddMeditation}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-add-meditation">{t(m.chooseMeditation)}</Label>
    <NativeSelect
      id="{id}-add-meditation"
      bind:value={addPick}
      disabled={!addCandidates.length}
    >
      <option value="" disabled>{t(adminMessages.chooseOption)}</option>
      {#each addCandidates as meditation (meditation.id)}
        <option value={String(meditation.id)}>{meditation.name}</option>
      {/each}
    </NativeSelect>
    {#if !addCandidates.length}
      <p class="text-sm text-ink-muted">{t(m.noMeditationsToAdd)}</p>
    {/if}
  </div>
</FormDialog>

<FormDialog
  bind:open={assignOpen}
  title={t(m.assignTitle)}
  description={assignTarget?.name}
  submitLabel={t(m.addMeditationSubmit)}
  canSubmit={assignPick !== ""}
  onSubmit={runAssign}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-assign-program">{t(m.program)}</Label>
    <NativeSelect id="{id}-assign-program" bind:value={assignPick}>
      <option value="" disabled>{t(adminMessages.chooseOption)}</option>
      {#each data?.programs ?? [] as program (program.id)}
        <option value={String(program.id)}>{program.title}</option>
      {/each}
    </NativeSelect>
  </div>
</FormDialog>

<FormDialog
  bind:open={uploadOpen}
  title={t(m.uploadTitle)}
  description={t(m.uploadHint)}
  submitLabel={t(m.upload)}
  canSubmit={!!audioFile && uploadProgram !== "" && !uploadTooLong}
  onSubmit={runUpload}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-upload-program">{t(m.program)}</Label>
    <NativeSelect id="{id}-upload-program" bind:value={uploadProgram}>
      <option value="" disabled>{t(adminMessages.chooseOption)}</option>
      {#each data?.programs ?? [] as program (program.id)}
        <option value={String(program.id)}>{program.title}</option>
      {/each}
    </NativeSelect>
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-audio">{t(m.audioFile)}</Label>
    <Input
      id="{id}-audio"
      type="file"
      accept=".mp3,.wav,.ogg,.opus,audio/mpeg,audio/wav,audio/ogg,audio/opus"
      onchange={(event) => chooseAudio(event.currentTarget)}
    />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-name">{t(m.name)}</Label>
    <Input id="{id}-name" bind:value={meditationName} />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-description">{t(m.description)}</Label>
    <Textarea
      id="{id}-description"
      bind:value={uploadDescription}
      rows={3}
      aria-invalid={uploadTooLong}
      aria-describedby="{id}-description-count"
    />
    {@render counter(
      `${id}-description-count`,
      uploadDescription,
      uploadTooLong,
    )}
  </div>
</FormDialog>

<FormDialog
  bind:open={editOpen}
  title={t(m.editDescriptionTitle)}
  description={editTarget?.name}
  submitLabel={t(actionMessages.save)}
  canSubmit={!editTooLong}
  onSubmit={runEdit}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-description">{t(m.description)}</Label>
    <Textarea
      id="{id}-edit-description"
      bind:value={editDescription}
      rows={4}
      aria-invalid={editTooLong}
      aria-describedby="{id}-edit-description-count"
    />
    {@render counter(
      `${id}-edit-description-count`,
      editDescription,
      editTooLong,
    )}
    <p class="text-sm text-ink-muted">{t(m.editDescriptionHint)}</p>
  </div>
</FormDialog>

<ConfirmDialog
  bind:open={removeOpen}
  title={t(m.removeTitle)}
  itemName={editTarget?.name}
  consequence={t(m.removeConsequence)}
  confirmLabel={t(m.removeConfirm)}
  destructive
  onConfirm={runRemove}
  fallbackFocus={() => detailHeading}
/>
