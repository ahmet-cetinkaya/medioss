<!--
  @component
  The four states of a data region (plan rule 5).

  - `loading`: an aria-hidden skeleton that reserves `minHeight`, the region
    marked `aria-busy`, and visually hidden "Loading…" text (FR-013).
  - `error`: an InlineMessage with `errorMessage` plus a Retry button when
    `onRetry` is given (FR-011). Never redirects. `errorMessage` is text the
    caller already translated, e.g. `t(serverReasonMessages[reason])`;
    without it the generic "Could not load this section" shows.
  - `empty`: `emptyTitle` (heading at `headingLevel`, default 3), `emptyText`
    and the optional `emptyAction` snippet (FR-014).
  - `ready`: renders `children`.

  Props: `status`, `minHeight` (CSS length, default "6rem"; reserved in the
  loading, error and empty states), `errorMessage`, `onRetry`, `emptyTitle`,
  `emptyText`, `emptyAction`, `headingLevel` (2 | 3 | 4), `label` (translated;
  when set, the region is a labelled `<section>`), `class`, `children`.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { actionMessages, stateMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import InlineMessage from "./inline-message.svelte";

  type Props = {
    status: "loading" | "error" | "empty" | "ready";
    minHeight?: string;
    errorMessage?: string;
    onRetry?: () => void;
    emptyTitle?: string;
    emptyText?: string;
    emptyAction?: Snippet;
    headingLevel?: 2 | 3 | 4;
    label?: string;
    class?: string;
    children: Snippet;
  };

  let {
    status,
    minHeight = "6rem",
    errorMessage,
    onRetry,
    emptyTitle,
    emptyText,
    emptyAction,
    headingLevel = 3,
    label,
    class: className,
    children,
  }: Props = $props();
</script>

<svelte:element
  this={label ? "section" : "div"}
  aria-label={label}
  aria-busy={status === "loading" || undefined}
  data-region-state={status}
  class={className}
  style:min-height={status === "ready" ? undefined : minHeight}
>
  {#if status === "loading"}
    <div
      aria-hidden="true"
      class="flex min-h-[inherit] flex-col gap-3 rounded-tile bg-surface-solid/50 p-6"
    >
      <div
        class="h-4 w-2/3 rounded-full bg-surface-solid motion-safe:animate-pulse"
      ></div>
      <div
        class="h-4 w-1/2 rounded-full bg-surface-solid motion-safe:animate-pulse"
      ></div>
    </div>
    <span class="sr-only">{t(stateMessages.loading)}</span>
  {:else if status === "error"}
    <div class="flex flex-col items-start gap-3">
      <InlineMessage tone="error">
        {errorMessage ?? t(stateMessages.sectionLoadFailed)}
      </InlineMessage>
      {#if onRetry}
        <Button variant="secondary" onclick={onRetry}>
          {t(actionMessages.retry)}
        </Button>
      {/if}
    </div>
  {:else if status === "empty"}
    <div class="flex flex-col items-start gap-2">
      <svelte:element
        this={`h${headingLevel}`}
        class="text-xl font-semibold text-ink-strong"
      >
        {emptyTitle ?? t(stateMessages.empty)}
      </svelte:element>
      {#if emptyText}
        <p class="text-sm text-ink-body">{emptyText}</p>
      {/if}
      {#if emptyAction}
        <div class="mt-2">{@render emptyAction()}</div>
      {/if}
    </div>
  {:else}
    {@render children()}
  {/if}
</svelte:element>
