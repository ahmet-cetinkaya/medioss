<!--
  @component
  The program cover (or generated artwork) filling its positioned parent.
  Nothing renders while the program is still being looked up. `wash` (default
  true) adds the white wash for text-free covers; the scrim-captioned inline
  cover turns it off so the scrim is the only layer over the image.
-->
<script lang="ts">
  import { programCoverUrl, type ProgramSummary } from "$lib/api/meditation";
  import GeneratedArtwork from "$lib/components/content/generated-artwork.svelte";

  type Props = {
    /** undefined while the program is looked up; null for none. */
    program: ProgramSummary | null | undefined;
    meditationId: string;
    showCover: boolean;
    onCoverError: () => void;
    wash?: boolean;
  };

  let {
    program,
    meditationId,
    showCover,
    onCoverError,
    wash = true,
  }: Props = $props();
</script>

{#if program && showCover}
  <img
    src={programCoverUrl(program.id)}
    alt=""
    class="absolute inset-0 size-full object-cover"
    onerror={onCoverError}
  />
  {#if wash}
    <span
      aria-hidden="true"
      class="absolute inset-0 bg-(image:--gradient-image-wash)"
    ></span>
  {/if}
{:else if program !== undefined}
  <GeneratedArtwork
    seed={program?.id ?? meditationId}
    category="meditation"
    {wash}
  />
{/if}
