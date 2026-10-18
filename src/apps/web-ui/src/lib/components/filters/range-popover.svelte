<!--
  @component
  Date-range filter in a popover (History, Statistics). The trigger is a pill
  that shows the current choice; the panel holds the quick presets (a radio
  group, so arrows move and Tab leaves it) and a custom from/to pair with an
  Apply button. Built on bits-ui Popover: Enter/Space opens, Escape closes
  and returns focus to the trigger, focus is trapped while open.

  Value is a preset or `{ from, to }` (api/date-range.ts), so the same
  component serves every screen; each screen passes only the presets its
  API supports. Custom dates are inclusive local dates (`YYYY-MM-DD`) from
  native date inputs, validated against the server rules before Apply.

  Props: `options` ({ value, label }[] translated presets), `bind:value`,
  `name` (radio group name, unique per page), `legend` (translated name of
  the control), `class`.
-->
<script lang="ts" generics="Preset extends string">
  import { Popover } from "bits-ui";
  import {
    MAX_CUSTOM_RANGE_DAYS,
    isCustomRange,
    type CustomRange,
    type DateRangeValue,
  } from "$lib/api/date-range";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { formatCalendarDay } from "$lib/i18n/formatting/date-time-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { cn } from "$lib/utils.js";
  import { ChipGroup } from "$lib/components/ui/chip-group/index.js";
  import { customRangeError } from "./custom-range-error";
  import { rangePopoverMessages as m } from "./range-popover.messages";

  type Props = {
    options: readonly { value: Preset; label: string }[];
    value: DateRangeValue<Preset>;
    name: string;
    legend: string;
    class?: string;
  };

  let {
    options,
    value = $bindable(),
    name,
    legend,
    class: className,
  }: Props = $props();

  let open = $state(false);
  let draft = $state<CustomRange>({ from: "", to: "" });
  let attempted = $state(false);

  const preset = $derived(isCustomRange(value) ? undefined : value);
  const error = $derived(customRangeError(draft));
  const errorText = $derived(
    !attempted || error === null
      ? ""
      : error === "missing"
        ? t(m.errorMissing)
        : error === "order"
          ? t(m.errorOrder)
          : t(m.errorTooLong, { days: MAX_CUSTOM_RANGE_DAYS }),
  );
  const currentLabel = $derived(
    isCustomRange(value)
      ? t(m.customLabel, {
          from: formatCalendarDay(value.from, locale.current),
          to: formatCalendarDay(value.to, locale.current),
        })
      : (options.find((option) => option.value === value)?.label ?? ""),
  );

  // Opening the popover starts the custom fields from the active range.
  $effect(() => {
    if (open) {
      draft = isCustomRange(value) ? { ...value } : { from: "", to: "" };
      attempted = false;
    }
  });

  function choosePreset(next: Preset | undefined) {
    if (next === undefined) return;
    value = next;
    open = false;
  }

  function applyCustom() {
    attempted = true;
    if (error !== null) return;
    value = { from: draft.from, to: draft.to };
    open = false;
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger
    aria-label={t(m.triggerLabel, { range: currentLabel })}
    class={cn(
      "glass-control focus-ring inline-flex min-h-11 max-w-full items-center gap-2 rounded-full px-4.5 py-1 text-sm font-medium text-ink-strong motion-safe:transition-colors motion-safe:duration-(--duration-fast)",
      className,
    )}
  >
    <AppIcon name="calendar-blank" size={20} />
    <span class="min-w-0 break-words">{currentLabel}</span>
    <AppIcon name="caret-down" size={16} />
  </Popover.Trigger>
  <Popover.Portal>
    <!-- Marker keeps the popover visible under the player focus-mode CSS. -->
    <div data-dialog-portal="" class="contents">
      <Popover.Content
        sideOffset={8}
        align="start"
        aria-label={legend}
        class="elevation-2 focus-ring z-50 flex w-[min(22rem,calc(100vw-2rem))] flex-col gap-4 rounded-panel border border-rim bg-surface-solid p-4"
      >
        <ChipGroup
          {options}
          {name}
          legend={t(m.presetsLegend)}
          showCheck={false}
          bind:value={() => preset, choosePreset}
        />
        <fieldset class="m-0 flex min-w-0 flex-col gap-3 border-0 p-0">
          <legend class="mb-2 text-sm font-medium text-ink-body">
            {t(m.customHeading)}
          </legend>
          <div class="flex flex-wrap gap-3">
            <label
              class="flex min-w-0 flex-1 flex-col gap-1 text-sm text-ink-body"
            >
              {t(m.from)}
              <Input
                type="date"
                bind:value={draft.from}
                aria-invalid={attempted && error !== null}
                aria-describedby={errorText ? `${name}-error` : undefined}
              />
            </label>
            <label
              class="flex min-w-0 flex-1 flex-col gap-1 text-sm text-ink-body"
            >
              {t(m.to)}
              <Input
                type="date"
                bind:value={draft.to}
                aria-invalid={attempted && error !== null}
                aria-describedby={errorText ? `${name}-error` : undefined}
              />
            </label>
          </div>
          <p
            id="{name}-error"
            role="alert"
            class="min-h-5 text-sm text-destructive"
          >
            {errorText}
          </p>
          <Button onclick={applyCustom}>{t(m.apply)}</Button>
        </fieldset>
      </Popover.Content>
    </div>
  </Popover.Portal>
</Popover.Root>
