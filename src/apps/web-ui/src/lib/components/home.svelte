<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import {
		fetchContinueProgram,
		fetchDailySuggestion,
		fetchMe,
		type ContinueProgram,
		type Meditation,
	} from "$lib/api/meditation";

	let role = $state<string | null>(null);
	let ready = $state(false);
	let continueProgram = $state<ContinueProgram | null>(null);
	let dailySuggestion = $state<Meditation | null>(null);

	onMount(async () => {
		try {
			const me = await fetchMe();
			role = me.role;
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
	});
</script>

{#if ready}
	<main class="mx-auto flex min-h-screen max-w-2xl flex-col justify-center gap-6 px-6">
		<h1 class="text-4xl font-semibold tracking-tight">MediOSS</h1>
		<p class="text-lg">A private, self-hosted space for meditation and rest.</p>
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
							{#if continueProgram.program.description || continueProgram.program.progress !== null}
								<Card.Description>
									{#if continueProgram.program.description}
										{continueProgram.program.description}
										{#if continueProgram.program.progress !== null}
											·
										{/if}
									{/if}
									{#if continueProgram.program.progress !== null}
										{Math.round(continueProgram.program.progress * 100)}% complete
									{/if}
								</Card.Description>
							{/if}
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
						<Card.Title>{dailySuggestion.name}</Card.Title>
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
		<div class="flex flex-wrap gap-3">
			<Button size="lg" onclick={() => (window.location.href = "/library")}>
				Begin
			</Button>
			{#if role === "admin"}
				<Button size="lg" variant="outline" onclick={() => (window.location.href = "/admin")}>
					Admin
				</Button>
			{/if}
		</div>
	</main>
{/if}
