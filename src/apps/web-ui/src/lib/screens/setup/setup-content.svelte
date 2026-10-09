<!--
  @component
  Setup content (FR-083, FR-084, PB-15; plan D2 "Setup page"). No `/me` probe.

  - Loading until setup-status answers: no form exists meanwhile (FR-083).
  - A status failure shows the error with Retry and no form (FR-084).
  - Already set up: the message with a Login link.
  - The form: setup secret (password input, autocomplete off), admin email,
    admin password (at least 8 code points, checked before any request).
    The secret is cleared after every attempt and never stored (PB-15).
  - Success: the message with a Login link; no navigation. A refusal shows
    `t(serverReasonMessages[reason])`; server text is never shown.
-->
<script lang="ts">
  import { tick } from "svelte";
  import { ApiError } from "$lib/api/api-error";
  import { completeSetup, fetchSetupStatus } from "$lib/api/identity";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import { PASSWORD_MIN_LENGTH } from "$lib/components/account/password-rules";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
  import { setupMessages as m } from "./setup.messages";

  const LOGIN_PATH = "/login/";
  const min = PASSWORD_MIN_LENGTH;
  const id = $props.id();

  let setupRequired = $state<boolean | null>(null);
  let statusFailure = $state<ServerReason | null>(null);
  let secret = $state("");
  let email = $state("");
  let password = $state("");
  let tooShort = $state(false);
  let busy = $state(false);
  let done = $state(false);
  let refusal = $state<ServerReason | null>(null);
  let passwordRef = $state<HTMLInputElement | null>(null);
  let loginRef = $state<HTMLElement | null>(null);
  let request = 0;

  const status = $derived(
    statusFailure ? "error" : setupRequired === null ? "loading" : "ready",
  );
  const statusError = $derived(
    statusFailure === "network" || statusFailure === "server"
      ? t(serverReasonMessages[statusFailure])
      : t(m.statusFailed),
  );
  const refusalText = $derived(
    refusal && refusal !== "generic"
      ? t(serverReasonMessages[refusal])
      : t(m.failed),
  );

  async function load() {
    const current = ++request;
    setupRequired = null;
    statusFailure = null;
    try {
      const result = await fetchSetupStatus();
      if (current === request) setupRequired = result.setup_required;
    } catch (error) {
      if (current !== request) return;
      statusFailure =
        error instanceof ApiError ? serverReason(error).reason : "generic";
    }
  }

  $effect(() => {
    void load();
  });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    refusal = null;
    tooShort = [...password].length < min;
    if (tooShort) {
      secret = "";
      return passwordRef?.focus();
    }

    busy = true;
    try {
      await completeSetup(secret, email, password);
      password = "";
      done = true;
      announce(t(m.done));
      await tick();
      loginRef?.focus();
    } catch (error) {
      refusal =
        error instanceof ApiError ? serverReason(error).reason : "generic";
      if (refusal === "setupAlreadyCompleted") setupRequired = false;
    } finally {
      // PB-15: the secret never outlives an attempt.
      secret = "";
      busy = false;
    }
  }
</script>

<RegionState
  {status}
  minHeight="18rem"
  errorMessage={statusError}
  onRetry={load}
>
  {#if done}
    <div class="flex flex-col items-start gap-4">
      <InlineMessage tone="success">{t(m.done)}</InlineMessage>
      <Button bind:ref={loginRef} href={LOGIN_PATH}>{t(m.goToLogin)}</Button>
    </div>
  {:else if !setupRequired}
    <div class="flex flex-col items-start gap-4">
      <p class="text-ink-body">{t(m.alreadySetUp)}</p>
      <Button href={LOGIN_PATH}>{t(m.goToLogin)}</Button>
    </div>
  {:else}
    <form class="flex flex-col gap-5" novalidate onsubmit={submit}>
      <p class="text-ink-body">{t(m.intro)}</p>
      <div class="flex flex-col gap-2">
        <label for="{id}-secret" class="text-sm font-medium text-ink-strong">
          {t(m.secret)}
        </label>
        <Input
          bind:value={secret}
          id="{id}-secret"
          type="password"
          autocomplete="off"
          required
          aria-describedby="{id}-secret-hint"
        />
        <p id="{id}-secret-hint" class="text-sm text-ink-muted">
          {t(m.secretHint)}
        </p>
      </div>
      <div class="flex flex-col gap-2">
        <label for="{id}-email" class="text-sm font-medium text-ink-strong">
          {t(m.email)}
        </label>
        <Input
          bind:value={email}
          id="{id}-email"
          type="email"
          autocomplete="username"
          required
        />
      </div>
      <div class="flex flex-col gap-2">
        <label for="{id}-password" class="text-sm font-medium text-ink-strong">
          {t(m.password)}
        </label>
        <Input
          bind:ref={passwordRef}
          bind:value={password}
          id="{id}-password"
          type="password"
          autocomplete="new-password"
          required
          aria-invalid={tooShort || undefined}
          aria-describedby="{id}-password-note"
        />
        {#if tooShort}
          <InlineMessage tone="error" role="alert">
            <span id="{id}-password-note">
              {t(m.passwordTooShort, { min })}
            </span>
          </InlineMessage>
        {:else}
          <p id="{id}-password-note" class="text-sm text-ink-muted">
            {t(m.passwordHint, { min })}
          </p>
        {/if}
      </div>
      {#if refusal}
        <InlineMessage tone="error" role="alert">{refusalText}</InlineMessage>
      {/if}
      <Button type="submit" class="w-full" {busy}>{t(m.submit)}</Button>
    </form>
  {/if}
</RegionState>
