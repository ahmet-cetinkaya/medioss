<!--
  @component
  Icon plus text feedback. The icon carries a translated tone name ("Error",
  "Success", "Note"), so tone is never conveyed by color alone (FR-092).

  Props: `tone` ("error" | "success" | "info", default "info"), `role`
  ("status" | "alert", optional; omit for static text), `class`, `children`.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import type { IconName } from "$lib/components/icons/icon-name";
  import { t } from "$lib/i18n/translate";
  import { cn } from "$lib/utils.js";
  import { toneMessages } from "./feedback.messages";

  type Tone = "error" | "success" | "info";

  type Props = {
    tone?: Tone;
    role?: "status" | "alert";
    class?: string;
    children: Snippet;
  };

  let { tone = "info", role, class: className, children }: Props = $props();

  const ICONS: Record<Tone, IconName> = {
    error: "warning-circle",
    success: "check-circle",
    info: "warning-circle",
  };

  const TONE_CLASSES: Record<Tone, string> = {
    error: "text-destructive",
    success: "text-success",
    info: "text-ink-body",
  };
</script>

<div
  {role}
  class={cn("flex items-start gap-2 text-sm", TONE_CLASSES[tone], className)}
>
  <AppIcon
    name={ICONS[tone]}
    size={16}
    label={t(toneMessages[tone])}
    class="mt-0.5 shrink-0"
  />
  <div class="min-w-0">{@render children()}</div>
</div>
