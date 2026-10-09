<!--
  @component
  Login content (FR-080–FR-082, PB-14; plan D2 "Login page").

  - On load it probes `/me` (no redirect on 401). A signed-in visitor is sent
    on with `location.replace(safe next ?? "/")`; a 401, a network error, or
    any other failure shows the form. No form shows while probing.
  - The Setup link shows only when setup-status says `setup_required`.
  - A 401 shows one generic message, keeps the email, and clears the password.
  - A 429 starts a countdown from Retry-After; Sign in stays disabled until
    it reaches 0. The start and the end are announced politely.
  - Success replaces the page with the same destination; credentials live
    only in this component's state.
-->
<script lang="ts">
  import { ApiError } from "$lib/api/api-error";
  import { fetchMeProfile, fetchSetupStatus, login } from "$lib/api/identity";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import {
    formatClock,
    formatClockSpoken,
  } from "$lib/i18n/formatting/clock-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
  import { loginDestination } from "./login-destination";
  import { loginMessages as m, RESET_PASSWORD_COMMAND } from "./login.messages";

  const SETUP_PATH = "/setup/";
  const id = $props.id();

  let probing = $state(true);
  let setupRequired = $state(false);
  let email = $state("");
  let password = $state("");
  let busy = $state(false);
  let refusal = $state<ServerReason | null>(null);
  let remaining = $state(0);
  let passwordRef = $state<HTMLInputElement | null>(null);

  const refusalText = $derived(
    refusal === "invalidCredentials" ||
      refusal === "network" ||
      refusal === "server"
      ? t(serverReasonMessages[refusal])
      : t(m.failed),
  );
  const help = $derived(t(m.resetHelp).split("{command}"));

  const goOn = () =>
    location.replace(loginDestination(location.search, location.origin));

  async function probe() {
    fetchSetupStatus().then(
      (status) => (setupRequired = status.setup_required),
      () => {},
    );
    try {
      await fetchMeProfile();
      goOn();
    } catch {
      probing = false;
    }
  }

  $effect(() => {
    void probe();
  });

  // One tick per second while throttled; the end is announced once.
  $effect(() => {
    if (remaining <= 0) return;
    const timer = setTimeout(() => {
      remaining -= 1;
      if (remaining === 0) announce(t(m.throttleEnded));
    }, 1000);
    return () => clearTimeout(timer);
  });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (busy || remaining > 0) return;
    busy = true;
    refusal = null;
    try {
      const result = await login(email, password);
      if (result.ok) return goOn();
      remaining = result.retryAfterSeconds;
      announce(
        t(m.throttledAnnouncement, {
          time: formatClockSpoken(remaining, locale.current),
        }),
      );
    } catch (error) {
      refusal =
        error instanceof ApiError ? serverReason(error).reason : "generic";
      if (refusal === "invalidCredentials") {
        password = "";
        passwordRef?.focus();
      }
    }
    busy = false;
  }
</script>

<RegionState status={probing ? "loading" : "ready"} minHeight="18rem">
  <form class="flex flex-col gap-5" novalidate onsubmit={submit}>
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
        autocomplete="current-password"
        required
        aria-describedby="{id}-help"
      />
    </div>
    {#if remaining > 0}
      <InlineMessage tone="error">
        {t(m.throttled, { time: formatClock(remaining) })}
      </InlineMessage>
    {:else if refusal}
      <InlineMessage tone="error" role="alert">{refusalText}</InlineMessage>
    {/if}
    <Button type="submit" class="w-full" {busy} disabled={remaining > 0}>
      {t(m.submit)}
    </Button>
    <p id="{id}-help" class="text-sm text-ink-muted">
      {help[0]}<code class="font-mono text-ink-strong"
        >{RESET_PASSWORD_COMMAND}</code
      >{help[1] ?? ""}
    </p>
    {#if setupRequired}
      <p class="flex flex-wrap items-center gap-x-2 text-sm text-ink-body">
        {t(m.setupNeeded)}
        <Button variant="link" size="sm" class="px-0" href={SETUP_PATH}>
          {t(m.setupLink)}
        </Button>
      </p>
    {/if}
  </form>
</RegionState>
