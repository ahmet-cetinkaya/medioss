<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import {
		fetchRecentPrograms,
		programProgressLabel,
		type RecentProgram,
	} from "$lib/api/meditation";

	let entries = $state<RecentProgram[]>([]);
	let error = $state("");

	onMount(async () => {
		try {
			entries = await fetchRecentPrograms();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "could not load your history";
		}
	});

	interface DayGroup {
		label: string;
		entries: RecentProgram[];
	}

	function dayLabel(iso: string): string {
		const date = new Date(iso);
		const today = new Date();
		const yesterday = new Date();
		yesterday.setDate(today.getDate() - 1);
		const sameDay = (a: Date, b: Date) =>
			a.getFullYear() === b.getFullYear() &&
			a.getMonth() === b.getMonth() &&
			a.getDate() === b.getDate();
		if (sameDay(date, today)) return "Today";
		if (sameDay(date, yesterday)) return "Yesterday";
		return date.toLocaleDateString(undefined, {
			year: "numeric",
			month: "long",
			day: "numeric",
		});
	}

	function timeLabel(iso: string): string {
		return new Date(iso).toLocaleTimeString(undefined, {
			hour: "2-digit",
			minute: "2-digit",
		});
	}

	// Already newest-first from the API; group consecutive entries sharing a
	// calendar day into one timeline section.
	const groups = $derived.by<DayGroup[]>(() => {
		const grouped: DayGroup[] = [];
		for (const entry of entries) {
			const label = dayLabel(entry.last_touched_at);
			const current = grouped.at(-1);
			if (current && current.label === label) {
				current.entries.push(entry);
			} else {
				grouped.push({ label, entries: [entry] });
			}
		}
		return grouped;
	});
</script>

<main class="mx-auto min-h-screen w-full max-w-2xl px-4 py-10">
	<header class="mb-8 flex items-baseline justify-between">
		<h1 class="text-3xl font-semibold tracking-tight">History</h1>
		<div class="flex gap-2">
			<Button variant="outline" size="sm" onclick={() => (window.location.href = "/stats")}>Statistics</Button>
			<Button
				variant="outline"
				size="sm"
				onclick={() => (history.length > 1 ? history.back() : (window.location.href = "/"))}
			>
				Back
			</Button>
		</div>
	</header>

	{#if error}
		<p class="text-destructive">{error}</p>
	{/if}

	{#if groups.length === 0 && !error}
		<p class="text-muted-foreground">
			Nothing here yet — play a meditation and it will show up in your history.
		</p>
	{/if}

	{#each groups as group (group.label)}
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-medium text-muted-foreground">{group.label}</h2>
			<ol class="relative border-l pl-6">
				{#each group.entries as entry (entry.program.id + entry.last_touched_at)}
					<li class="relative mb-6 last:mb-0">
						<span
							class="absolute top-1.5 -left-[29px] h-3 w-3 rounded-full border-2 border-background bg-primary"
							aria-hidden="true"
						></span>
						<a
							href="/program/?id={entry.program.id}"
							class="flex items-center gap-3 rounded-md border p-3 transition-colors hover:bg-muted"
						>
							{#if entry.program.has_cover}
								<img
									src="/api/meditation/programs/{entry.program.id}/cover"
									alt=""
									class="h-14 w-14 shrink-0 rounded-md object-cover"
								/>
							{:else}
								<div class="h-14 w-14 shrink-0 rounded-md bg-muted" aria-hidden="true"></div>
							{/if}
							<div class="min-w-0 flex-1">
								<p class="truncate font-medium">{entry.program.title}</p>
								{#if entry.program.description}
									<p class="truncate text-sm text-muted-foreground">
										{entry.program.description}
									</p>
								{/if}
								<p class="text-sm text-muted-foreground">{programProgressLabel(entry.program)}</p>
							</div>
							<span class="shrink-0 text-sm text-muted-foreground">
								{timeLabel(entry.last_touched_at)}
							</span>
						</a>
					</li>
				{/each}
			</ol>
		</section>
	{/each}
</main>
