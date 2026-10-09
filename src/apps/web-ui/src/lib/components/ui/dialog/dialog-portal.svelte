<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";

  let { ...restProps }: DialogPrimitive.PortalProps = $props();
</script>

<DialogPrimitive.Portal {...restProps}>
  <!-- data-dialog-portal marks body-level portals so the Player's focus
       mode (.player-focus-on) can hide the page UI without touching open
       dialogs. -->
  <div data-dialog-portal="" class="contents">
    <slot />
  </div>
</DialogPrimitive.Portal>
