<!-- @component Dialogs that put a meditation into a program: pick a
  meditation for a program (search + card list), or pick a program for a
  meditation. Clicking a card performs the add. -->
<script lang="ts">
  import {
    addMember,
    fetchProgramMembers,
    type Meditation,
    type Program,
  } from "$lib/api/meditation";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import { Input } from "$lib/components/ui/input/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { meditationAdminMessages as m } from "./admin.messages";
  import { foldForSearch } from "./meditations-logic";
  import { say } from "./meditations-say";
  import MeditationPickerList from "./meditation-picker-list.svelte";
  import { describeFailure } from "$lib/components/dialogs/form-dialog.svelte";

  let {
    id,
    programs,
    library,
    selectedId,
    members,
    onChanged,
  }: {
    id: string;
    programs: Program[];
    library: Meditation[];
    selectedId: number | null;
    /** Loaded members of the selected program, reused to skip a fetch. */
    members: Meditation[] | null;
    /** Reload after a change; `programId` names the program that changed. */
    onChanged: (programId?: number) => Promise<void>;
  } = $props();

  // --- Add a meditation to a program (from the program side). ---
  let addOpen = $state(false);
  let addTarget = $state<Program | null>(null);
  let addSearch = $state("");
  let addCandidates = $state<Meditation[]>([]);
  let addBusy = $state(false);
  let addFailure = $state<ReturnType<typeof describeFailure> | null>(null);

  export async function askAdd(program: Program) {
    addTarget = program;
    addSearch = "";
    addFailure = null;
    const inProgram =
      program.id === selectedId && members
        ? members
        : await fetchProgramMembers(program.id).catch(() => []);
    const taken = new Set(inProgram.map((x) => x.id));
    addCandidates = library.filter((x) => !taken.has(x.id));
    addOpen = true;
  }

  async function runAdd(meditation: Meditation) {
    const program = addTarget;
    if (!program || addBusy) return;
    addBusy = true;
    addFailure = null;
    try {
      await addMember(program.id, meditation.id);
      say(m.addedToProgram, { name: meditation.name, title: program.title });
      addOpen = false;
      await onChanged(program.id);
    } catch (error) {
      addFailure = describeFailure(error);
    } finally {
      addBusy = false;
    }
  }

  // --- Add a meditation to another program (from the meditation side). ---
  let assignOpen = $state(false);
  let assignTarget = $state<Meditation | null>(null);
  let assignSearch = $state("");
  let assignBusy = $state(false);
  let assignFailure = $state<ReturnType<typeof describeFailure> | null>(null);

  export function askAssign(meditation: Meditation) {
    // Program-member rows lack `programs`; the library row carries them.
    assignTarget = library.find((x) => x.id === meditation.id) ?? meditation;
    assignSearch = "";
    assignFailure = null;
    assignOpen = true;
  }

  async function runAssign(program: Program) {
    const meditation = assignTarget;
    if (!meditation || assignBusy) return;
    assignBusy = true;
    assignFailure = null;
    try {
      await addMember(program.id, meditation.id);
      say(m.addedToProgram, { name: meditation.name, title: program.title });
      assignOpen = false;
      await onChanged(program.id);
    } catch (error) {
      assignFailure = describeFailure(error);
    } finally {
      assignBusy = false;
    }
  }

  const assignNeedle = $derived(foldForSearch(assignSearch.trim()));
  const assignMatches = $derived(
    assignTarget
      ? programs.filter(
          (program) =>
            !(assignTarget.programs ?? []).some((p) => p.id === program.id) &&
            (assignNeedle === "" ||
              foldForSearch(program.title).includes(assignNeedle)),
        )
      : [],
  );
</script>

<Dialog.Root bind:open={addOpen}>
  <Dialog.Content
    showCloseButton={false}
    escapeKeydownBehavior={addBusy ? "ignore" : "close"}
    interactOutsideBehavior={addBusy ? "ignore" : "close"}
    aria-busy={addBusy}
  >
    <Dialog.Header>
      <Dialog.Title>{t(m.addMeditationTitle)}</Dialog.Title>
      <Dialog.Description>{addTarget?.title}</Dialog.Description>
    </Dialog.Header>
    <div class="grid grid-cols-1 gap-3">
      <Input
        id="{id}-add-search"
        type="search"
        bind:value={addSearch}
        placeholder={t(m.searchMeditationsLabel)}
        aria-label={t(m.searchMeditationsLabel)}
      />
      <p class="text-sm text-ink-muted">{t(m.addFromLibraryHint)}</p>
      {#if addFailure}
        <InlineMessage tone="error" role="alert">
          {t(addFailure.message, addFailure.params)}
        </InlineMessage>
      {/if}
      <MeditationPickerList
        listId="{id}-add-list"
        meditations={addCandidates}
        search={addSearch}
        onPick={(meditation) => void runAdd(meditation)}
      />
    </div>
    <Dialog.Footer>
      <Button
        variant="ghost"
        disabled={addBusy}
        onclick={() => (addOpen = false)}
      >
        {t(actionMessages.cancel)}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={assignOpen}>
  <Dialog.Content
    showCloseButton={false}
    escapeKeydownBehavior={assignBusy ? "ignore" : "close"}
    interactOutsideBehavior={assignBusy ? "ignore" : "close"}
    aria-busy={assignBusy}
  >
    <Dialog.Header>
      <Dialog.Title>{t(m.assignTitle)}</Dialog.Title>
      <Dialog.Description>{assignTarget?.name}</Dialog.Description>
    </Dialog.Header>
    <div class="grid grid-cols-1 gap-3">
      <Input
        id="{id}-assign-search"
        type="search"
        bind:value={assignSearch}
        placeholder={t(m.searchProgramsLabel)}
        aria-label={t(m.searchProgramsLabel)}
      />
      {#if assignFailure}
        <InlineMessage tone="error" role="alert">
          {t(assignFailure.message, assignFailure.params)}
        </InlineMessage>
      {/if}
      <ul
        id="{id}-assign-list"
        class="grid max-h-80 grid-cols-1 gap-2 overflow-y-auto pe-1"
      >
        {#each assignMatches as program (program.id)}
          <li>
            <button
              type="button"
              class="flex w-full flex-wrap items-center gap-2 rounded-tile border border-rim bg-surface-solid/50 p-3 text-start hover:bg-surface-solid/80"
              onclick={() => void runAssign(program)}
            >
              <span class="min-w-0 flex-1 font-medium break-words"
                >{program.title}</span
              >
            </button>
          </li>
        {:else}
          <li class="text-sm text-ink-muted">
            {t(
              assignNeedle === "" ? m.assignInEveryProgram : m.assignNoMatches,
            )}
          </li>
        {/each}
      </ul>
    </div>
    <Dialog.Footer>
      <Button
        variant="ghost"
        disabled={assignBusy}
        onclick={() => (assignOpen = false)}
      >
        {t(actionMessages.cancel)}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
