<!--
  @component
  Confirmation dialog (plan D6, FR-016–FR-018, FR-020) on AlertDialog.

  Props: `open` (bindable), `title`, `itemName?` (user content, shown as-is),
  `consequence`, `confirmLabel` (a specific verb such as "Delete mix"),
  `destructive` (default false: destructive vs primary confirm),
  `onConfirm: () => Promise<void>`, `returnFocusTo?` (focus target on close;
  default: the element focused when the dialog opened), `fallbackFocus?`
  (element or getter used when that target was removed, e.g. a deleted row),
  `reasonContext?` (passed to serverReason), `errorMessage?` (returns a
  Message for an error, checked before the server reason).

  Focus starts on Cancel. Escape cancels, except while the action runs: then
  Escape is ignored, Cancel is disabled, the confirm shows progress, and a
  second activation is a no-op. Success closes; a rejection stays open and
  shows the reason (role="alert"), stored as a Message so a language switch
  re-renders it (FR-113).
-->
<script lang="ts">
  import type { ServerReasonContext } from "$lib/api/server-reason";
  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import {
    describeFailure,
    type ErrorMessage,
    type Failure,
  } from "./form-dialog.svelte";
  import { restoreFocus, type FocusSource } from "./return-focus";

  type Props = {
    open?: boolean;
    title: string;
    itemName?: string;
    consequence: string;
    confirmLabel: string;
    destructive?: boolean;
    onConfirm: () => Promise<void>;
    returnFocusTo?: HTMLElement | null;
    fallbackFocus?: FocusSource<HTMLElement>;
    reasonContext?: ServerReasonContext;
    errorMessage?: ErrorMessage;
  };

  let {
    open = $bindable(false),
    title,
    itemName,
    consequence,
    confirmLabel,
    destructive = false,
    onConfirm,
    returnFocusTo,
    fallbackFocus,
    reasonContext,
    errorMessage,
  }: Props = $props();

  let submitting = $state(false);
  let failure = $state<Failure | null>(null);
  let cancelRef = $state<HTMLButtonElement | null>(null);
  let contentRef = $state<HTMLElement | null>(null);
  let opener: HTMLElement | null = null;

  $effect.pre(() => {
    if (!open) failure = null;
  });

  async function runAction() {
    if (submitting) return;
    submitting = true;
    failure = null;
    try {
      await onConfirm();
      open = false;
    } catch (error) {
      failure = describeFailure(error, reasonContext, errorMessage);
    } finally {
      submitting = false;
    }
  }
</script>

<AlertDialog.Root bind:open>
  <AlertDialog.Content
    bind:ref={contentRef}
    escapeKeydownBehavior={submitting ? "ignore" : "close"}
    aria-busy={submitting}
    onOpenAutoFocus={(event) => {
      opener =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      // Cancel's ref binds after this fires; focus it on the next tick.
      event.preventDefault();
      setTimeout(() => (cancelRef ?? contentRef)?.focus());
    }}
    onCloseAutoFocus={(event) =>
      restoreFocus(event, returnFocusTo ?? opener, fallbackFocus)}
  >
    <AlertDialog.Header>
      <AlertDialog.Title>{title}</AlertDialog.Title>
      <AlertDialog.Description class="grid gap-2">
        {#if itemName}
          <span class="font-semibold break-words text-ink-strong"
            >{itemName}</span
          >
        {/if}
        <span>{consequence}</span>
      </AlertDialog.Description>
    </AlertDialog.Header>
    {#if failure}
      <InlineMessage tone="error" role="alert">
        {t(failure.message, failure.params)}
      </InlineMessage>
    {/if}
    <AlertDialog.Footer>
      <AlertDialog.Cancel bind:ref={cancelRef} disabled={submitting}>
        {t(actionMessages.cancel)}
      </AlertDialog.Cancel>
      <Button
        variant={destructive ? "destructive" : "primary"}
        busy={submitting}
        onclick={runAction}
      >
        {confirmLabel}
      </Button>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
