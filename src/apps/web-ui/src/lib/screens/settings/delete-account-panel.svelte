<!--
  @component
  Settings "Delete account" panel (FR-070, PB-12), set apart as destructive.

  The Delete button stays disabled until the exact account email is typed;
  it then asks in the shared ConfirmDialog. Success goes to Login with no
  `next`. The last-active-admin refusal closes the dialog and shows its
  message in this panel; the user stays signed in. Other refusals stay in
  the dialog with their reason.

  Props: `user` (the signed-in user).
-->
<script lang="ts">
  import { failureMessage, reasonOf } from "$lib/api/failure";
  import { deleteOwnAccount } from "$lib/api/identity";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import { goToLogin } from "$lib/navigation/login-redirect";
  import type { SessionUser } from "$lib/session/gate-decision";
  import { deleteAccountMessages as m } from "./settings.messages";

  type Props = { user: SessionUser };

  let { user }: Props = $props();

  const id = $props.id();

  let typedEmail = $state("");
  let open = $state(false);
  let failure = $state<ReturnType<typeof reasonOf> | null>(null);

  const matches = $derived(typedEmail === user.email);

  async function runDelete() {
    failure = null;
    try {
      await deleteOwnAccount();
    } catch (error) {
      const refused = reasonOf(error);
      if (refused !== "lastActiveAdmin") throw error;
      failure = refused;
      return;
    }
    goToLogin();
  }
</script>

<Panel aria-labelledby="{id}-heading" emphasis="destructive">
  {#snippet heading()}
    <h2 id="{id}-heading" class="text-heading text-ink-strong">
      {t(m.heading)}
    </h2>
    <p class="mt-1 text-sm text-ink-muted">{t(m.consequence)}</p>
  {/snippet}

  <div class="flex flex-col gap-2">
    <label for="{id}-email" class="text-sm font-medium text-ink-strong">
      {t(m.typeEmail, { email: user.email })}
    </label>
    <Input
      id="{id}-email"
      inputmode="email"
      bind:value={typedEmail}
      autocomplete="off"
      spellcheck={false}
    />
    {#if failure}
      <InlineMessage tone="error" role="alert">
        {t(
          failureMessage(failure, m.failed, {
            specificReasons: ["lastActiveAdmin"],
          }).message,
        )}
      </InlineMessage>
    {/if}
    <div class="mt-2">
      <Button
        variant="destructive"
        disabled={!matches}
        onclick={() => (open = true)}
      >
        <AppIcon name="trash" size={18} />
        {t(m.action)}
      </Button>
    </div>
  </div>
</Panel>

<ConfirmDialog
  bind:open
  title={t(m.dialogTitle)}
  itemName={user.email}
  consequence={t(m.consequence)}
  confirmLabel={t(m.action)}
  destructive
  onConfirm={runDelete}
  errorMessage={(error) => failureMessage(reasonOf(error), m.failed).message}
/>
