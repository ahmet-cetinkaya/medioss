<!--
  @component
  The Sign out flow (plan D10, FR-007/FR-008). Mount once; call `start(trigger)`
  from any Sign out control (bind:this). It asks for confirmation, then posts
  the logout:

  - 204, or 401 (the session had already ended): `goToLogin()`, Login with no
    `next` and no error. The dialog stays busy until the page unloads.
  - Any other failure: the user stays on the screen; the dialog stays open
    and shows "Sign-out did not complete" (role="alert"), and the announcer
    repeats it. Only this fixed message is shown, never server text.

  Focus returns to `trigger` when the dialog closes (for the More sheet, the
  More button).
-->
<script lang="ts">
  import { logout } from "$lib/api/identity";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { goToLogin } from "$lib/navigation/login-redirect";
  import { signOutMessages } from "./shell.messages";

  let open = $state(false);
  let returnFocusTo = $state<HTMLElement | null>(null);

  /** Opens the confirmation; focus returns to `trigger` on close. */
  export function start(trigger: HTMLElement | null = null): void {
    returnFocusTo = trigger;
    open = true;
  }

  async function confirmSignOut(): Promise<void> {
    try {
      await logout();
    } catch (error) {
      announce(t(signOutMessages.failed));
      throw error;
    }
    goToLogin();
    // Leaving the page: keep the dialog busy instead of closing it.
    await new Promise<never>(() => {});
  }
</script>

<ConfirmDialog
  bind:open
  title={t(signOutMessages.title)}
  consequence={t(signOutMessages.consequence)}
  confirmLabel={t(actionMessages.signOut)}
  onConfirm={confirmSignOut}
  {returnFocusTo}
  errorMessage={() => signOutMessages.failed}
/>
