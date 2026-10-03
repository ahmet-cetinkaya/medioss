<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import * as Card from "$lib/components/ui/card";
	import {
		createNote,
		deleteNote,
		fetchMeditation,
		fetchNotes,
		fetchPlaybackState,
		postCompletion,
		postPosition,
		putCompleted,
		type Meditation,
		type Note,
	} from "$lib/api/meditation";

	let meditationId = $state<number | null>(null);
	let meditation = $state<Meditation | null>(null);
	let audio: HTMLAudioElement | undefined = $state();
	let error = $state("");
	let notes = $state<Note[]>([]);
	let noteDraft = $state("");
	let completed = $state(false);
	let speeds = [0.5, 0.75, 1, 1.25, 1.5, 1.75, 2];
	let speed = $state(1);

	// Media-time position reporting: throttled while playing, immediate on
	// pause; completion fires once when playback reaches the end (spec
	// FR-015/017..021).
	const POSITION_INTERVAL_MS = 5000;
	let lastPostedMs = 0;
	let startedAt: string | null = null;
	let completionSent = false;

	onMount(async () => {
		const params = new URLSearchParams(window.location.search);
		const id = Number(params.get("id"));
		if (!Number.isFinite(id) || id <= 0) {
			window.location.assign("/library");
			return;
		}
		meditationId = id;
		document.addEventListener("pagehide", flushOnHide);
		try {
			const [state, meta] = await Promise.all([fetchPlaybackState(id), fetchMeditation(id)]);
			meditation = meta;
			completed = state.completed;
			notes = await fetchNotes(id);
			// Resume from the stored position; a completed meditation starts
			// from the beginning (server resets position on completion).
			if (audio) {
				if (!state.completed && state.position_ms > 0) {
					audio.currentTime = state.position_ms / 1000;
				}
				audio.playbackRate = speed;
			}
		} catch (cause) {
			window.location.assign("/login");
		}
	});

	async function guard(action: () => Promise<void>) {
		try {
			await action();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "request failed";
		}
	}

	function onTimeUpdate() {
		if (!audio || meditationId === null) return;
		const nowMs = Math.floor(audio.currentTime * 1000);
		if (nowMs - lastPostedMs >= POSITION_INTERVAL_MS) {
			lastPostedMs = nowMs;
			void guard(() => postPosition(meditationId!, nowMs));
		}
	}

	function onPause() {
		if (!audio || meditationId === null) return;
		const nowMs = Math.floor(audio.currentTime * 1000);
		lastPostedMs = nowMs;
		void guard(() => postPosition(meditationId!, nowMs));
	}

	function onEnded() {
		if (!audio || meditationId === null || completionSent) return;
		completionSent = true;
		const finishedAt = new Date().toISOString();
		const start = startedAt ?? new Date(Date.now() - audio.duration * 1000).toISOString();
		void guard(async () => {
			await postCompletion(meditationId!, start, finishedAt);
			completed = true;
		});
	}

	function onPlay() {
		if (startedAt === null) startedAt = new Date().toISOString();
		syncMediaSessionState();
	}

	// Final position flush on tab close: sendBeacon carries the last position
	// without awaiting a response (plan D11).
	function flushOnHide() {
		if (meditationId === null || !audio || audio.paused) return;
		const payload = JSON.stringify({ position_ms: Math.floor(audio.currentTime * 1000) });
		navigator.sendBeacon(
			`/api/meditation/${meditationId}/playback-state`,
			new Blob([payload], { type: "application/json" }),
		);
	}

	function setRate(next: number) {
		speed = next;
		if (audio) audio.playbackRate = next;
	}

	function syncMediaSessionState() {
		if (typeof navigator === "undefined" || !("mediaSession" in navigator) || !audio) return;
		navigator.mediaSession.playbackState = audio.paused ? "paused" : "playing";
	}

	function setupMediaSession() {
		if (typeof navigator === "undefined" || !("mediaSession" in navigator) || !audio) return;
		if (meditation) {
			navigator.mediaSession.metadata = new MediaMetadata({
				title: meditation.name,
			});
		}
		navigator.mediaSession.setActionHandler("play", () => void audio!.play());
		navigator.mediaSession.setActionHandler("pause", () => audio!.pause());
		navigator.mediaSession.setActionHandler("seekbackward", () => {
			audio!.currentTime = Math.max(0, audio!.currentTime - 10);
		});
		navigator.mediaSession.setActionHandler("seekforward", () => {
			audio!.currentTime = Math.min(audio!.duration || 0, audio!.currentTime + 10);
		});
		navigator.mediaSession.setActionHandler("seekto", (details) => {
			if (details.seekTime !== undefined && audio!.duration) {
				audio!.currentTime = Math.min(details.seekTime, audio!.duration);
			}
		});
		navigator.mediaSession.setActionHandler("stop", () => {
			audio!.pause();
			audio!.currentTime = 0;
			syncMediaSessionState();
		});
		navigator.mediaSession.setActionHandler("previoustrack", () => {});
		navigator.mediaSession.setActionHandler("nexttrack", () => {});
	}

	async function toggleCompleted() {
		if (meditationId === null) return;
		const next = !completed;
		await guard(async () => {
			await putCompleted(meditationId!, next);
			completed = next;
		});
	}

	async function submitNote(event: SubmitEvent) {
		event.preventDefault();
		if (meditationId === null || !noteDraft.trim()) return;
		await guard(async () => {
			const created = await createNote(meditationId!, noteDraft);
			notes = [created, ...notes];
			noteDraft = "";
		});
	}

	async function removeNote(noteId: number) {
		if (meditationId === null) return;
		await guard(async () => {
			await deleteNote(meditationId!, noteId);
			notes = notes.filter((note) => note.id !== noteId);
		});
	}

	function formatTime(seconds: number): string {
		if (!Number.isFinite(seconds)) return "0:00";
		const minutes = Math.floor(seconds / 60);
		const rest = Math.floor(seconds % 60);
		return `${minutes}:${rest.toString().padStart(2, "0")}`;
	}
</script>

<main class="mx-auto min-h-screen w-full max-w-2xl px-4 py-10">
	<header class="mb-6 flex items-baseline justify-between">
		<h1 class="text-3xl font-semibold tracking-tight">Player</h1>
		<Button
			variant="outline"
			size="sm"
			onclick={() => (history.length > 1 ? history.back() : (window.location.href = "/library"))}
		>
			Back
		</Button>
	</header>

	{#if error}<p class="mb-4 text-destructive">{error}</p>{/if}

	{#if meditationId !== null}
		<audio
			bind:this={audio}
			src="/api/meditation/{meditationId}/audio"
			controls
			class="w-full"
			ontimeupdate={onTimeUpdate}
			onpause={() => {
				onPause();
				syncMediaSessionState();
			}}
			onended={onEnded}
			onplay={onPlay}
			onloadedmetadata={setupMediaSession}
		></audio>

		<div class="mt-4 flex flex-wrap items-center gap-2">
			<span class="text-sm text-muted-foreground">Speed</span>
			{#each speeds as value (value)}
				<Button
					size="sm"
					variant={speed === value ? "default" : "outline"}
					onclick={() => setRate(value)}
				>
					{value}×
				</Button>
			{/each}
		</div>

		<div class="mt-4 flex items-center gap-3">
			<Button variant="outline" size="sm" onclick={toggleCompleted}>
				{completed ? "✓ Completed" : "Mark completed"}
			</Button>
			<a href="/api/meditation/{meditationId}/download" download>
				<Button variant="outline" size="sm">Download</Button>
			</a>
		</div>

		<Card.Root class="mt-8">
			<Card.Header><Card.Title>Notes</Card.Title>
				<Card.Description>Private to you, newest first.</Card.Description>
			</Card.Header>
			<Card.Content>
				<form class="mb-4 grid gap-2" onsubmit={submitNote}>
					<Label for="note" class="sr-only">Note</Label>
					<Input id="note" bind:value={noteDraft} placeholder="How did it feel?" />
					<Button type="submit" disabled={!noteDraft.trim()}>Add note</Button>
				</form>
				<ul class="grid gap-2">
					{#each notes as note (note.id)}
						<li class="flex items-start justify-between gap-2 rounded-md border p-3">
							<div>
								<p class="text-sm">{note.text}</p>
								<p class="text-xs text-muted-foreground">
									{new Date(note.created_at).toLocaleString()}
								</p>
							</div>
							<Button variant="ghost" size="sm" onclick={() => removeNote(note.id)}>
								Delete
							</Button>
						</li>
					{/each}
				</ul>
			</Card.Content>
		</Card.Root>
	{/if}
</main>
