<!-- Logic-free wrapper (plan D1): the shell gates admins, AdminContent loads. -->
<script lang="ts">
  import AppShell from "$lib/components/shell/app-shell.svelte";
  import AdminContent from "./admin-content.svelte";
</script>

<AppShell screen="admin" requires="admin">
  {#snippet children()}<AdminContent />{/snippet}
</AppShell>
