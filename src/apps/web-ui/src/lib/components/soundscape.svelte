<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { fetchMe } from "$lib/api/meditation";
	import { createSoundscapeMix, deleteSoundscapeMix, fetchSoundscapeCategories, fetchSoundscapeMixes, fetchSounds, soundAudioUrl, updateSoundscapeMix, type SoundscapeCategory, type SoundscapeMix, type SoundscapeSound } from "$lib/api/soundscape";
	let categories = $state<SoundscapeCategory[]>([]);
	let sounds = $state<SoundscapeSound[]>([]);
	let selected = $state<SoundscapeSound[]>([]);
	let playing = $state(false);
	let error = $state("");
	let audio = new Map<number, HTMLAudioElement>();
	let mixes = $state<SoundscapeMix[]>([]);
	let mixName = $state("");
	let timerMinutes = $state(5);
	let timer: ReturnType<typeof setTimeout> | undefined;
	let remaining = $state<number | null>(null);
	onMount(async () => { try { await fetchMe(); [categories, sounds, mixes] = await Promise.all([fetchSoundscapeCategories(), fetchSounds(), fetchSoundscapeMixes()]); } catch { window.location.assign("/login"); } });
	function formatDuration(durationMs: number): string {
		const totalSeconds = Math.round(durationMs / 1000);
		const minutes = Math.floor(totalSeconds / 60);
		const seconds = totalSeconds % 60;
		return minutes > 0 ? `${minutes}:${String(seconds).padStart(2, "0")} min` : `${seconds}s`;
	}
	function toggle(sound: SoundscapeSound) {
		if (selected.some((item) => item.id === sound.id)) {
			audio.get(sound.id)?.pause();
			audio.delete(sound.id);
			selected = selected.filter((item) => item.id !== sound.id);
			return;
		}
		error = "";
		const player = new Audio(soundAudioUrl(sound.id));
		player.loop = true;
		player.dataset.volume = "1";
		audio.set(sound.id, player);
		selected = [...selected, sound];
		// Join a running soundscape immediately: adding while playing must not
		// require stopping and restarting everything.
		if (playing) player.play().catch(() => { error = "Playback needs a browser interaction."; });
	}
	async function play() { try { await Promise.all([...audio.values()].map((player) => player.play())); playing = true; } catch { error = "Playback needs a browser interaction."; } }
	function cancelTimer() {
		if (timer) clearInterval(timer);
		timer = undefined;
		remaining = null;
		for (const sound of selected) {
			const player = audio.get(sound.id);
			if (player) player.volume = Number(player.dataset.volume ?? player.volume);
		}
	}

	function stop() {
		cancelTimer();
		for (const player of audio.values()) {
			player.pause();
			player.currentTime = 0;
		}
		playing = false;
	}
	function startTimer() {
		if (!Number.isInteger(timerMinutes) || timerMinutes < 5 || timerMinutes > 120) {
			error = "Timer must be between 5 and 120 minutes.";
			return;
		}
		cancelTimer();
		const deadline = Date.now() + timerMinutes * 60_000;
		remaining = timerMinutes * 60;
		timer = setInterval(() => {
			remaining = Math.max(0, Math.ceil((deadline - Date.now()) / 1000));
			const gain = remaining < 60 ? remaining / 60 : 1;
			for (const sound of selected) {
				const player = audio.get(sound.id);
				if (player) player.volume = ((player.dataset.volume ? Number(player.dataset.volume) : player.volume) * gain);
			}
			if (remaining === 0) stop();
		}, 250);
	}
	function loadMix(mix: SoundscapeMix) {
		stop();
		audio = new Map();
		const next = mix.items.map((item) => sounds.find((sound) => sound.id === item.sound_id)).filter((sound): sound is SoundscapeSound => sound !== undefined);
		for (const sound of next) {
			const player = new Audio(soundAudioUrl(sound.id));
			player.loop = true;
			player.volume = (mix.items.find((item) => item.sound_id === sound.id)?.volume ?? 100) / 100;
			player.dataset.volume = String(player.volume);
			audio.set(sound.id, player);
		}
		selected = next;
		if (next.length !== mix.items.length) error = "Some sounds from this mix are no longer available.";
	}

	async function renameMix(mix: SoundscapeMix) {
		const name = window.prompt("Mix name", mix.name);
		if (!name?.trim()) return;
		try {
			const updated = await updateSoundscapeMix(mix.id, name, mix.items);
			mixes = mixes.map((item) => item.id === mix.id ? updated : item);
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "Could not rename mix.";
		}
	}

	async function removeMix(mix: SoundscapeMix) {
		try {
			await deleteSoundscapeMix(mix.id);
			mixes = mixes.filter((item) => item.id !== mix.id);
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "Could not delete mix.";
		}
	}

	async function saveMix() {
		if (!mixName.trim() || selected.length === 0) return;
		try {
			const mix = await createSoundscapeMix(mixName, selected.map((sound, position) => ({ sound_id: sound.id, volume: Math.round((audio.get(sound.id)?.volume ?? 1) * 100), position })));
			mixes = [mix, ...mixes];
			mixName = "";
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "Could not save mix.";
		}
	}
</script>
<main class="mx-auto min-h-screen max-w-5xl p-4 sm:p-8">
	<header class="mb-6 flex flex-wrap items-center justify-between gap-3"><div><h1 class="text-3xl font-semibold">Soundscape</h1><p class="text-muted-foreground">Layer any number of ambient sounds into your mix.</p></div><a class="underline" href="/">Home</a></header>
	{#if error}<p class="mb-4 text-destructive" role="status">{error}</p>{/if}
	<section class="mb-8 flex flex-wrap gap-2"><Button onclick={playing ? stop : play} disabled={selected.length === 0}>{playing ? "Stop" : "Play mix"}</Button><label class="flex items-center gap-2 text-sm" for="timer">Timer minutes <input id="timer" class="w-16 rounded border p-1" type="number" min="5" max="120" bind:value={timerMinutes} /></label><Button variant="outline" onclick={startTimer} disabled={!playing}>Start timer</Button><Button variant="outline" onclick={cancelTimer} disabled={remaining === null}>Cancel timer</Button>{#if remaining !== null}<span class="self-center text-sm" aria-live="polite">{Math.floor(remaining / 60)}:{String(remaining % 60).padStart(2, "0")}</span>{/if}</section>
	<section class="mb-6 flex flex-wrap gap-2"><input class="rounded border p-2" bind:value={mixName} placeholder="Mix name" aria-label="Mix name" /><Button variant="outline" onclick={saveMix} disabled={selected.length === 0 || !mixName.trim()}>Save mix</Button>{#each mixes as mix}<span class="flex items-center gap-1 rounded bg-muted px-2 py-1 text-sm"><button class="underline" onclick={() => loadMix(mix)}>{mix.name}</button><button aria-label="Rename {mix.name}" onclick={() => renameMix(mix)}>Rename</button><button aria-label="Delete {mix.name}" onclick={() => removeMix(mix)}>Delete</button></span>{/each}</section>
	<section class="grid gap-6 md:grid-cols-2"><div>{#each categories as category}<h2 class="mb-2 text-lg font-medium">{category.name}</h2><div class="mb-5 grid gap-2">{#each sounds.filter((sound) => sound.category_id === category.id) as sound}<Card.Root class="flex flex-row items-center justify-between p-3"><span>{sound.name} · {formatDuration(sound.duration_ms)}</span><Button size="sm" variant={selected.some((item) => item.id === sound.id) ? "secondary" : "outline"} onclick={() => toggle(sound)}>{selected.some((item) => item.id === sound.id) ? "Remove" : "Add"}</Button><a href="/api/soundscape/sounds/{sound.id}/download"><Button size="sm" variant="ghost" type="button">Download</Button></a></Card.Root>{/each}</div>{/each}</div><aside><h2 class="mb-2 text-lg font-medium">Your mix ({selected.length})</h2>{#if selected.length === 0}<p class="text-muted-foreground">Choose sounds from the library.</p>{/if}{#each selected as sound}<Card.Root class="mb-2 p-3"><label class="block text-sm font-medium" for="volume-{sound.id}">{sound.name} volume</label><input id="volume-{sound.id}" class="w-full" type="range" min="0" max="100" value="100" oninput={(event) => { const player = audio.get(sound.id); if (player) { player.volume = Number((event.currentTarget as HTMLInputElement).value) / 100; player.dataset.volume = String(player.volume); } }} /></Card.Root>{/each}</aside></section>
</main>
