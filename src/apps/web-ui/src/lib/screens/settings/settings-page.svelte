<!-- Logic-free wrapper (plan D1): the shell gates, SettingsContent renders. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import SettingsContent from "./settings-content.svelte";
</script>

<AppShell screen="settings">
  {#snippet children()}<SettingsContent />{/snippet}
</AppShell>
