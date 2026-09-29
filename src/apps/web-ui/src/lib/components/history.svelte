<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { timeline, type MoodRange, type TimelineEvent } from "$lib/api/mood";

	const ranges: { value: MoodRange; label: string }[] = [{ value: "today", label: "Today" }, { value: "last7", label: "Last 7 days" }, { value: "last30", label: "Last 30 days" }, { value: "all", label: "All time" }];
	let events = $state<TimelineEvent[]>([]); let nextCursor = $state<string | null>(null); let range = $state<MoodRange>("all"); let error = $state(""); let loading = $state(true);
	const date = (iso: string) => new Date(iso).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
	async function load() { loading = true; error = ""; try { const page = await timeline(range); events = page.items; nextCursor = page.next_cursor; } catch (cause) { error = cause instanceof Error ? cause.message : "Could not load timeline"; } finally { loading = false; } }
	async function loadMore() { if (!nextCursor) return; try { const page = await timeline(range, nextCursor); events = [...events, ...page.items]; nextCursor = page.next_cursor; } catch (cause) { error = cause instanceof Error ? cause.message : "Could not load more timeline entries"; } }
	onMount(load);
</script>

<main class="mx-auto min-h-screen w-full max-w-3xl px-4 py-10">
	<header class="mb-8 flex items-baseline justify-between"><h1 class="text-3xl font-semibold tracking-tight">History</h1><Button variant="outline" size="sm" onclick={() => (window.location.href = "/mood")}>Mood tracker</Button></header>
	<div class="mb-6 flex flex-wrap gap-2" aria-label="Timeline range">{#each ranges as option}<Button variant={range === option.value ? "default" : "outline"} size="sm" aria-pressed={range === option.value} onclick={() => { range = option.value; void load(); }}>{option.label}</Button>{/each}</div>
	{#if error}<p class="mb-4 text-destructive">{error}</p>{/if}
	{#if loading}<p class="text-muted-foreground">Loading timeline…</p>{:else if events.length === 0}<p class="text-muted-foreground">Your mood and meditation history will appear here.</p>{:else}<div class="space-y-3">{#each events as event}<Card.Root><Card.Content class="p-4"><p class="font-semibold">{event.kind === "Mood" ? `Mood ${event.score}/5` : event.kind === "MeditationNote" ? `Meditation note: ${event.title}` : `Completed meditation: ${event.title}`}</p><p class="text-sm text-muted-foreground">{date(event.recorded_at)}</p>{#if event.note}<p class="mt-2 whitespace-pre-wrap">{event.note}</p>{/if}</Card.Content></Card.Root>{/each}{#if nextCursor}<Button variant="outline" onclick={loadMore}>Load more timeline</Button>{/if}</div>{/if}
</main>
