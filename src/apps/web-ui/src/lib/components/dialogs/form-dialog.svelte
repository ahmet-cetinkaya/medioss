<!--
  @component
  Generic form dialog (plan D6): title, optional description, fields
  (children), Cancel, and one primary submit button (FR-101).

  Props: `open` (bindable), `title`, `description?`, `submitLabel`,
  `onSubmit: () => Promise<void>`, `canSubmit` (default true; false disables
  submit), `destructive` (default false: the submit button uses the destructive
  variant), `children` (field snippet), `returnFocusTo?` (focus target on
  close; default: the element focused when the dialog opened),
  `fallbackFocus?` (element or getter used when that target was removed),
  `reasonContext?` (passed to serverReason), `errorMessage?` (returns a
  Message for an error, checked before the server reason).

  Busy: the submit runs once per activation; while it runs, submit shows
  progress, Cancel is disabled, and Escape and outside clicks are ignored.
  A never-settling promise (401 redirect) keeps it busy. Success closes the
  dialog; a rejection keeps it open with the reason (role="alert"). The reason
  is stored as a Message, so a language switch re-renders it (FR-113).
-->
<script lang="ts" module>
  import { ApiError } from "$lib/api/api-error";
  import {
    serverReason,
    type ServerReasonContext,
  } from "$lib/api/server-reason";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
  import type { Message, MessageParams } from "$lib/i18n/message";

  export type Failure = { message: Message; params?: MessageParams };

  export type ErrorMessage = (error: unknown) => Message | null | undefined;

  /** Turns a rejection into a Message plus params (never translated text). */
  export function describeFailure(
    error: unknown,
    context?: ServerReasonContext,
    errorMessage?: ErrorMessage,
  ): Failure {
    const custom = errorMessage?.(error);
    if (custom) return { message: custom };
    if (error instanceof ApiError) {
      const { reason, params } = serverReason(error, context);
      return { message: serverReasonMessages[reason], params };
    }
    return { message: serverReasonMessages.generic };
  }
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import { restoreFocus, type FocusSource } from "./return-focus";

  type Props = {
    open?: boolean;
    title: string;
    description?: string;
    submitLabel: string;
    destructive?: boolean;
    onSubmit: () => Promise<void>;
    canSubmit?: boolean;
    children: Snippet;
    returnFocusTo?: HTMLElement | null;
    fallbackFocus?: FocusSource<HTMLElement>;
    reasonContext?: ServerReasonContext;
    errorMessage?: ErrorMessage;
  };

  let {
    open = $bindable(false),
    title,
    description,
    submitLabel,
    destructive = false,
    onSubmit,
    canSubmit = true,
    children,
    returnFocusTo,
    fallbackFocus,
    reasonContext,
    errorMessage,
  }: Props = $props();

  let submitting = $state(false);
  let failure = $state<Failure | null>(null);
  let opener: HTMLElement | null = null;

  $effect.pre(() => {
    if (!open) failure = null;
  });

  async function runSubmit(event: SubmitEvent) {
    event.preventDefault();
    if (submitting || !canSubmit) return;
    submitting = true;
    failure = null;
    try {
      await onSubmit();
      open = false;
    } catch (error) {
      failure = describeFailure(error, reasonContext, errorMessage);
    } finally {
      submitting = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content
    showCloseButton={false}
    escapeKeydownBehavior={submitting ? "ignore" : "close"}
    interactOutsideBehavior={submitting ? "ignore" : "close"}
    onOpenAutoFocus={() => {
      opener =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
    }}
    onCloseAutoFocus={(event) =>
      restoreFocus(event, returnFocusTo ?? opener, fallbackFocus)}
  >
    <form class="grid gap-5" onsubmit={runSubmit} aria-busy={submitting}>
      <Dialog.Header>
        <Dialog.Title>{title}</Dialog.Title>
        {#if description}
          <Dialog.Description>{description}</Dialog.Description>
        {/if}
      </Dialog.Header>
      <div class="grid gap-4">{@render children()}</div>
      {#if failure}
        <InlineMessage tone="error" role="alert">
          {t(failure.message, failure.params)}
        </InlineMessage>
      {/if}
      <Dialog.Footer>
        <Button
          variant="ghost"
          disabled={submitting}
          onclick={() => (open = false)}
        >
          {t(actionMessages.cancel)}
        </Button>
        <Button
          type="submit"
          variant={destructive ? "destructive" : "primary"}
          busy={submitting}
          disabled={!canSubmit}
        >
          {submitLabel}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
