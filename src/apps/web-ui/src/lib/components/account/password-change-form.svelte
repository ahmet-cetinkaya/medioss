<!--
  @component
  New password + confirmation form (FR-103, FR-104). Used by the required
  change view and by Settings.

  - The rules (`password-rules.ts`) run on submit before any request: too
    short shows the minimum-length rule on the first field, a mismatch shows
    the mismatch message on the second; focus moves to that field and
    nothing is sent.
  - While the request runs the submit button is busy (one request per
    activation).
  - A refusal shows `t(serverReasonMessages[reason])` (role="alert"); server
    text is never shown.
  - Success clears both fields, announces "Your password has been changed"
    through the page announcer, and calls `onSuccess(user)`.
  - Passwords live only in this component's state; nothing is stored.

  Props: `onSuccess?(user)`, `busy?: boolean` (bindable, default the form's
  own state): lets a wrapping dialog ignore Escape and outside clicks while
  the request runs, like FormDialog does.
-->
<script lang="ts">
  import { ApiError } from "$lib/api/api-error";
  import { changePassword, type MeProfile } from "$lib/api/identity";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
  import { passwordMessages } from "./account.messages";
  import {
    PASSWORD_MIN_LENGTH,
    type PasswordRuleViolation,
    validateNewPassword,
  } from "./password-rules";

  type Props = {
    onSuccess?: (user: MeProfile) => void | Promise<void>;
    busy?: boolean;
  };

  let { onSuccess, busy = $bindable(false) }: Props = $props();

  const id = $props.id();
  const min = PASSWORD_MIN_LENGTH;

  let password = $state("");
  let confirm = $state("");
  let violation = $state<PasswordRuleViolation | null>(null);
  let refusal = $state<ServerReason | null>(null);
  let passwordRef = $state<HTMLInputElement | null>(null);
  let confirmRef = $state<HTMLInputElement | null>(null);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    refusal = null;
    violation = validateNewPassword(password, confirm);
    if (violation === "tooShort") return passwordRef?.focus();
    if (violation === "mismatch") return confirmRef?.focus();

    busy = true;
    try {
      const user = await changePassword(password);
      password = "";
      confirm = "";
      announce(t(passwordMessages.changed));
      await onSuccess?.(user);
    } catch (error) {
      refusal =
        error instanceof ApiError ? serverReason(error).reason : "generic";
    } finally {
      busy = false;
    }
  }
</script>

<form class="flex flex-col gap-5" novalidate onsubmit={submit}>
  <div class="flex flex-col gap-2">
    <label for="{id}-new" class="text-sm font-medium text-ink-strong">
      {t(passwordMessages.newPassword)}
    </label>
    <Input
      bind:ref={passwordRef}
      bind:value={password}
      id="{id}-new"
      type="password"
      autocomplete="new-password"
      aria-invalid={violation === "tooShort" || undefined}
      aria-describedby={violation === "tooShort"
        ? `${id}-new-error`
        : `${id}-hint`}
    />
    {#if violation === "tooShort"}
      <InlineMessage tone="error" role="alert">
        <span id="{id}-new-error">{t(passwordMessages.tooShort, { min })}</span>
      </InlineMessage>
    {:else}
      <p id="{id}-hint" class="text-sm text-ink-muted">
        {t(passwordMessages.hint, { min })}
      </p>
    {/if}
  </div>
  <div class="flex flex-col gap-2">
    <label for="{id}-confirm" class="text-sm font-medium text-ink-strong">
      {t(passwordMessages.confirmPassword)}
    </label>
    <Input
      bind:ref={confirmRef}
      bind:value={confirm}
      id="{id}-confirm"
      type="password"
      autocomplete="new-password"
      aria-invalid={violation === "mismatch" || undefined}
      aria-describedby={violation === "mismatch"
        ? `${id}-confirm-error`
        : undefined}
    />
    {#if violation === "mismatch"}
      <InlineMessage tone="error" role="alert">
        <span id="{id}-confirm-error">{t(passwordMessages.mismatch)}</span>
      </InlineMessage>
    {/if}
  </div>
  {#if refusal}
    <InlineMessage tone="error" role="alert">
      {t(serverReasonMessages[refusal])}
    </InlineMessage>
  {/if}
  <div>
    <Button type="submit" {busy}>{t(passwordMessages.submit)}</Button>
  </div>
</form>
