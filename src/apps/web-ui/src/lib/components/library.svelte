<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import {
		fetchCategories,
		fetchPrograms,
		type Category,
		type Program,
	} from "$lib/api/meditation";

	let programs = $state<Program[]>([]);
	let categories = $state<Category[]>([]);
	let error = $state("");

	onMount(async () => {
		try {
			[programs, categories] = await Promise.all([fetchPrograms(), fetchCategories()]);
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "could not load the library";
		}
	});

	interface Group {
		name: string;
		programs: Program[];
	}

	// Category-grouped program cards, uncategorized last (spec FR-031/033).
	const groups = $derived.by<Group[]>(() => {
		const grouped = categories.map((category) => ({
			name: category.name,
			programs: programs.filter((program) => program.category_id === category.id),
		}));
		const uncategorized = programs.filter((program) => program.category_id === null);
		if (uncategorized.length > 0) {
			grouped.push({ name: "Uncategorized", programs: uncategorized });
		}
		return grouped;
	});

	function progressLabel(program: Program): string {
		return `${Math.round(program.progress * 100)}%`;
	}
</script>

<main class="mx-auto min-h-screen w-full max-w-4xl px-4 py-10">
	<header class="mb-8 flex items-baseline justify-between">
		<h1 class="text-3xl font-semibold tracking-tight">Library</h1>
		<nav class="flex gap-2 text-sm">
			<Button variant="outline" size="sm" onclick={() => (window.location.href = "/admin")}>
				Admin
			</Button>
		</nav>
	</header>

	{#if error}
		<p class="text-destructive">{error}</p>
	{/if}

	{#if groups.length === 0 && !error}
		<p class="text-muted-foreground">
			No programs yet. An admin can create them under Admin.
		</p>
	{/if}

	{#each groups as group (group.name)}
		<section class="mb-10">
			<h2 class="mb-4 text-lg font-medium">{group.name}</h2>
			<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
				{#each group.programs as program (program.id)}
					<a href="/program/?id={program.id}" class="block">
					<Card.Root class="transition-shadow hover:shadow-md">
						<Card.Header>
							{#if program.has_cover}
								<img
									src="/api/meditation/programs/{program.id}/cover"
									alt=""
									class="mb-3 h-32 w-full rounded-md object-cover"
								/>
							{:else}
								<div
									class="mb-3 h-32 w-full rounded-md bg-muted"
									aria-label="no cover"
								></div>
							{/if}
							<Card.Title>{program.title}</Card.Title>
							{#if program.description}
								<Card.Description>{program.description}</Card.Description>
							{/if}
						</Card.Header>
						<Card.Content>
							<p class="text-sm text-muted-foreground">
								{#if program.progress >= 1}
									✓ Completed
								{:else}
									{progressLabel(program)} complete
								{/if}
							</p>
						</Card.Content>
					</Card.Root>
					</a>
				{/each}
			</div>
		</section>
	{/each}
</main>
