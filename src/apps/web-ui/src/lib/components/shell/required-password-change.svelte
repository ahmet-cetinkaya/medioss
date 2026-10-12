<!--
  @component
  The forced change-password view (FR-105–FR-107). AppShell renders it in
  place of every screen, at the same URL, while the user is flagged. It
  explains why, offers PasswordChangeForm and Sign out (same confirmed flow
  as the shell). On success it re-probes the session; the gate then renders
  the original screen at the same URL.
-->
<script lang="ts">
  import PasswordChangeForm from "$lib/components/account/password-change-form.svelte";
  import { requiredPasswordMessages } from "$lib/components/account/account.messages";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import {
    actionMessages,
    screenTitleMessages,
  } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { session } from "$lib/session/session.svelte";
  import SignOut from "./sign-out.svelte";
  import UnauthenticatedLayout from "./unauthenticated-layout.svelte";

  let signOut = $state<SignOut | null>(null);
</script>

<UnauthenticatedLayout title={screenTitleMessages.changePassword}>
  <p class="text-ink-body">{t(requiredPasswordMessages.explanation)}</p>
  <PasswordChangeForm
    onSuccess={async () => {
      await session.refresh();
    }}
  />
  {#snippet footer()}
    <Button
      variant="ghost"
      onclick={(event: MouseEvent & { currentTarget: HTMLElement }) =>
        signOut?.start(event.currentTarget)}
    >
      <AppIcon name="sign-out" />
      {t(actionMessages.signOut)}
    </Button>
  {/snippet}
</UnauthenticatedLayout>
<SignOut bind:this={signOut} />
