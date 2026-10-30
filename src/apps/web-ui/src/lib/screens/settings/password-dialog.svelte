<!--
  @component
  Change-password dialog (FR-066): the shared PasswordChangeForm inside a
  dialog, opened from the Profile panel. A successful change hands the fresh
  profile to `session.setUser` (never `session.refresh`, which would unmount
  Settings) and closes the dialog; the form announces the success itself.

  Props: `open` (bindable). While the change request runs, Escape and
  outside clicks do not close the dialog (the form reports `busy`), matching
  FormDialog.
-->
<script lang="ts">
  import PasswordChangeForm from "$lib/components/account/password-change-form.svelte";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import { session } from "$lib/session/session.svelte";
  import { passwordPanelMessages as m } from "./settings.messages";

  type Props = { open: boolean };

  let { open = $bindable() }: Props = $props();
  let busy = $state(false);
</script>

<Dialog.Root bind:open>
  <Dialog.Content
    escapeKeydownBehavior={busy ? "ignore" : "close"}
    interactOutsideBehavior={busy ? "ignore" : "close"}
  >
    <Dialog.Header>
      <Dialog.Title>{t(m.heading)}</Dialog.Title>
      <Dialog.Description>{t(m.intro)}</Dialog.Description>
    </Dialog.Header>
    <PasswordChangeForm
      bind:busy
      onSuccess={(updated) => {
        session.setUser(updated);
        open = false;
      }}
    />
  </Dialog.Content>
</Dialog.Root>
