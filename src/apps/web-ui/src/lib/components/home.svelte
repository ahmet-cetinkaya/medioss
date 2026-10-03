<script lang="ts">
	import { onMount } from "svelte";
	import { fetchMeProfile } from "$lib/api/identity";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import {
		fetchContinueProgram,
		fetchDailySuggestion,
		fetchRecentPrograms,
		meditationProgressLabel,
		programProgressLabel,
		type ContinueProgram,
		type Meditation,
		type RecentProgram,
	} from "$lib/api/meditation";

	let role = $state<string | null>(null);
	let ready = $state(false);
	let displayName = $state<string | null>(null);
	let email = $state<string | null>(null);
	let hasAvatar = $state(false);
	let myId = $state<number | null>(null);
	let avatarCacheTag = $state(Date.now());
	let continueProgram = $state<ContinueProgram | null>(null);
	let dailySuggestion = $state<Meditation | null>(null);
	let recentPrograms = $state<RecentProgram[]>([]);

	onMount(async () => {
		try {
			const me = await fetchMeProfile();
			myId = me.id;
			role = me.role;
			displayName = me.display_name;
			email = me.email;
			hasAvatar = me.has_avatar;
		} catch {
			window.location.assign("/login");
			return;
		}
		ready = true;
		try {
			continueProgram = await fetchContinueProgram();
		} catch {
			// Continuing is a nicety, not core navigation — a failure here must
			// not block the rest of the home page.
		}
		try {
			dailySuggestion = await fetchDailySuggestion();
		} catch {
			// Same as above: a suggestion failure must not block the home page.
		}
		try {
			recentPrograms = await fetchRecentPrograms(5);
		} catch {
			// Same as above: a history failure must not block the home page.
		}
	});
</script>

{#if ready}
	<main class="mx-auto flex min-h-screen max-w-2xl flex-col justify-center gap-6 px-6">
		<div class="flex items-center gap-3">
			{#if hasAvatar}
				<img
					src={`/api/identity/avatar/${myId}?v=${avatarCacheTag}`}
					alt="Avatar"
					class="h-12 w-12 rounded-full object-cover"
				/>
			{:else}
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-muted text-lg font-medium">
					{(displayName ?? email ?? "?").charAt(0).toLocaleUpperCase("tr")}
				</div>
			{/if}
			<div class="grid">
				<h1 class="text-4xl font-semibold tracking-tight">MediOSS</h1>
			</div>
		</div>
		<p class="text-lg">A private, self-hosted space for meditation and rest.</p>
		<div class="flex flex-wrap gap-2">
			<Button onclick={() => (window.location.href = "/mood")}>Mood tracker</Button>
			<Button variant="outline" onclick={() => (window.location.href = "/history")}>History</Button>
			<Button variant="outline" onclick={() => (window.location.href = "/stats")}>Statistics</Button>
			<Button variant="outline" onclick={() => (window.location.href = "/settings")}>Settings</Button>
		</div>
		{#if continueProgram}
			<section class="grid gap-2">
				<h2 class="text-sm font-medium text-muted-foreground">Continue program</h2>
				<Card.Root class="flex flex-row items-center gap-4 p-4">
					<div class="shrink-0">
						{#if continueProgram.program.has_cover}
							<img
								src="/api/meditation/programs/{continueProgram.program.id}/cover"
								alt=""
								class="h-20 w-20 rounded-md object-cover"
							/>
						{:else}
							<div class="h-20 w-20 rounded-md bg-muted" aria-hidden="true"></div>
						{/if}
					</div>
					<div class="flex flex-1 flex-col gap-2">
						<div>
							<Card.Title>{continueProgram.program.title}</Card.Title>
							<Card.Description>
								{#if continueProgram.program.description}
									{continueProgram.program.description}
									·
								{/if}
								{programProgressLabel(continueProgram.program)}
							</Card.Description>
						</div>
						<Button
							size="sm"
							class="self-start"
							onclick={() =>
								(window.location.href = `/program/?id=${continueProgram!.program.id}`)}
						>
							Continue program
						</Button>
					</div>
				</Card.Root>
			</section>
		{/if}
		{#if dailySuggestion}
			<section class="grid gap-2">
				<h2 class="text-sm font-medium text-muted-foreground">Daily suggestion</h2>
				<Card.Root class="flex flex-row items-center gap-4 p-4">
					<div class="shrink-0">
						{#if dailySuggestion.program_has_cover}
							<img
								src="/api/meditation/programs/{dailySuggestion.program_id}/cover"
								alt=""
								class="h-20 w-20 rounded-md object-cover"
							/>
						{:else}
							<div class="h-20 w-20 rounded-md bg-muted" aria-hidden="true"></div>
						{/if}
					</div>
					<div class="flex flex-1 flex-col gap-2">
						<div>
							<Card.Title>{dailySuggestion.name}</Card.Title>
							<Card.Description>{meditationProgressLabel(dailySuggestion)}</Card.Description>
						</div>
						<Button
							size="sm"
							class="self-start"
							onclick={() => (window.location.href = `/player/?id=${dailySuggestion!.id}`)}
						>
							Listen now
						</Button>
					</div>
				</Card.Root>
			</section>
		{/if}
		{#if recentPrograms.length > 0}
			<section class="grid gap-2">
				<div class="flex items-baseline justify-between">
					<h2 class="text-sm font-medium text-muted-foreground">Recently experienced</h2>
					<a href="/history" class="text-sm font-medium underline-offset-4 hover:underline">
						See all
					</a>
				</div>
				<div class="-mx-6 flex gap-3 overflow-x-auto px-6 pb-2">
					{#each recentPrograms as entry (entry.program.id)}
						<a href="/program/?id={entry.program.id}" class="block shrink-0">
							<Card.Root class="w-36 gap-2 p-3">
								{#if entry.program.has_cover}
									<img
										src="/api/meditation/programs/{entry.program.id}/cover"
										alt=""
										class="h-24 w-full rounded-md object-cover"
									/>
								{:else}
									<div class="h-24 w-full rounded-md bg-muted" aria-hidden="true"></div>
								{/if}
								<p class="truncate text-sm font-medium">{entry.program.title}</p>
								<p class="text-xs text-muted-foreground">{programProgressLabel(entry.program)}</p>
							</Card.Root>
						</a>
					{/each}
				</div>
			</section>
		{/if}
		<div class="flex flex-wrap gap-3">
			<Button size="lg" onclick={() => (window.location.href = "/library")}>
				Begin
			</Button>
			<Button size="lg" variant="outline" onclick={() => (window.location.href = "/soundscape")}>
				Soundscape
			</Button>
			<Button size="lg" variant="outline" onclick={() => (window.location.href = "/stats")}>
				Statistics
			</Button>
			{#if role === "admin"}
				<Button size="lg" variant="outline" onclick={() => (window.location.href = "/admin")}>
					Admin
				</Button>
			{/if}
		</div>
	</main>
{/if}
