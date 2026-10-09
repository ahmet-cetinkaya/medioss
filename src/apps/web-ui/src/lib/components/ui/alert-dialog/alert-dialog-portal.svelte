<script lang="ts">
  import { AlertDialog as AlertDialogPrimitive } from "bits-ui";

  let { ...restProps }: AlertDialogPrimitive.PortalProps = $props();
</script>

<!-- data-dialog-portal marks body-level portals so the Player's focus
     mode (.player-focus-on) can hide the page UI without touching open
     dialogs. -->
<AlertDialogPrimitive.Portal {...restProps}>
  <!-- data-dialog-portal marks body-level portals so the Player's focus
       mode (.player-focus-on) can hide the page UI without touching open
       dialogs. -->
  <div data-dialog-portal="" class="contents">
    <slot />
  </div>
</AlertDialogPrimitive.Portal>
