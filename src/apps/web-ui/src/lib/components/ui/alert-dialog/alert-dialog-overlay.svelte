<script lang="ts">
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    ...restProps
  }: AlertDialogPrimitive.OverlayProps = $props();
</script>

<AlertDialogPrimitive.Overlay
  bind:ref
  data-slot="alert-dialog-overlay"
  class={cn(
    "fixed inset-0 z-50 bg-ink-strong/30 backdrop-blur-sm reduce-transparency:bg-ink-strong/60 reduce-transparency:backdrop-blur-none motion-safe:duration-(--duration-slow) motion-safe:data-[state=open]:animate-in motion-safe:data-[state=closed]:animate-out motion-safe:data-[state=closed]:fade-out-0 motion-safe:data-[state=open]:fade-in-0",
    className,
  )}
  {...restProps}
/>
