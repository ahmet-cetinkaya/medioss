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
  import {
    fetchCategories,
    fetchLibrary,
    fetchProgramMembers,
    fetchPrograms,
    reorderProgramMeditations,
    type Meditation,
    type Program,
  } from "$lib/api/meditation";
  import { type ServerReason } from "$lib/api/server-reason";
  import {
    failureMessage as formatFailureMessage,
    NETWORK_OR_SERVER_REASONS,
    reasonOf,
  } from "$lib/api/failure";
  import {
    describeFailure,
    type Failure,
  } from "$lib/components/dialogs/form-dialog.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { t } from "$lib/i18n/locale.svelte";
  import { groupPrograms } from "$lib/screens/library/library-groups";
  import {
    adminMessages,
    meditationAdminMessages as m,
  } from "./admin.messages";
  import AddMeditationDialogs from "./add-meditation-dialogs.svelte";
  import CategoryDialogs from "./category-dialogs.svelte";
  import MemberDialogs from "./member-dialogs.svelte";
  import CategoriesPanel from "./meditations-categories-panel.svelte";
  import DetailPanel from "./meditations-detail-panel.svelte";
  import {
    matchesFilter,
    parseProgramId,
    swapped,
    type CategoryFilter,
    type MeditationsData,
  } from "./meditations-logic";
  import ProgramsPanel from "./meditations-programs-panel.svelte";
  import { say } from "./meditations-say";
  import UnassignedPanel from "./meditations-unassigned-panel.svelte";
  import ProgramDialogs from "./program-dialogs.svelte";
  import UploadDialog from "./upload-dialog.svelte";

  const id = $props.id();

  let data = $state<MeditationsData | null>(null);
  let loadFailure = $state<ServerReason | null>(null);
  let request = 0;

  const status = $derived(loadFailure ? "error" : data ? "ready" : "loading");
  const loadErrorText = $derived(
    t(
      formatFailureMessage(loadFailure, adminMessages.loadFailed, {
        specificReasons: NETWORK_OR_SERVER_REASONS,
      }).message,
    ),
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
      if (current === request) loadFailure = reasonOf(error);
    }
  }

  // A deep link (/admin/meditations/<id>/) opens with a program selected.
  onMount(() => void load().then(() => loadMembers()));

  // Selection: category filter ("all" | "none" | category id) and program.
  // The selected program lives in the address (/admin/meditations/<id>/),
  // so reload, Back and bookmarks land on the same view.
  let filter = $state<CategoryFilter>("all");
  let selectedId = $state<number | null>(parseProgramId(location.pathname));
  const selected = $derived(
    data?.programs.find((p) => p.id === selectedId) ?? null,
  );
  const visiblePrograms = $derived(
    (data?.programs ?? []).filter((p) => matchesFilter(p, filter)),
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

  // Reload the lists; also the members when the changed program is open.
  async function refresh(programId?: number) {
    await load(false);
    if (programId !== undefined && programId === selectedId)
      await loadMembers(false);
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
  // Failure of a member action; cleared on selection change.
  let memberFailure = $state<Failure | null>(null);

  function focusMovedRow(movedId: number, delta: -1 | 1) {
    const dir = delta < 0 ? "up" : "down";
    const other = delta < 0 ? "down" : "up";
    const row = document.getElementById(`${id}-row-${movedId}`);
    const button =
      row?.querySelector<HTMLButtonElement>(
        `[data-move="${dir}"]:not(:disabled)`,
      ) ??
      row?.querySelector<HTMLButtonElement>(
        `[data-move="${other}"]:not(:disabled)`,
      );
    button?.focus();
  }

  async function move(index: number, delta: -1 | 1) {
    const programId = selectedId;
    const prev = members;
    const target = index + delta;
    if (programId === null || !prev || reordering) return;
    if (target < 0 || target >= prev.length) return;
    const next = swapped(prev, index, target);
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
    focusMovedRow(moved.id, delta);
  }

  let categoryDialogs = $state<ReturnType<typeof CategoryDialogs>>();
  let programDialogs = $state<ReturnType<typeof ProgramDialogs>>();
  let addMeditationDialogs = $state<ReturnType<typeof AddMeditationDialogs>>();
  let uploadDialog = $state<ReturnType<typeof UploadDialog>>();
  let memberDialogs = $state<ReturnType<typeof MemberDialogs>>();

  let coverVersions = $state<Record<number, number>>({});
  const coverVersion = (programId: number) => coverVersions[programId] ?? 0;
  const bumpCoverVersion = (programId: number) =>
    (coverVersions[programId] = coverVersion(programId) + 1);
</script>

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
        <ProgramsPanel
          {id}
          groups={groupedPrograms}
          hasVisible={visiblePrograms.length > 0}
          hasAny={data.programs.length > 0}
          {selectedId}
          {coverVersion}
          bind:headingElement={programsHeading}
          onAdd={() => programDialogs?.askAdd()}
          onSelect={(program) => void selectProgram(program)}
          onEdit={(program) => programDialogs?.askEdit(program)}
          onAddMeditation={(program) =>
            void addMeditationDialogs?.askAdd(program)}
          onUpload={(program) => uploadDialog?.ask(program)}
          onDelete={(program) => programDialogs?.askDelete(program)}
        />
      </div>

      <div class={selected ? "hidden" : "block"}>
        <CategoriesPanel
          {id}
          categories={data.categories}
          bind:filter
          {countIn}
          bind:headingElement={categoriesHeading}
          onAdd={() => categoryDialogs?.askAdd()}
          onRename={(category) => categoryDialogs?.askRename(category)}
          onDelete={(category) => categoryDialogs?.askDelete(category)}
        />
      </div>

      {#if !selected && unassigned.length}
        <div class="lg:col-span-2">
          <UnassignedPanel
            {id}
            meditations={unassigned}
            onAssign={(meditation) =>
              addMeditationDialogs?.askAssign(meditation)}
          />
        </div>
      {/if}

      <div class="lg:col-span-2 {selected ? 'block' : 'hidden'}">
        <DetailPanel
          {id}
          {selected}
          categories={data.categories}
          {members}
          {membersFailure}
          {reordering}
          {reorderFailure}
          {memberFailure}
          {coverVersion}
          bind:headingElement={detailHeading}
          onBack={() => void backToPrograms()}
          onEdit={(program) => programDialogs?.askEdit(program)}
          onUpload={(program) => uploadDialog?.ask(program)}
          onDelete={(program) => programDialogs?.askDelete(program)}
          onAddMeditation={(program) =>
            void addMeditationDialogs?.askAdd(program)}
          onRetryMembers={() => void loadMembers()}
          onMove={(index, delta) => void move(index, delta)}
          onEditDescription={(meditation) =>
            memberDialogs?.askEditDescription(meditation)}
          onRemove={(meditation) => memberDialogs?.askRemove(meditation)}
          onDeleteMeditation={(meditation) =>
            memberDialogs?.askDelete(meditation)}
        />
      </div>
    </div>
  {/if}
</RegionState>

<CategoryDialogs
  bind:this={categoryDialogs}
  {id}
  onChanged={() => refresh()}
  onDeleting={(category) => {
    if (filter === category.id) filter = "all";
  }}
/>

<ProgramDialogs
  bind:this={programDialogs}
  {id}
  categories={data?.categories ?? []}
  onChanged={refresh}
  onCoverReplaced={bumpCoverVersion}
  onDeleting={(program) => {
    if (selectedId === program.id) void backToPrograms();
  }}
/>

<AddMeditationDialogs
  bind:this={addMeditationDialogs}
  {id}
  programs={data?.programs ?? []}
  library={data?.library ?? []}
  {selectedId}
  {members}
  onChanged={refresh}
/>

<UploadDialog
  bind:this={uploadDialog}
  {id}
  programs={data?.programs ?? []}
  onUploaded={refresh}
/>

<MemberDialogs
  bind:this={memberDialogs}
  {id}
  {selectedId}
  onChanged={() => refresh(selectedId ?? undefined)}
  fallbackFocus={() => detailHeading}
/>
