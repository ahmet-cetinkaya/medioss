<!--
  @component
  The next action of an empty Home region (FR-028): the Library link, or,
  when the library has no content, the Admin link for administrators only.
  Render it only where `hasStartAction()` is true (members get no action when
  there is no content).
-->
<script lang="ts" module>
  import { session } from "$lib/session/session.svelte";
  import { homeLibrary } from "./home-library.svelte";

  export const hasStartAction = (): boolean =>
    homeLibrary.empty !== true || session.isAdmin;
</script>

<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { t } from "$lib/i18n/translate";
  import { ROUTES } from "$lib/navigation/routes";
  import { homeMessages as m } from "./home.messages";
</script>

{#if homeLibrary.empty === true}
  {#if session.isAdmin}
    <Button variant="secondary" href={ROUTES.admin.path}>
      {t(m.openAdmin)}
    </Button>
  {/if}
{:else}
  <Button variant="secondary" href={ROUTES.library.path}>
    {t(m.openLibrary)}
  </Button>
{/if}
