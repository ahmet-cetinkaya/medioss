<!--
  @component
  Switch: a native checkbox with `role="switch"`, so Space toggles it and
  assistive technology reports on/off. A visible "On" / "Off" text shows the
  state beside the track, not color alone (FR-092). The whole row is a 44 px
  label target.

  Props: `bind:checked`, `label` (translated accessible name, shown),
  `class`, and every native input attribute (`disabled`, `name`, `id`,
  `onchange`, ...).
-->
<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { t } from "$lib/i18n/translate";
  import { cn } from "$lib/utils.js";
  import { toggleMessages } from "$lib/components/feedback/feedback.messages";

  type Props = Omit<HTMLInputAttributes, "type" | "role" | "checked"> & {
    checked?: boolean;
    label: string;
  };

  let {
    checked = $bindable(false),
    label,
    class: className,
    ...restProps
  }: Props = $props();
</script>

<label
  class={cn(
    "inline-flex min-h-11 max-w-full cursor-pointer items-center gap-3 text-sm text-ink-strong select-none has-disabled:cursor-not-allowed has-disabled:opacity-60",
    className,
  )}
>
  <input
    type="checkbox"
    role="switch"
    class="peer sr-only"
    bind:checked
    {...restProps}
  />
  <span class="min-w-0 break-words">{label}</span>
  <span
    aria-hidden="true"
    class={cn(
      "relative inline-flex h-7 w-12 shrink-0 items-center rounded-full border peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-accent-strong motion-safe:transition-colors motion-safe:duration-(--duration-fast)",
      checked
        ? "border-accent-fill bg-accent-fill"
        : "glass-control",
    )}
  >
    <span
      class={cn(
        "size-5 rounded-full motion-safe:transition-transform motion-safe:duration-(--duration-fast)",
        checked
          ? "translate-x-6 bg-surface-solid"
          : "translate-x-0.5 bg-control-border",
      )}
    ></span>
  </span>
  <span aria-hidden="true" class="min-w-8 text-ink-body">
    {checked ? t(toggleMessages.on) : t(toggleMessages.off)}
  </span>
</label>
