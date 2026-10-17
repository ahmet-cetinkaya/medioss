<!-- @component Dialogs that put a meditation into a program: pick an existing one, or assign an unassigned one. -->
<script lang="ts">
  import {
    addMember,
    fetchProgramMembers,
    type Meditation,
    type Program,
  } from "$lib/api/meditation";
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import { Label } from "$lib/components/ui/label/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import {
    adminMessages,
    meditationAdminMessages as m,
  } from "./admin.messages";
  import { say } from "./meditations-say";
  import NativeSelect from "./native-select.svelte";

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

  let addOpen = $state(false);
  let addTarget = $state<Program | null>(null);
  let addPick = $state("");
  let addCandidates = $state<Meditation[]>([]);

  export async function askAdd(program: Program) {
    addTarget = program;
    addPick = "";
    const inProgram =
      program.id === selectedId && members
        ? members
        : await fetchProgramMembers(program.id).catch(() => []);
    const taken = new Set(inProgram.map((x) => x.id));
    addCandidates = library.filter((x) => !taken.has(x.id));
    addOpen = true;
  }

  async function runAdd() {
    const program = addTarget;
    const meditation = addCandidates.find((x) => x.id === Number(addPick));
    if (!program || !meditation) return;
    await addMember(program.id, meditation.id);
    say(m.addedToProgram, { name: meditation.name, title: program.title });
    await onChanged(program.id);
  }

  let assignOpen = $state(false);
  let assignTarget = $state<Meditation | null>(null);
  let assignPick = $state("");

  export function askAssign(meditation: Meditation) {
    assignTarget = meditation;
    assignPick = "";
    assignOpen = true;
  }

  async function runAssign() {
    const meditation = assignTarget;
    const program = programs.find((p) => p.id === Number(assignPick));
    if (!meditation || !program) return;
    await addMember(program.id, meditation.id);
    say(m.addedToProgram, { name: meditation.name, title: program.title });
    await onChanged();
  }
</script>

<FormDialog
  bind:open={addOpen}
  title={t(m.addMeditationTitle)}
  description={addTarget?.title}
  submitLabel={t(m.addMeditationSubmit)}
  canSubmit={addPick !== ""}
  onSubmit={runAdd}
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
      {#each programs as program (program.id)}
        <option value={String(program.id)}>{program.title}</option>
      {/each}
    </NativeSelect>
  </div>
</FormDialog>
