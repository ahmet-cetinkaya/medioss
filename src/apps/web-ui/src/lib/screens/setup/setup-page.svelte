<!-- Logic-free wrapper (plan D1): the unauthenticated layout, SetupContent loads. -->
<script lang="ts">
  import UnauthenticatedLayout from "$lib/components/shell/unauthenticated-layout.svelte";
  import { screenTitleMessages } from "$lib/i18n/common.messages";
  import SetupContent from "./setup-content.svelte";
</script>

<UnauthenticatedLayout title={screenTitleMessages.setup}>
  <SetupContent />
</UnauthenticatedLayout>
