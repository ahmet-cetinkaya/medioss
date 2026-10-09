<!--
  @component
  Shown in place of an admin screen's content for members (FR-071). The
  content never mounts, so no admin request is made.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import Panel from "$lib/components/ui/panel/panel.svelte";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { ROUTES } from "$lib/navigation/routes";
  import { shellMessages } from "./shell.messages";

  const headingId = $props.id();
</script>

<Panel aria-labelledby={headingId} class="flex flex-col items-start gap-3">
  <h2 id={headingId} class="text-xl font-semibold text-ink-strong">
    {t(shellMessages.adminsOnlyTitle)}
  </h2>
  <p class="text-ink-body">{t(shellMessages.adminsOnlyText)}</p>
  <Button variant="secondary" href={ROUTES.home.path}>
    {t(actionMessages.backToHome)}
  </Button>
</Panel>
