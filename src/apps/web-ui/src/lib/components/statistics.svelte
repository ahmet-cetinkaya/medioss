<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import {
		fetchStatistics,
		type StatisticsRange,
		type UserStatistics,
	} from "$lib/api/meditation";

	const ranges: { value: StatisticsRange; label: string }[] = [
		{ value: "today", label: "Today" },
		{ value: "last30", label: "Last 30 days" },
		{ value: "all", label: "All time" },
	];

	let range = $state<StatisticsRange>("all");
	let statistics = $state<UserStatistics | null>(null);
	let error = $state("");
	let loading = $state(true);

	async function load() {
		loading = true;
		error = "";
		try {
			statistics = await fetchStatistics(range);
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "could not load statistics";
		} finally {
			loading = false;
		}
	}

	function selectRange(value: StatisticsRange) {
		range = value;
		void load();
	}

	function dateLabel(date: string) {
		return new Date(`${date}T00:00:00`).toLocaleDateString(undefined, {
			month: "short",
			day: "numeric",
		});
	}

	onMount(load);
</script>

<main class="mx-auto min-h-screen w-full max-w-2xl px-4 py-10">
	<header class="mb-8 flex items-baseline justify-between">
		<h1 class="text-3xl font-semibold tracking-tight">Statistics</h1>
		<Button variant="outline" size="sm" onclick={() => (window.location.href = "/history")}>History</Button>
	</header>

	<div class="mb-6 flex gap-2" aria-label="Statistics range">
		{#each ranges as option}
			<Button variant={range === option.value ? "default" : "outline"} size="sm" onclick={() => selectRange(option.value)}>
				{option.label}
			</Button>
		{/each}
	</div>

	{#if error}<p class="mb-4 text-destructive">{error}</p>{/if}
	{#if loading}
		<p class="text-muted-foreground">Loading statistics…</p>
	{:else if statistics}
		<div class="mb-8 grid grid-cols-2 gap-4 sm:grid-cols-4">
			{#each [
				["Total minutes", statistics.total_minutes],
				["Total days", statistics.total_days],
				["Current streak", statistics.current_streak],
				["Longest streak", statistics.longest_streak],
			] as metric}
				<Card.Root><Card.Header class="p-4"><Card.Description>{metric[0]}</Card.Description><Card.Title class="text-2xl">{metric[1]}</Card.Title></Card.Header></Card.Root>
			{/each}
		</div>

		<Card.Root>
			<Card.Header><Card.Title>Meditation calendar</Card.Title><Card.Description>Completed practice days in the selected range</Card.Description></Card.Header>
			<Card.Content>
				{#if statistics.calendar.length === 0}
					<p class="text-sm text-muted-foreground">No completed sessions in this range.</p>
				{:else}
					<ul class="grid grid-cols-2 gap-2 sm:grid-cols-3" aria-label="Meditated days">
						{#each statistics.calendar as day}
							<li class="rounded-md border bg-muted px-3 py-2 text-sm"><strong>{dateLabel(day.date)}</strong><span class="ml-2 text-muted-foreground">{day.minutes} min</span></li>
						{/each}
					</ul>
				{/if}
			</Card.Content>
		</Card.Root>
	{/if}
</main>
