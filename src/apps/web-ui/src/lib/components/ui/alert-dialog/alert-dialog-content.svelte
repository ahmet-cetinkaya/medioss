<script lang="ts">
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui";
  import AlertDialogPortal from "./alert-dialog-portal.svelte";
  import AlertDialogOverlay from "./alert-dialog-overlay.svelte";
  import {
    cn,
    type WithoutChild,
    type WithoutChildrenOrChild,
  } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let {
    ref = $bindable(null),
    class: className,
    portalProps,
    ...restProps
  }: WithoutChild<AlertDialogPrimitive.ContentProps> & {
    portalProps?: WithoutChildrenOrChild<
      ComponentProps<typeof AlertDialogPortal>
    >;
  } = $props();
</script>

<AlertDialogPortal {...portalProps}>
  <AlertDialogOverlay />
  <AlertDialogPrimitive.Content
    bind:ref
    data-slot="alert-dialog-content"
    class={cn(
      "glass-solid elevation-2 fixed top-[50%] left-[50%] z-50 grid max-h-[calc(100dvh-2rem)] w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-5 overflow-y-auto rounded-panel p-6 text-ink-body sm:max-w-md motion-safe:duration-(--duration-slow) motion-safe:data-[state=open]:animate-in motion-safe:data-[state=closed]:animate-out motion-safe:data-[state=closed]:fade-out-0 motion-safe:data-[state=open]:fade-in-0 motion-safe:data-[state=closed]:zoom-out-95 motion-safe:data-[state=open]:zoom-in-95",
      className,
    )}
    {...restProps}
  />
</AlertDialogPortal>
