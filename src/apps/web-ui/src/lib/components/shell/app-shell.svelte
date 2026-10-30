<!--
  @component
  The app shell (plan D1/D2; DESIGN_SYSTEM §4.1, §4.3). The only place screen
  content mounts. Screen wrappers render:

    <AppShell screen="stats">
      {#snippet children()}<StatsContent />{/snippet}
    </AppShell>

  Props:
  - `screen` (ScreenKey): nav current item, h1, and document title.
  - `title?`: runtime h1 override (already translated or user content). An
    explicit `null` suppresses the h1 (Home: its greeting is the in-content h1).
  - `documentTitle?`: runtime document-title override; separate from `title`
    when a screen suppresses its shell h1.
  - `requires?` ("user" | "admin"): screens whose route requires an admin are
    always gated as "admin", whatever is passed.
  - `search?`: shows the Library header search (Library only).
  - `aside?` (snippet; Home only): a labeled `<aside>` beside main at
    >= 1024 px and below it under 1024 px (FR-030).
  - `children` (snippet): the screen content. It mounts only when the gate
    says "render", so content never loads for anonymous, flagged, or
    (on admin screens) member users.

  Gate (`decideGate` after `session.load()`): skeleton while loading;
  redirect to Login (keeps the skeleton) when signed out; error with Retry
  (never Login, FR-011); the change-password view in place, without
  navigation, when flagged (FR-105); the shell with an admins-only state for
  members on admin screens (FR-071); otherwise the content.

  Layout, CSS-only (no matchMedia): >= 1280 px labeled sidebar; 768–1279 px
  icon rail; < 768 px sticky top bar and fixed bottom bar with More. Hidden
  surfaces use `display: none`, so each tier exposes one nav surface. DOM
  order: skip link, sidebar, rail, bottom bar, header (h1, search, account),
  main, aside.

  Also: `<html lang>` and document title effects, one LiveAnnouncer, one
  Sign out flow, and `pageshow` revalidation (Back after sign-out shows Login).
-->
<script lang="ts">
  import { onMount, tick, type Snippet } from "svelte";
  import { ApiError } from "$lib/api/api-error";
  import { serverReason } from "$lib/api/server-reason";
  import LiveAnnouncer from "$lib/components/feedback/live-announcer.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { brandMessages, stateMessages } from "$lib/i18n/common.messages";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
  import { redirectToLogin } from "$lib/navigation/login-redirect";
  import { ROUTES, type ScreenKey } from "$lib/navigation/routes";
  import { decideGate, type GateRequirement } from "$lib/session/gate-decision";
  import { session } from "$lib/session/session.svelte";
  import AdminsOnly from "./admins-only.svelte";
  import BottomBar from "./bottom-bar.svelte";
  import IconRail from "./icon-rail.svelte";
  import RequiredPasswordChange from "./required-password-change.svelte";
  import ShellHeader from "./shell-header.svelte";
  import SidebarNav from "./sidebar-nav.svelte";
  import SignOut from "./sign-out.svelte";
  import SkipLink from "./skip-link.svelte";
  import { shellMessages } from "./shell.messages";

  type Props = {
    screen: ScreenKey;
    title?: string | null;
    /** Extra controls on the header row next to the search (Library's filter). */
    headerActions?: Snippet;
    documentTitle?: string;
    requires?: GateRequirement;
    search?: boolean;
    aside?: Snippet;
    children: Snippet;
  };

  let {
    screen,
    title,
    documentTitle,
    requires = "user",
    search = false,
    headerActions,
    aside,
    children,
  }: Props = $props();

  const requirement = $derived<GateRequirement>(
    ROUTES[screen].requiresAdmin ? "admin" : requires,
  );
  const decision = $derived(decideGate(session.state, requirement));
  const pageTitle = $derived(documentTitle ?? title ?? t(ROUTES[screen].title));
  const headerTitle = $derived(title === null ? null : (title ?? pageTitle));
  const errorText = $derived.by(() => {
    const state = session.state;
    if (state.status !== "error") return undefined;
    const reason =
      state.error instanceof ApiError
        ? serverReason(state.error).reason
        : "generic";
    return t(serverReasonMessages[reason]);
  });

  let signOut = $state<SignOut | null>(null);
  let mainRef = $state<HTMLElement | null>(null);
  let contentRef = $state<HTMLElement | null>(null);
  let leavingPasswordChange = false;

  // The desktop frame is fixed (the content column scrolls internally), so
  // after a ClientRouter page swap the window may keep an old offset and the
  // inner scroller an old position. Both start every page at the top.
  onMount(() => {
    const scrollToTop = () => {
      window.scrollTo({ top: 0 });
      contentRef?.scrollTo({ top: 0 });
    };
    void session.load();
    scrollToTop();
    window.addEventListener("pageshow", session.revalidateOnPageShow);
    document.addEventListener("astro:page-load", scrollToTop);
    return () => {
      window.removeEventListener("pageshow", session.revalidateOnPageShow);
      document.removeEventListener("astro:page-load", scrollToTop);
    };
  });

  $effect(() => {
    if (decision === "redirectLogin") redirectToLogin();
  });

  // The change-password view sets its own title (UnauthenticatedLayout).
  $effect(() => {
    if (decision === "changePassword") return;
    document.documentElement.lang = locale.current;
    document.title = t(shellMessages.documentTitle, {
      screen: pageTitle,
      product: t(brandMessages.productName),
    });
  });

  // After a required password change the form is gone: focus main instead.
  $effect(() => {
    if (decision === "changePassword") leavingPasswordChange = true;
    else if (decision === "render" && leavingPasswordChange) {
      leavingPasswordChange = false;
      void tick().then(() => mainRef?.focus());
    }
  });

  const startSignOut = (trigger: HTMLElement | null) => signOut?.start(trigger);
</script>

{#snippet skeleton()}
  <div
    aria-hidden="true"
    class="min-h-dvh md:h-dvh md:p-4 md:overflow-hidden xl:p-8"
    data-shell-skeleton=""
  >
    <div
      class="glass-shell flex min-h-dvh flex-col md:h-[calc(100dvh-2rem)] md:min-h-0 md:flex-row md:rounded-shell xl:h-[calc(100dvh-4rem)]"
    >
      <div class="hidden w-64 shrink-0 flex-col gap-3 px-8 py-8 xl:flex">
        <div class="mb-6 size-10 rounded-full bg-surface-solid/70"></div>
        <div class="h-10 rounded-full bg-surface-solid/50"></div>
        <div class="h-10 rounded-full bg-surface-solid/50"></div>
        <div class="h-10 rounded-full bg-surface-solid/50"></div>
      </div>
      <div
        class="hidden w-22 shrink-0 flex-col items-center gap-4 py-6 md:flex xl:hidden"
      >
        <div class="size-10 rounded-full bg-surface-solid/70"></div>
        <div class="size-12 rounded-full bg-surface-solid/50"></div>
        <div class="size-12 rounded-full bg-surface-solid/50"></div>
        <div class="size-12 rounded-full bg-surface-solid/50"></div>
      </div>
      <div class="flex min-w-0 flex-1 flex-col">
        <div
          class="flex h-16 items-center gap-3 px-4 md:h-auto md:px-8 md:pt-8"
        >
          <div class="size-10 rounded-full bg-surface-solid/70 md:hidden"></div>
          <div
            class="h-7 w-40 rounded-full bg-surface-solid/60 md:h-10 md:w-64"
          ></div>
          <div class="ms-auto size-12 rounded-full bg-surface-solid/70"></div>
        </div>
        <div class="flex flex-col gap-4 px-4 pt-6 md:px-8">
          <div
            class="h-32 rounded-panel bg-surface-solid/40 motion-safe:animate-pulse"
          ></div>
          <div
            class="h-32 rounded-panel bg-surface-solid/40 motion-safe:animate-pulse"
          ></div>
        </div>
      </div>
    </div>
  </div>
{/snippet}

{#if decision === "skeleton" || decision === "redirectLogin"}
  {@render skeleton()}
  <span class="sr-only" role="status">{t(stateMessages.loading)}</span>
{:else if decision === "changePassword"}
  <RequiredPasswordChange />
{:else}
  <SkipLink />
  <div class="min-h-dvh md:h-dvh md:overflow-hidden md:p-4 xl:p-8">
    <div
      class="glass-shell elevation-2 flex min-h-dvh flex-col md:h-[calc(100dvh-2rem)] md:min-h-0 md:flex-row md:overflow-hidden md:rounded-shell xl:h-[calc(100dvh-4rem)]"
    >
      <SidebarNav
        class="hidden xl:flex"
        {screen}
        isAdmin={session.isAdmin}
        onSignOut={startSignOut}
      />
      <IconRail
        class="hidden md:flex xl:hidden"
        {screen}
        isAdmin={session.isAdmin}
        onSignOut={startSignOut}
      />
      <div
        class="flex min-w-0 flex-1 flex-col pb-[calc(5.5rem+var(--safe-area-bottom))] md:min-h-0 md:pb-0"
      >
        <ShellHeader title={headerTitle} {search}>
          {#snippet actions()}{@render headerActions?.()}{/snippet}
        </ShellHeader>
        <div
          bind:this={contentRef}
          class={aside
            ? "flex flex-col gap-6 px-4 pt-6 md:min-h-0 md:flex-1 md:overflow-y-auto md:px-8 md:pb-8 lg:grid lg:grid-cols-[minmax(0,1fr)_22rem] lg:items-start"
            : "px-4 pt-6 md:min-h-0 md:flex-1 md:overflow-y-auto md:px-8 md:pb-8"}
        >
          <main
            bind:this={mainRef}
            id="main"
            tabindex="-1"
            class="min-w-0 outline-none"
          >
            {#if decision === "error"}
              <RegionState
                status="error"
                minHeight="12rem"
                errorMessage={errorText}
                onRetry={() => void session.refresh()}
              >
                {#snippet children()}{/snippet}
              </RegionState>
            {:else if decision === "adminsOnly"}
              <AdminsOnly />
            {:else}
              {@render children()}
            {/if}
          </main>
          {#if aside && decision === "render"}
            <aside aria-label={t(shellMessages.aside)} class="min-w-0">
              {@render aside()}
            </aside>
          {/if}
        </div>
      </div>
    </div>
  </div>
  <!-- Outside the glass shell: its backdrop-filter would make it the
       containing block of the fixed bar, so the bar scrolled with the page. -->
  <BottomBar
    class="md:hidden"
    {screen}
    isAdmin={session.isAdmin}
    onSignOut={startSignOut}
  />
  <SignOut bind:this={signOut} />
  <LiveAnnouncer />
{/if}
