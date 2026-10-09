<!-- Logic-free wrapper (plan D1): the unauthenticated layout, LoginContent loads. -->
<script lang="ts">
  import UnauthenticatedLayout from "$lib/components/shell/unauthenticated-layout.svelte";
  import { screenTitleMessages } from "$lib/i18n/common.messages";
  import LoginContent from "./login-content.svelte";
</script>

<UnauthenticatedLayout title={screenTitleMessages.login}>
  <LoginContent />
</UnauthenticatedLayout>
