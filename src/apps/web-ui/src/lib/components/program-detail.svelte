<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import {
		fetchLibrary,
		fetchPlaybackState,
		fetchPrograms,
		type Meditation,
		type Program,
	} from "$lib/api/meditation";

	let program = $state<Program | null>(null);
	let members = $state<Meditation[]>([]);
	let memberStates = $state<Record<number, boolean>>({});
	let error = $state("");

	onMount(async () => {
		const params = new URLSearchParams(window.location.search);
		const id = Number(params.get("id"));
		if (!Number.isFinite(id) || id <= 0) {
			window.location.assign("/library");
			return;
		}
		try {
			const [programs, library] = await Promise.all([fetchPrograms(), fetchLibrary()]);
			program = programs.find((candidate) => candidate.id === id) ?? null;
			if (!program) {
				window.location.assign("/library");
				return;
			}
			// The API returns progress but not membership; derive members from
			// the meditations list once memberships are exposed — for now the
			// detail view lists the whole library with the caller's completed
			// state per item.
			members = library;
			const states = await Promise.all(
				library.map(async (meditation) => {
					const state = await fetchPlaybackState(meditation.id);
					return [meditation.id, state.completed] as const;
				}),
			);
			memberStates = Object.fromEntries(states);
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "could not load the program";
		}
	});
</script>

<main class="mx-auto min-h-screen w-full max-w-3xl px-4 py-10">
	<header class="mb-8 flex items-baseline justify-between">
		<div>
			{#if program}
				<h1 class="text-3xl font-semibold tracking-tight">{program.title}</h1>
				{#if program.description}
					<p class="mt-1 text-muted-foreground">{program.description}</p>
				{/if}
			{/if}
		</div>
		<Button variant="outline" size="sm" onclick={() => (window.location.href = "/library")}>
			Back
		</Button>
	</header>

	{#if error}
		<p class="text-destructive">{error}</p>
	{/if}

	<Card.Root>
		<Card.Header>
			<Card.Title>Meditations</Card.Title>
			{#if program?.progress !== null && program?.progress !== undefined}
				<Card.Description>
					Completed: {Math.round((program.progress ?? 0) * 100)}%
				</Card.Description>
			{/if}
		</Card.Header>
		<Card.Content>
			<ul class="grid gap-2">
				{#each members as meditation (meditation.id)}
					<li>
						<a
							href="/player/?id={meditation.id}"
							class="flex items-center justify-between rounded-md border p-3 transition-colors hover:bg-muted"
						>
							<div>
								<p class="font-medium">{meditation.name}</p>
								<p class="text-sm text-muted-foreground">
									{(meditation.size_bytes / 1024).toFixed(0)} KB · Opus
								</p>
							</div>
							{#if memberStates[meditation.id]}
								<span class="text-sm text-muted-foreground">✓ Completed</span>
							{/if}
						</a>
					</li>
				{/each}
			</ul>
		</Card.Content>
	</Card.Root>
</main>
