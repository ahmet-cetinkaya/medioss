<!--
  @component
  The focus overlay's opaque color wash (over the program cover when there
  is one), mounted only while playing.
-->
<script lang="ts">
  import { programCoverUrl, type ProgramSummary } from "$lib/api/meditation";

  type Props = {
    /** undefined while the program is looked up; null for none. */
    program: ProgramSummary | null | undefined;
    showCover: boolean;
    onCoverError: () => void;
  };

  let { program, showCover, onCoverError }: Props = $props();
</script>

<div
  aria-hidden="true"
  class="focus-overlay-backdrop fixed inset-0 z-40"
  data-player-focus-backdrop=""
>
  {#if program && showCover}
    <img
      src={programCoverUrl(program.id)}
      alt=""
      class="focus-overlay-cover absolute inset-0 size-full object-cover"
      onerror={onCoverError}
    />
  {/if}
  <div
    class="focus-overlay-wash absolute inset-0"
    data-over-cover={program && showCover ? "" : undefined}
  ></div>
</div>
