<!--
  @component
  Settings Profile panel (FR-067, FR-068, PB-12).

  - Avatar preview: the picture when the user has one (if it fails to load,
    the initial), else the Turkish-cased initial of the display name, else
    the User icon.
  - Upload: `avatarFileProblem` refuses other types or more than 2 MB before
    any request, stating the types and the limit. Remove asks first in the
    shared ConfirmDialog.
  - Display name: at most 64 characters, the email as placeholder; saving an
    empty value clears it.
  - Every change hands the fresh profile to `session.setUser`, so the account
    button updates (avatar version bump included).

  Props: `user` (the signed-in user).
-->
<script lang="ts">
  import { avatarInitial } from "$lib/account/avatar-initial";
  import { failureMessage, reasonOf } from "$lib/api/failure";
  import {
    avatarUrl,
    fetchMeProfile,
    removeAvatar,
    updateDisplayName,
    uploadAvatar,
  } from "$lib/api/identity";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import type { SessionUser } from "$lib/session/gate-decision";
  import { session } from "$lib/session/session.svelte";
  import { AVATAR_TYPES, avatarFileProblem } from "./avatar-file";
  import PasswordDialog from "./password-dialog.svelte";
  import {
    passwordPanelMessages,
    profileMessages as m,
  } from "./settings.messages";

  type Props = { user: SessionUser };

  let { user }: Props = $props();

  const id = $props.id();
  const DISPLAY_NAME_MAX = 64;

  let displayName = $state(session.user?.display_name ?? "");
  let passwordOpen = $state(false);
  let nameBusy = $state(false);
  let nameSaved = $state(false);
  let nameFailure = $state<ReturnType<typeof reasonOf> | null>(null);
  let uploadBusy = $state(false);
  let avatarFailure = $state<ReturnType<typeof reasonOf> | null>(null);
  let removeOpen = $state(false);
  let fileInput = $state<HTMLInputElement | null>(null);
  let uploadButton = $state<HTMLElement | null>(null);

  const src = $derived(
    user.has_avatar ? avatarUrl(user.id, session.avatarVersion) : null,
  );
  // Remembers which URL failed, so a new avatar version gets a fresh try.
  let failedSrc = $state<string | null>(null);
  const initial = $derived(avatarInitial(user.display_name));

  /** The avatar endpoints return nothing: re-read the profile for the shell. */
  async function syncProfile(fallback: Partial<SessionUser>) {
    try {
      session.setUser(await fetchMeProfile());
    } catch {
      session.setUser({ ...user, ...fallback });
    }
  }

  async function onAvatarChosen(
    event: Event & { currentTarget: HTMLInputElement },
  ) {
    const input = event.currentTarget;
    const file = input.files?.[0];
    input.value = "";
    if (!file || uploadBusy) return;
    avatarFailure = null;
    const problem = avatarFileProblem(file);
    if (problem) {
      avatarFailure = {
        message: problem === "type" ? m.wrongType : m.tooLarge,
      };
      return;
    }
    uploadBusy = true;
    try {
      await uploadAvatar(file);
      await syncProfile({ has_avatar: true });
      announce(t(m.uploaded));
    } catch (error) {
      avatarFailure = reasonOf(error);
    } finally {
      uploadBusy = false;
    }
  }

  async function runRemove() {
    await removeAvatar();
    avatarFailure = null;
    await syncProfile({ has_avatar: false });
    announce(t(m.removed));
  }

  async function saveName(event: SubmitEvent) {
    event.preventDefault();
    if (nameBusy) return;
    nameBusy = true;
    nameSaved = false;
    nameFailure = null;
    try {
      const me = await updateDisplayName(displayName.trim());
      session.setUser(me);
      displayName = me.display_name ?? "";
      nameSaved = true;
      announce(t(m.nameSaved));
    } catch (error) {
      nameFailure = reasonOf(error);
    } finally {
      nameBusy = false;
    }
  }
</script>

<Panel aria-labelledby="{id}-heading">
  {#snippet heading()}
    <h2 id="{id}-heading" class="text-heading text-ink-strong">
      {t(m.heading)}
    </h2>
    <p class="mt-1 text-sm text-ink-muted">{t(m.intro)}</p>
  {/snippet}

  <div class="flex flex-col gap-6">
    <div class="flex flex-col gap-4 sm:flex-row sm:items-center">
      <div
        class="elevation-1 grid size-20 shrink-0 place-items-center overflow-hidden rounded-full border border-rim bg-surface-solid text-ink-strong"
      >
        {#if src && failedSrc !== src}
          <img
            {src}
            alt={t(m.pictureAlt)}
            class="size-full object-cover"
            onerror={() => (failedSrc = src)}
          />
        {:else if initial}
          <span aria-hidden="true" class="text-2xl font-semibold"
            >{initial}</span
          >
        {:else}
          <AppIcon name="user" size={32} />
        {/if}
      </div>
      <div class="flex flex-col gap-2">
        <div class="flex flex-wrap gap-2">
          <input
            bind:this={fileInput}
            type="file"
            class="sr-only"
            tabindex="-1"
            aria-hidden="true"
            accept={AVATAR_TYPES.join(",")}
            onchange={onAvatarChosen}
          />
          <Button
            bind:ref={uploadButton}
            variant="secondary"
            busy={uploadBusy}
            aria-describedby="{id}-rules"
            onclick={() => fileInput?.click()}
          >
            <AppIcon name="upload-simple" size={18} />
            {t(m.uploadPicture)}
          </Button>
          {#if user.has_avatar}
            <Button
              variant="ghost"
              disabled={uploadBusy}
              onclick={() => (removeOpen = true)}
            >
              <AppIcon name="trash" size={18} />
              {t(m.removePicture)}
            </Button>
          {/if}
        </div>
        <p id="{id}-rules" class="text-sm text-ink-muted">
          {t(m.pictureRules)}
        </p>
        {#if avatarFailure}
          <InlineMessage tone="error" role="alert">
            {t(failureMessage(avatarFailure, m.uploadFailed).message)}
          </InlineMessage>
        {/if}
      </div>
    </div>

    <form class="flex flex-col gap-2" novalidate onsubmit={saveName}>
      <label for="{id}-name" class="text-sm font-medium text-ink-strong">
        {t(m.displayName)}
      </label>
      <Input
        id="{id}-name"
        bind:value={displayName}
        maxlength={DISPLAY_NAME_MAX}
        placeholder={user.email}
        autocomplete="nickname"
        aria-describedby="{id}-name-hint"
        oninput={() => (nameSaved = false)}
      />
      <p id="{id}-name-hint" class="text-sm text-ink-muted">
        {t(m.displayNameHint)}
      </p>
      {#if nameFailure}
        <InlineMessage tone="error" role="alert">
          {t(failureMessage(nameFailure, m.nameFailed).message)}
        </InlineMessage>
      {:else if nameSaved}
        <InlineMessage tone="success">{t(m.nameSaved)}</InlineMessage>
      {/if}
      <div class="mt-2 flex flex-wrap gap-2">
        <Button type="submit" busy={nameBusy}>{t(m.saveName)}</Button>
        <Button variant="secondary" onclick={() => (passwordOpen = true)}>
          <AppIcon name="shield-check" />
          {t(passwordPanelMessages.change)}
        </Button>
      </div>
    </form>
  </div>
</Panel>

<PasswordDialog bind:open={passwordOpen} />

<ConfirmDialog
  bind:open={removeOpen}
  title={t(m.removeTitle)}
  consequence={t(m.removeConsequence)}
  confirmLabel={t(m.removePicture)}
  destructive
  onConfirm={runRemove}
  fallbackFocus={() => uploadButton}
  errorMessage={(error) =>
    failureMessage(reasonOf(error), m.removeFailed).message}
/>
