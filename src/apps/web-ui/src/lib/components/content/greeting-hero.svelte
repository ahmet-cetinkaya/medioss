<!--
  @component
  Greeting hero (DESIGN_SYSTEM §5.8), horizontal: the user's profile photo
  beside the greeting. The greeting is the screen's real h1 (the caller owns
  the time-of-day wording, FR-022/023). The photo is decorative (`alt=""`);
  fallback chain as the account button: avatar image, Turkish-cased initial,
  generic User icon.

  Props: `lead`, optional `name` (weight 700, joined to `lead` with a space),
  optional `tail` (appended as-is), `user` (session user or null),
  `avatarVersion`, `class`.
-->
<script lang="ts">
  import { avatarInitial } from "$lib/account/avatar-initial";
  import { avatarUrl } from "$lib/api/identity";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import type { SessionUser } from "$lib/session/gate-decision";
  import { cn } from "$lib/utils.js";

  type Props = {
    lead: string;
    name?: string | null;
    tail?: string;
    user: SessionUser | null;
    avatarVersion: number;
    class?: string;
  };

  let {
    lead,
    name,
    tail,
    user,
    avatarVersion,
    class: className,
  }: Props = $props();

  const src = $derived(
    user?.has_avatar ? avatarUrl(user.id, avatarVersion) : null,
  );
  // Remembers which URL failed, so a new avatar version gets a fresh try.
  let failedSrc = $state<string | null>(null);
  const initial = $derived(avatarInitial(user?.display_name));
</script>

<div class={cn("flex min-w-0 items-center gap-4 md:gap-5", className)}>
  <span
    class="grid size-16 shrink-0 place-items-center overflow-hidden rounded-full border border-rim bg-surface-solid text-ink-strong md:size-20"
  >
    {#if src && failedSrc !== src}
      <img
        {src}
        alt=""
        class="size-full object-cover"
        onerror={() => (failedSrc = src)}
      />
    {:else if initial}
      <span aria-hidden="true" class="text-2xl font-semibold">{initial}</span>
    {:else}
      <AppIcon name="user" />
    {/if}
  </span>
  <h1
    class="min-w-0 text-[1.625rem] leading-[1.2] font-normal break-words text-ink-strong sm:text-greeting"
  >
    {lead}{#if name}{" "}<span class="font-bold">{name}</span>{/if}{tail}
  </h1>
</div>
