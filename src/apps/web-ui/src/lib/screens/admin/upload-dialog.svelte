<!-- @component Upload meditation audio dialog, program prefilled; open it through the exported `ask`. -->
<script lang="ts">
  import {
    titleCaseFromFile,
    uploadAudio,
    type Program,
  } from "$lib/api/meditation";
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import {
    adminMessages,
    meditationAdminMessages as m,
  } from "./admin.messages";
  import DescriptionCounter from "./description-counter.svelte";
  import { DESCRIPTION_MAX, descriptionLength } from "./meditations-logic";
  import { say } from "./meditations-say";
  import NativeSelect from "./native-select.svelte";

  let {
    id,
    programs,
    onUploaded,
  }: {
    id: string;
    programs: Program[];
    onUploaded: (programId: number) => Promise<void>;
  } = $props();

  let open = $state(false);
  let programPick = $state("");
  let audioFile = $state<File | null>(null);
  let meditationName = $state("");
  let description = $state("");
  const tooLong = $derived(
    descriptionLength(description.trim()) > DESCRIPTION_MAX,
  );

  export function ask(program: Program | null) {
    programPick = program ? String(program.id) : "";
    audioFile = null;
    meditationName = "";
    description = "";
    open = true;
  }

  function chooseAudio(input: HTMLInputElement) {
    audioFile = input.files?.[0] ?? null;
    // Title Case prefill from the file name, still editable (FR-024).
    if (audioFile && !meditationName.trim())
      meditationName = titleCaseFromFile(audioFile.name);
  }

  async function run() {
    const programId = Number(programPick);
    if (!audioFile || !programId) return;
    const meditation = await uploadAudio(
      audioFile,
      meditationName.trim(),
      programId,
      description.trim() || null,
    );
    say(m.uploaded, { name: meditation.name });
    await onUploaded(programId);
  }
</script>

<FormDialog
  bind:open
  title={t(m.uploadTitle)}
  description={t(m.uploadHint)}
  submitLabel={t(m.upload)}
  canSubmit={!!audioFile && programPick !== "" && !tooLong}
  onSubmit={run}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-upload-program">{t(m.program)}</Label>
    <NativeSelect id="{id}-upload-program" bind:value={programPick}>
      <option value="" disabled>{t(adminMessages.chooseOption)}</option>
      {#each programs as program (program.id)}
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
      bind:value={description}
      rows={3}
      aria-invalid={tooLong}
      aria-describedby="{id}-description-count"
    />
    <DescriptionCounter
      counterId="{id}-description-count"
      text={description}
      over={tooLong}
    />
  </div>
</FormDialog>
