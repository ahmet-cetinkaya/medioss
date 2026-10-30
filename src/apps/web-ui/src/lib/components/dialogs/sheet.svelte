<!--
  @component
  Bottom sheet (plan D6) for the mobile "More" menu (FR-087): a modal Dialog
  pinned to the bottom edge. Focus moves to the first control in `children`,
  stays inside, Escape or an outside click closes it, and focus returns to the
  opener (FR-017).

  Props: `open` (bindable), `title` (the accessible name), `hideTitle`
  (visually hide the title), `children`, `closeLabel?` (default "Close"),
  `returnFocusTo?` (focus target on close; default: the element focused when
  the sheet opened), `fallbackFocus?` (element or getter used when that
  target was removed), `onCloseAutoFocus?` (runs first; call
  `event.preventDefault()` to skip the focus return, e.g. when closing the
  sheet hands off to a ConfirmDialog), `class`.
-->
<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import type { Snippet } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { IconButton } from "$lib/components/ui/icon-button/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import { cn } from "$lib/utils.js";
  import { restoreFocus, type FocusSource } from "./return-focus";

  type Props = {
    open?: boolean;
    title: string;
    hideTitle?: boolean;
    children: Snippet;
    closeLabel?: string;
    returnFocusTo?: HTMLElement | null;
    fallbackFocus?: FocusSource<HTMLElement>;
    onCloseAutoFocus?: (event: Event) => void;
    class?: string;
  };

  let {
    open = $bindable(false),
    title,
    hideTitle = false,
    children,
    closeLabel,
    returnFocusTo,
    fallbackFocus,
    onCloseAutoFocus,
    class: className,
  }: Props = $props();

  let opener: HTMLElement | null = null;
</script>

<Dialog.Root bind:open>
  <Dialog.Portal>
    <Dialog.Overlay />
    <DialogPrimitive.Content
      data-slot="sheet-content"
      class={cn(
        "glass-solid elevation-2 fixed inset-x-0 bottom-0 z-50 grid max-h-[85dvh] gap-4 overflow-y-auto rounded-t-panel px-6 pt-6 pb-[max(1.5rem,var(--safe-area-bottom))] text-ink-body motion-safe:duration-(--duration-slow) motion-safe:data-[state=open]:animate-in motion-safe:data-[state=closed]:animate-out motion-safe:data-[state=open]:slide-in-from-bottom motion-safe:data-[state=closed]:slide-out-to-bottom",
        className,
      )}
      onOpenAutoFocus={() => {
        opener =
          document.activeElement instanceof HTMLElement
            ? document.activeElement
            : null;
      }}
      onCloseAutoFocus={(event) => {
        onCloseAutoFocus?.(event);
        if (event.defaultPrevented) return;
        restoreFocus(event, returnFocusTo ?? opener, fallbackFocus);
      }}
    >
      <Dialog.Title class={hideTitle ? "sr-only" : "min-h-11 pe-14"}>
        {title}
      </Dialog.Title>
      {@render children()}
      <IconButton
        class="absolute end-3 top-3"
        label={closeLabel ?? t(actionMessages.close)}
        icon="x"
        onclick={() => (open = false)}
      />
    </DialogPrimitive.Content>
  </Dialog.Portal>
</Dialog.Root>
