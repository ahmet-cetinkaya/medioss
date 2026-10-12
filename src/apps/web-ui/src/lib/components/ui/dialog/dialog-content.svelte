<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import DialogPortal from "./dialog-portal.svelte";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import type { Snippet } from "svelte";
  import * as Dialog from "./index.js";
  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let {
    ref = $bindable(null),
    class: className,
    portalProps,
    children,
    showCloseButton = true,
    closeLabel,
    ...restProps
  }: WithoutChildrenOrChild<DialogPrimitive.ContentProps> & {
    portalProps?: WithoutChildrenOrChild<ComponentProps<typeof DialogPortal>>;
    children: Snippet;
    showCloseButton?: boolean;
    /** Accessible name of the close button; defaults to actionMessages.close. */
    closeLabel?: string;
  } = $props();
</script>

<DialogPortal {...portalProps}>
  <Dialog.Overlay />
  <DialogPrimitive.Content
    bind:ref
    data-slot="dialog-content"
    class={cn(
      "glass-solid elevation-2 fixed top-[50%] left-[50%] z-50 grid max-h-[calc(100dvh-2rem)] w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-5 overflow-y-auto rounded-panel p-6 text-ink-body sm:max-w-md motion-safe:duration-(--duration-slow) motion-safe:data-[state=open]:animate-in motion-safe:data-[state=closed]:animate-out motion-safe:data-[state=closed]:fade-out-0 motion-safe:data-[state=open]:fade-in-0 motion-safe:data-[state=closed]:zoom-out-95 motion-safe:data-[state=open]:zoom-in-95",
      className,
    )}
    {...restProps}
  >
    {@render children?.()}
    {#if showCloseButton}
      <DialogPrimitive.Close
        class={cn(
          buttonVariants({ variant: "ghost", size: "icon-sm" }),
          "absolute end-3 top-3",
        )}
        aria-label={closeLabel ?? t(actionMessages.close)}
      >
        <XIcon size={20} aria-hidden="true" />
      </DialogPrimitive.Close>
    {/if}
  </DialogPrimitive.Content>
</DialogPortal>
