<!--
  @component
  Admin content (FR-072): section chips (Users, Meditations, Soundscape),
  each with its own address — /admin/ (meditations), /admin/soundscape/, /admin/users/
  (plan D10). Selecting a chip navigates (pushState), so the browser Back
  moves between sections. Mounted only for admins by the AppShell gate
  (FR-071).
-->
<script lang="ts">
  import { ChipGroup } from "$lib/components/ui/chip-group/index.js";
  import { t } from "$lib/i18n/translate";
  import { adminMessages as m } from "./admin.messages";
  import {
    adminSectionPath,
    parseAdminSection,
    type AdminSection,
  } from "./admin-section";
  import MeditationsSection from "./meditations-section.svelte";
  import SoundscapeSection from "./soundscape-section.svelte";
  import UsersSection from "./users-section.svelte";

  let section = $state<AdminSection>(parseAdminSection(location.pathname));

  // Also follow in-app navigations that only swap the island's island
  // siblings (ClientRouter swaps the page; this covers direct pushes).
  $effect(() => {
    const onPop = () => (section = parseAdminSection(location.pathname));
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  });

  const options = $derived([
    { value: "meditations" as const, label: t(m.sectionMeditations) },
    { value: "soundscape" as const, label: t(m.sectionSoundscape) },
    { value: "users" as const, label: t(m.sectionUsers) },
  ]);

  $effect(() => {
    const target = adminSectionPath(section);
    if (location.pathname !== target)
      history.pushState(history.state, "", target);
  });
</script>

<div class="flex flex-col gap-6">
  <ChipGroup
    {options}
    bind:value={section}
    name="admin-section"
    legend={t(m.sectionLegend)}
    legendHidden
    showCheck={false}
  />

  {#if section === "users"}
    <UsersSection />
  {:else if section === "meditations"}
    <MeditationsSection />
  {:else}
    <SoundscapeSection />
  {/if}
</div>
