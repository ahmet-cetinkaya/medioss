<!--
  @component
  Settings content (FR-066): stacked panels in this order: Profile (with the
  change-password dialog button), Language, Your data, and Delete account (destructive). The shell mounts it
  only for a signed-in user, so `session.user` is known; password and profile
  changes go through `session.setUser`, never `session.refresh` (a refresh
  would put the shell back into its loading state and unmount this screen).
-->
<script lang="ts">
  import LanguageSwitcher from "$lib/components/i18n/language-switcher.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import { session } from "$lib/session/session.svelte";
  import DataPanel from "./data-panel.svelte";
  import DeleteAccountPanel from "./delete-account-panel.svelte";
  import ProfilePanel from "./profile-panel.svelte";
  import { languagePanelMessages } from "./settings.messages";

  const user = $derived(session.user);
</script>

{#if user}
  <div class="mx-auto flex w-full max-w-3xl flex-col gap-6">
    <ProfilePanel {user} />

    <Panel aria-labelledby="settings-language-heading">
      {#snippet heading()}
        <h2 id="settings-language-heading" class="text-heading text-ink-strong">
          {t(languagePanelMessages.heading)}
        </h2>
        <p class="mt-1 text-sm text-ink-muted">
          {t(languagePanelMessages.intro)}
        </p>
      {/snippet}
      <LanguageSwitcher legendHidden />
    </Panel>

    <DataPanel />

    <DeleteAccountPanel {user} />
  </div>
{/if}
