<!--
  @component
  Admin Users section (FR-073–FR-076, PB-13, PB-17). A create form that keeps
  its values on refusal; the created user's temporary password shown once in
  a read-only field with Copy (component state only: gone after a reload);
  and the user list (avatar, name, email, role, status as text) with
  Disable (confirm: immediate sign-out), Enable (no confirm), and Delete
  (confirm naming the email). Disabling yourself maps to `selfDisable`.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { avatarInitial } from "$lib/account/avatar-initial";
  import { ApiError } from "$lib/api/api-error";
  import {
    avatarUrl,
    createUser,
    deleteUser,
    fetchUsers,
    setUserStatus,
    type IdentityUser,
  } from "$lib/api/identity";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import {
    describeFailure,
    type Failure,
  } from "$lib/components/dialogs/form-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
  import { t } from "$lib/i18n/translate";
  import { session } from "$lib/session/session.svelte";
  import { adminMessages, userMessages as m } from "./admin.messages";

  const id = $props.id();

  let users = $state<IdentityUser[] | null>(null);
  let loadFailure = $state<ServerReason | null>(null);
  let request = 0;

  const status = $derived(
    loadFailure
      ? "error"
      : users
        ? users.length
          ? "ready"
          : "empty"
        : "loading",
  );
  const loadErrorText = $derived(
    loadFailure === "network" || loadFailure === "server"
      ? t(serverReasonMessages[loadFailure])
      : t(adminMessages.loadFailed),
  );

  async function load(reset = true) {
    const current = ++request;
    if (reset) {
      users = null;
      loadFailure = null;
    }
    try {
      const result = await fetchUsers();
      if (current !== request) return;
      users = result;
      loadFailure = null;
    } catch (error) {
      if (current === request)
        loadFailure =
          error instanceof ApiError ? serverReason(error).reason : "generic";
    }
  }

  onMount(() => void load());

  // Create user. The temporary password lives only in `created` (PB-13).
  let email = $state("");
  let password = $state("");
  let creating = $state(false);
  let createFailure = $state<Failure | null>(null);
  let created = $state<{ email: string; password: string } | null>(null);
  let copyState = $state<"copied" | "selected" | null>(null);
  let passwordField = $state<HTMLInputElement | null>(null);

  async function submitCreate(event: SubmitEvent) {
    event.preventDefault();
    if (creating || !email.trim() || !password) return;
    creating = true;
    createFailure = null;
    created = null;
    copyState = null;
    try {
      const user = await createUser(email.trim(), password);
      created = { email: user.email, password };
      email = "";
      password = "";
      announce(t(m.created, { email: user.email }));
      void load(false);
    } catch (error) {
      createFailure = describeFailure(error);
    } finally {
      creating = false;
    }
  }

  async function copyPassword() {
    if (!created) return;
    try {
      await navigator.clipboard.writeText(created.password);
      copyState = "copied";
    } catch {
      passwordField?.focus();
      passwordField?.select();
      copyState = "selected";
    }
  }

  // Lifecycle actions.
  let busyId = $state<number | null>(null);
  let rowFailure = $state<Failure | null>(null);
  let disableTarget = $state<IdentityUser | null>(null);
  let disableOpen = $state(false);
  let deleteTarget = $state<IdentityUser | null>(null);
  let deleteOpen = $state(false);
  let listHeading = $state<HTMLElement | null>(null);
  let failedAvatars = $state<Record<number, true>>({});

  const isSelf = (user: IdentityUser | null) =>
    user !== null && user.id === session.user?.id;

  async function enableUser(user: IdentityUser) {
    if (busyId !== null) return;
    busyId = user.id;
    rowFailure = null;
    try {
      await setUserStatus(user.id, false);
      announce(t(m.enabled, { email: user.email }));
      await load(false);
    } catch (error) {
      rowFailure = describeFailure(error);
    } finally {
      busyId = null;
    }
  }

  function askDisable(user: IdentityUser) {
    rowFailure = null;
    disableTarget = user;
    disableOpen = true;
  }

  async function runDisable() {
    const target = disableTarget;
    if (!target) return;
    await setUserStatus(target.id, true);
    announce(t(m.disabled, { email: target.email }));
    await load(false);
  }

  function askDelete(user: IdentityUser) {
    rowFailure = null;
    deleteTarget = user;
    deleteOpen = true;
  }

  async function runDelete() {
    const target = deleteTarget;
    if (!target) return;
    await deleteUser(target.id);
    announce(t(m.deleted, { email: target.email }));
    await load(false);
  }

  const roleText = (user: IdentityUser) =>
    t(user.role === "admin" ? m.roleAdmin : m.roleMember);
  const statusText = (user: IdentityUser) =>
    t(
      user.disabled
        ? m.statusDisabled
        : user.must_change_password
          ? m.statusMustChange
          : m.statusActive,
    );
</script>

<div class="flex flex-col gap-6">
  <Panel aria-labelledby="{id}-create-heading">
    {#snippet heading()}
      <h2 id="{id}-create-heading" class="text-heading text-ink-strong">
        {t(m.createHeading)}
      </h2>
    {/snippet}
    <form
      class="grid grid-cols-1 min-w-0 gap-4 md:max-w-xl"
      onsubmit={submitCreate}
      novalidate
    >
      <div class="grid grid-cols-1 gap-2">
        <Label for="{id}-email">{t(m.email)}</Label>
        <Input
          id="{id}-email"
          type="email"
          bind:value={email}
          autocomplete="off"
          spellcheck="false"
          required
        />
      </div>
      <div class="grid grid-cols-1 gap-2">
        <Label for="{id}-password">{t(m.temporaryPassword)}</Label>
        <Input
          id="{id}-password"
          bind:value={password}
          autocomplete="off"
          spellcheck="false"
          required
          aria-describedby="{id}-password-hint"
        />
        <p id="{id}-password-hint" class="text-sm text-ink-muted">
          {t(m.temporaryPasswordHint)}
        </p>
      </div>
      {#if createFailure}
        <InlineMessage tone="error" role="alert">
          {t(createFailure.message, createFailure.params)}
        </InlineMessage>
      {/if}
      <div>
        <Button
          type="submit"
          busy={creating}
          disabled={!email.trim() || !password}
        >
          {t(m.create)}
        </Button>
      </div>
    </form>

    {#if created}
      <div
        class="mt-6 grid grid-cols-1 gap-3 rounded-tile bg-surface-solid/50 p-4 md:max-w-xl"
        data-testid="created-user"
      >
        <InlineMessage tone="success">
          {t(m.created, { email: created.email })}
        </InlineMessage>
        <Label for="{id}-created-password">{t(m.temporaryPassword)}</Label>
        <div class="flex flex-wrap items-center gap-2">
          <Input
            id="{id}-created-password"
            bind:ref={passwordField}
            value={created.password}
            readonly
            autocomplete="off"
            spellcheck="false"
            class="min-w-0 flex-1 font-mono select-all"
            aria-describedby="{id}-created-note"
            onfocus={(event) => event.currentTarget.select()}
          />
          <Button variant="secondary" onclick={copyPassword}>
            {t(actionMessages.copy)}
          </Button>
        </div>
        <p id="{id}-created-note" class="text-sm text-ink-body">
          {t(m.shownOnce, { email: created.email })}
        </p>
        {#if copyState}
          <InlineMessage tone="success" role="status">
            {t(copyState === "copied" ? m.copied : m.copySelected)}
          </InlineMessage>
        {/if}
      </div>
    {/if}
  </Panel>

  <Panel aria-labelledby="{id}-list-heading">
    {#snippet heading()}
      <h2
        id="{id}-list-heading"
        bind:this={listHeading}
        tabindex="-1"
        class="text-heading text-ink-strong outline-none"
      >
        {t(m.listHeading)}
      </h2>
    {/snippet}
    {#if rowFailure}
      <InlineMessage tone="error" role="alert" class="mb-4">
        {t(rowFailure.message, rowFailure.params)}
      </InlineMessage>
    {/if}
    <RegionState
      {status}
      minHeight="12rem"
      errorMessage={loadErrorText}
      onRetry={() => void load()}
      emptyTitle={t(m.noUsers)}
    >
      <ul class="grid grid-cols-1 gap-3">
        {#each users ?? [] as user (user.id)}
          {@const initial = avatarInitial(user.display_name)}
          <li
            class="flex min-w-0 flex-wrap items-center gap-4 rounded-tile bg-surface-solid/50 p-4"
          >
            <span
              class="grid size-11 shrink-0 place-items-center overflow-hidden rounded-full border border-rim bg-surface-solid text-ink-strong"
            >
              {#if user.has_avatar && !failedAvatars[user.id]}
                <img
                  src={avatarUrl(user.id, session.avatarVersion)}
                  alt=""
                  class="size-full object-cover"
                  onerror={() => (failedAvatars[user.id] = true)}
                />
              {:else if initial}
                <span aria-hidden="true" class="font-semibold">{initial}</span>
              {:else}
                <AppIcon name="user" />
              {/if}
            </span>
            <div class="grid grid-cols-1 min-w-0 flex-1 gap-0.5">
              <span class="font-medium break-words text-ink-strong">
                {user.display_name ?? user.email}
              </span>
              {#if user.display_name}
                <span class="text-sm break-all text-ink-body">{user.email}</span
                >
              {/if}
              <span class="text-sm text-ink-muted">
                {t(m.roleStatus, {
                  role: roleText(user),
                  status: statusText(user),
                })}
              </span>
            </div>
            <div class="flex flex-wrap gap-2">
              {#if user.disabled}
                <Button
                  variant="secondary"
                  size="sm"
                  busy={busyId === user.id}
                  aria-label={t(m.enableNamed, { email: user.email })}
                  onclick={() => void enableUser(user)}
                >
                  {t(m.enable)}
                </Button>
              {:else}
                <Button
                  variant="secondary"
                  size="sm"
                  disabled={busyId === user.id}
                  aria-label={t(m.disableNamed, { email: user.email })}
                  onclick={() => askDisable(user)}
                >
                  {t(m.disable)}
                </Button>
              {/if}
              <Button
                variant="secondary"
                size="sm"
                disabled={busyId === user.id}
                aria-label={t(m.deleteNamed, { email: user.email })}
                onclick={() => askDelete(user)}
              >
                <AppIcon name="trash" size={16} />
                {t(actionMessages.delete)}
              </Button>
            </div>
          </li>
        {/each}
      </ul>
    </RegionState>
  </Panel>
</div>

<ConfirmDialog
  bind:open={disableOpen}
  title={t(m.disableTitle)}
  itemName={disableTarget?.email}
  consequence={t(m.disableConsequence)}
  confirmLabel={t(m.disableConfirm)}
  destructive
  onConfirm={runDisable}
  reasonContext={{ selfDisable: isSelf(disableTarget) }}
/>

<ConfirmDialog
  bind:open={deleteOpen}
  title={t(m.deleteTitle)}
  itemName={deleteTarget?.email}
  consequence={t(m.deleteConsequence)}
  confirmLabel={t(m.deleteConfirm)}
  destructive
  onConfirm={runDelete}
  fallbackFocus={() => listHeading}
/>
