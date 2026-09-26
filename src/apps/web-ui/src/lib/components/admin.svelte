<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import * as Card from "$lib/components/ui/card";
	import {
		addMember,
		assignCategory,
		createCategory,
		createProgram,
		fetchCategories,
		fetchLibrary,
		fetchMe,
		titleCaseFromFile,
		uploadAudio,
		type Category,
		type Meditation,
	} from "$lib/api/meditation";

	let me = $state<{ role: string } | null>(null);
	let meditations = $state<Meditation[]>([]);
	let categories = $state<Category[]>([]);
	let error = $state("");
	let message = $state("");

	let file = $state<File | null>(null);
	let name = $state("");
	let uploading = $state(false);

	let programTitle = $state("");
	let programDescription = $state("");
	let cover = $state<File | null>(null);
	let creatingProgram = $state(false);

	let categoryName = $state("");

	onMount(async () => {
		try {
			me = await fetchMe();
			if (me.role !== "admin") {
				window.location.assign("/library");
				return;
			}
			await reload();
		} catch (cause) {
			window.location.assign("/login");
		}
	});

	async function reload() {
		[meditations, categories] = await Promise.all([fetchLibrary(), fetchCategories()]);
	}

	function onFileChosen(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		file = input.files?.[0] ?? null;
		if (file) {
			// Title Case prefill from the file name, still editable (FR-024).
			if (!name.trim()) name = titleCaseFromFile(file.name);
		}
	}

	async function submitUpload(event: SubmitEvent) {
		event.preventDefault();
		if (!file || uploading) return;
		uploading = true;
		error = "";
		message = "";
		try {
			const created = await uploadAudio(file, name);
			message = `Uploaded "${created.name}" (${created.size_bytes} bytes stored as Opus)`;
			name = "";
			file = null;
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "upload failed";
		} finally {
			uploading = false;
		}
	}

	async function submitProgram(event: SubmitEvent) {
		event.preventDefault();
		if (!programTitle.trim()) return;
		creatingProgram = true;
		error = "";
		try {
			const program = await createProgram(programTitle, programDescription || null, cover);
			message = `Program "${program.title}" created`;
			programTitle = "";
			programDescription = "";
			cover = null;
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "program creation failed";
		} finally {
			creatingProgram = false;
		}
	}

	async function submitCategory(event: SubmitEvent) {
		event.preventDefault();
		if (!categoryName.trim()) return;
		error = "";
		try {
			await createCategory(categoryName);
			categoryName = "";
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "category creation failed";
		}
	}

	async function addTo(programId: number, event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		const meditationId = Number(select.value);
		if (!meditationId) return;
		try {
			await addMember(programId, meditationId);
			select.value = "";
			message = "Added to program";
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "could not add member";
		}
	}
</script>

<main class="mx-auto min-h-screen w-full max-w-3xl px-4 py-10">
	<header class="mb-8 flex items-baseline justify-between">
		<h1 class="text-3xl font-semibold tracking-tight">Admin</h1>
		<Button variant="outline" size="sm" onclick={() => (window.location.href = "/library")}>
			Library
		</Button>
	</header>

	{#if error}<p class="mb-4 text-destructive">{error}</p>{/if}
	{#if message}<p class="mb-4 text-sm text-muted-foreground">{message}</p>{/if}

	<Card.Root class="mb-8">
		<Card.Header><Card.Title>Upload meditation audio</Card.Title>
			<Card.Description>MP3, WAV or OGG up to 100MB; stored as Opus.</Card.Description>
		</Card.Header>
		<Card.Content>
			<form class="grid gap-4" onsubmit={submitUpload}>
				<div class="grid gap-2">
					<Label for="file">Audio file</Label>
					<Input id="file" type="file" accept=".mp3,.wav,.ogg,.opus" onchange={onFileChosen} />
				</div>
				<div class="grid gap-2">
					<Label for="name">Name</Label>
					<Input id="name" bind:value={name} placeholder="Morning Meditation" />
				</div>
				<Button type="submit" disabled={!file || uploading}>
					{uploading ? "Converting…" : "Upload"}
				</Button>
			</form>
		</Card.Content>
	</Card.Root>

	<Card.Root class="mb-8">
		<Card.Header><Card.Title>Create program</Card.Title></Card.Header>
		<Card.Content>
			<form class="grid gap-4" onsubmit={submitProgram}>
				<div class="grid gap-2">
					<Label for="program-title">Title</Label>
					<Input id="program-title" bind:value={programTitle} />
				</div>
				<div class="grid gap-2">
					<Label for="program-description">Description (optional)</Label>
					<Input id="program-description" bind:value={programDescription} />
				</div>
				<div class="grid gap-2">
					<Label for="cover">Cover image (optional, PNG/JPEG/WebP ≤ 5MB)</Label>
					<Input id="cover" type="file" accept="image/png,image/jpeg,image/webp"
						onchange={(event) => (cover = (event.currentTarget as HTMLInputElement).files?.[0] ?? null)} />
				</div>
				<Button type="submit" disabled={!programTitle.trim() || creatingProgram}>Create</Button>
			</form>
		</Card.Content>
	</Card.Root>

	<Card.Root class="mb-8">
		<Card.Header><Card.Title>Create category</Card.Title></Card.Header>
		<Card.Content>
			<form class="flex gap-2" onsubmit={submitCategory}>
				<Input bind:value={categoryName} placeholder="Morning" />
				<Button type="submit" disabled={!categoryName.trim()}>Add</Button>
			</form>
			{#if categories.length > 0}
				<ul class="mt-3 flex flex-wrap gap-2 text-sm">
					{#each categories as category (category.id)}
						<li class="rounded-full bg-muted px-3 py-1">{category.name}</li>
					{/each}
				</ul>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header><Card.Title>Library ({meditations.length})</Card.Title></Card.Header>
		<Card.Content>
			<ul class="grid gap-2">
				{#each meditations as meditation (meditation.id)}
					<li class="flex items-center justify-between rounded-md border p-3">
						<div>
							<p class="font-medium">{meditation.name}</p>
							<p class="text-sm text-muted-foreground">
								{(meditation.size_bytes / 1024).toFixed(0)} KB · Opus
							</p>
						</div>
						<div class="flex gap-2">
							<select class="rounded-md border bg-background px-2 py-1 text-sm"
								onchange={(event) => addTo(Number((event.currentTarget as HTMLSelectElement).dataset.program ?? 0), event)}
								data-program="0">
								<option value="">Add to…</option>
							</select>
							<a href="/player?id={meditation.id}">
								<Button variant="outline" size="sm" type="button">Play</Button>
							</a>
						</div>
					</li>
				{/each}
			</ul>
		</Card.Content>
	</Card.Root>
</main>
