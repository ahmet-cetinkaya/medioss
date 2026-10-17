import { announce } from "$lib/components/feedback/announcer.svelte";
import { t } from "$lib/i18n/locale.svelte";

/** Announce a translated message to assistive technology. */
export const say = (
  message: Parameters<typeof t>[0],
  params?: Parameters<typeof t>[1],
) => announce(t(message, params));
