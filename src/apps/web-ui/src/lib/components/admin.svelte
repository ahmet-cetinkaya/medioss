<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import * as Card from "$lib/components/ui/card";
	import { createUser, deleteUser, fetchUsers, setUserStatus, type IdentityUser } from "$lib/api/identity";
	import {
		addMember,
		assignCategory,
		createCategory,
		createProgram,
		fetchCategories,
		fetchLibrary,
		fetchMe,
		fetchPrograms,
		titleCaseFromFile,
		uploadAudio,
		type Category,
		type Meditation,
		type Program,
	} from "$lib/api/meditation";
	import {
		createSoundscapeCategory,
		deleteSound,
		deleteSoundscapeCategory,
		fetchSoundscapeCategories,
		fetchSounds,
		renameSoundscapeCategory,
		updateSound,
		uploadSound,
		type SoundscapeCategory,
		type SoundscapeSound,
	} from "$lib/api/soundscape";

	let me = $state<{ role: string } | null>(null);
	let meditations = $state<Meditation[]>([]);
	let categories = $state<Category[]>([]);
	let programs = $state<Program[]>([]);
	let error = $state("");
	let message = $state("");

	let file = $state<File | null>(null);
	let name = $state("");
	let uploadProgramId = $state<number | null>(null);
	let uploading = $state(false);

	let programTitle = $state("");
	let programDescription = $state("");
	let cover = $state<File | null>(null);
	let creatingProgram = $state(false);

	let categoryName = $state("");
	let soundscapeCategories = $state<SoundscapeCategory[]>([]);
	let soundscapeSounds = $state<SoundscapeSound[]>([]);
	let soundscapeCategoryName = $state("");
	let soundscapeFile = $state<File | null>(null);
	let soundscapeName = $state("");
	let soundscapeCategoryId = $state<number | null>(null);

	let users = $state<IdentityUser[]>([]);
	let newUserEmail = $state("");
	let newUserPassword = $state("");
	let creatingUser = $state(false);
	let usersError = $state("");
	let createdPassword = $state("");
	let createdEmail = $state("");
	let lifecycleBusy = $state<number | null>(null);
	let lifecycleError = $state("");

	onMount(async () => {
		try {
			me = await fetchMe();
			if (me.role !== "admin") {
				window.location.assign("/library");
				return;
			}
			await reload();
		} catch {
			window.location.assign("/login");
		}
	});

	async function reload() {
		const [library, cats, progs] = await Promise.all([
			fetchLibrary(),
			fetchCategories(),
			fetchPrograms(),
		]);
		meditations = library;
		categories = cats;
		programs = progs;
		[soundscapeCategories, soundscapeSounds, users] = await Promise.all([
			fetchSoundscapeCategories(),
			fetchSounds(),
			fetchUsers(),
		]);
	}

	async function toggleDisabled(user: IdentityUser) {
		lifecycleError = "";
		lifecycleBusy = user.id;
		try {
			await setUserStatus(user.id, !user.disabled);
			users = await fetchUsers();
		} catch (failure) {
			lifecycleError = lifecycleMessage(failure);
		} finally {
			lifecycleBusy = null;
		}
	}

	async function removeUser(user: IdentityUser) {
		lifecycleError = "";
		if (!window.confirm(`"${user.email}" kalıcı olarak silinecek: tüm verisi ve medyaları geri getirilemez. Onaylıyor musunuz?`)) return;
		lifecycleBusy = user.id;
		try {
			await deleteUser(user.id);
			users = await fetchUsers();
		} catch (failure) {
			lifecycleError = lifecycleMessage(failure);
		} finally {
			lifecycleBusy = null;
		}
	}

	function lifecycleMessage(failure: unknown): string {
		const status = (failure as { status?: number }).status;
		if (status === 400) return "Bu işlem son aktif yöneticiyi ortadan kaldırırdı.";
		if (status === 403) return "Bu işlem için yönetici yetkisi gerekir.";
		return "İşlem başarısız oldu.";
	}

	async function submitNewUser(event: SubmitEvent) {
		event.preventDefault();
		usersError = "";
		createdPassword = "";
		createdEmail = "";
		creatingUser = true;
		try {
			await createUser(newUserEmail.trim(), newUserPassword);
			createdEmail = newUserEmail.trim();
			createdPassword = newUserPassword;
			newUserEmail = "";
			newUserPassword = "";
			users = await fetchUsers();
		} catch (failure) {
			const status = (failure as { status?: number }).status;
			if (status === 409) {
				usersError = "Bu e-posta adresi zaten kayıtlı.";
			} else if (status === 400) {
				usersError = "Geçici şifre en az 8 karakter olmalı ve e-posta geçerli olmalı.";
			} else {
				usersError =
					failure instanceof Error ? failure.message : "Kullanıcı oluşturulamadı";
			}
		} finally {
			creatingUser = false;
		}
	}

	async function createSoundCategory() {
		if (!soundscapeCategoryName.trim()) return;
		try {
			await createSoundscapeCategory(soundscapeCategoryName);
			soundscapeCategoryName = "";
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "Could not create sound category.";
		}
	}

	async function removeSoundCategory(category: SoundscapeCategory) {
		if (!window.confirm(`Delete ${category.name}? Categories containing sounds cannot be deleted.`)) return;
		try {
			await deleteSoundscapeCategory(category.id);
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "Could not delete sound category.";
		}
	}

	async function renameSoundCategory(category: SoundscapeCategory) {
		const name = window.prompt("Category name", category.name);
		if (!name?.trim()) return;
		try {
			await renameSoundscapeCategory(category.id, name);
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "Could not rename sound category.";
		}
	}

	async function editSound(sound: SoundscapeSound) {
		const name = window.prompt("Sound name", sound.name);
		if (!name?.trim()) return;
		const categoryText = window.prompt("Category id", String(sound.category_id));
		const categoryId = Number(categoryText);
		if (!Number.isInteger(categoryId) || categoryId <= 0) return;
		try {
			await updateSound(sound.id, name, categoryId);
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "Could not update sound.";
		}
	}

	async function removeSound(sound: SoundscapeSound) {
		if (!window.confirm(`Delete ${sound.name}? It will be removed from saved mixes.`)) return;
		try {
			await deleteSound(sound.id);
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "Could not delete sound.";
		}
	}

	async function submitSoundUpload(event: SubmitEvent) {
		event.preventDefault();
		if (!soundscapeFile || !soundscapeCategoryId || !soundscapeName.trim()) return;
		try {
			await uploadSound(soundscapeName, soundscapeCategoryId, soundscapeFile);
			soundscapeFile = null;
			soundscapeName = "";
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "Could not upload sound.";
		}
	}

	function onFileChosen(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		file = input.files?.[0] ?? null;
		if (file && !name.trim()) {
			// Title Case prefill from the file name, still editable (FR-024).
			name = titleCaseFromFile(file.name);
		}
	}

	async function submitUpload(event: SubmitEvent) {
		event.preventDefault();
		if (!file || !uploadProgramId || uploading) return;
		uploading = true;
		error = "";
		message = "";
		try {
			const created = await uploadAudio(file, name, uploadProgramId);
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

	async function addToProgram(meditationId: number, event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		const programId = Number(select.value);
		if (!programId) return;
		try {
			await addMember(programId, meditationId);
			select.value = "";
			message = "Added to program";
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "could not add member";
		}
	}

	async function assignToCategory(programId: number, event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		const raw = select.value;
		const categoryId = raw === "" ? null : Number(raw);
		try {
			await assignCategory(programId, categoryId);
			message = "Category updated";
			await reload();
		} catch (cause) {
			error = cause instanceof Error ? cause.message : "could not assign category";
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
		<Card.Header><Card.Title>Users</Card.Title>
			<Card.Description>Create accounts and review who has access.</Card.Description>
		</Card.Header>
		<Card.Content>
			<form class="grid gap-4" onsubmit={submitNewUser}>
				<div class="grid gap-2">
					<Label for="new-user-email">Email</Label>
					<Input
						id="new-user-email"
						type="email"
						bind:value={newUserEmail}
						placeholder="member@example.com"
						required
					/>
				</div>
				<div class="grid gap-2">
					<Label for="new-user-password">Temporary password</Label>
					<Input
						id="new-user-password"
						bind:value={newUserPassword}
						placeholder="min. 8 characters"
						minlength={8}
						required
					/>
				</div>
				{#if usersError}<p class="text-sm text-destructive">{usersError}</p>{/if}
				{#if lifecycleError}<p class="text-sm text-destructive">{lifecycleError}</p>{/if}
				<Button type="submit" disabled={!newUserEmail || !newUserPassword || creatingUser}>
					{creatingUser ? "Creating…" : "Create user"}
				</Button>
			</form>
			{#if createdPassword}
				<div class="mt-4 rounded-md border bg-muted/50 p-4 text-sm">
					<p class="font-medium">{createdEmail} created.</p>
					<p class="mt-1">Temporary password (shown once):</p>
					<code class="mt-1 block select-all font-mono">{createdPassword}</code>
					<p class="mt-2 text-muted-foreground">
						Share it securely — the user must change it on first login.
					</p>
				</div>
			{/if}
			{#if users.length > 0}
				<table class="mt-6 w-full text-sm">
					<thead>
						<tr class="border-b text-left">
							<th class="py-2">Email</th>
							<th class="py-2">Role</th>
							<th class="py-2">Status</th>
							<th class="py-2 text-right">Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each users as user (user.id)}
							<tr class="border-b">
								<td class="py-2">{user.email}</td>
								<td class="py-2">{user.role}</td>
								<td class="py-2">
									{#if user.disabled}<span class="text-destructive">disabled</span>{:else if user.must_change_password}must change password{:else}active{/if}
								</td>
								<td class="py-2 text-right">
									<Button variant="outline" size="sm" disabled={lifecycleBusy === user.id} onclick={() => void toggleDisabled(user)}>{user.disabled ? "Enable" : "Disable"}</Button>
									<Button variant="outline" size="sm" class="ml-2" disabled={lifecycleBusy === user.id} onclick={() => void removeUser(user)}>Delete</Button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root class="mb-8">
		<Card.Header><Card.Title>Upload meditation audio</Card.Title>
			<Card.Description>MP3, WAV or OGG up to 100MB; stored as Opus.</Card.Description>
		</Card.Header>
		<Card.Content>
			<form class="grid gap-4" onsubmit={submitUpload}>
				<div class="grid gap-2">
					<Label for="upload-program">Program (the file is stored in its folder)</Label>
					<select
						id="upload-program"
						bind:value={uploadProgramId}
						class="rounded-md border bg-background px-3 py-2 text-sm"
					>
						<option value={null} disabled>Select a program…</option>
						{#each programs as program (program.id)}
							<option value={program.id}>{program.title}</option>
						{/each}
					</select>
					{#if programs.length === 0}
						<p class="text-sm text-muted-foreground">
							Create a program first — uploads belong to one.
						</p>
					{/if}
				</div>
				<div class="grid gap-2">
					<Label for="file">Audio file</Label>
					<Input id="file" type="file" accept=".mp3,.wav,.ogg,.opus" onchange={onFileChosen} />
				</div>
				<div class="grid gap-2">
					<Label for="name">Name</Label>
					<Input id="name" bind:value={name} placeholder="Morning Meditation" />
				</div>
				<Button type="submit" disabled={!file || !uploadProgramId || uploading}>
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

	{#if programs.length > 0}
		<Card.Root class="mb-8">
			<Card.Header><Card.Title>Programs</Card.Title>
				<Card.Description>Assign categories and meditations.</Card.Description>
			</Card.Header>
			<Card.Content>
				<ul class="grid gap-3">
					{#each programs as program (program.id)}
						<li class="rounded-md border p-3">
							<div class="flex flex-wrap items-center justify-between gap-2">
								<p class="font-medium">{program.title}</p>
								<select
									class="rounded-md border bg-background px-2 py-1 text-sm"
									value={program.category_id ?? ""}
									onchange={(event) => assignToCategory(program.id, event)}
								>
									<option value="">Uncategorized</option>
									{#each categories as category (category.id)}
										<option value={category.id}>{category.name}</option>
									{/each}
								</select>
							</div>
						</li>
					{/each}
				</ul>
			</Card.Content>
		</Card.Root>
	{/if}

	<Card.Root>
		<Card.Header><Card.Title>Library ({meditations.length})</Card.Title></Card.Header>
		<Card.Content>
			<ul class="grid gap-2">
				{#each meditations as meditation (meditation.id)}
					<li class="flex flex-wrap items-center justify-between gap-2 rounded-md border p-3">
						<div>
							<p class="font-medium">{meditation.name}</p>
							<p class="text-sm text-muted-foreground">
								{(meditation.size_bytes / 1024).toFixed(0)} KB · Opus
							</p>
						</div>
						<div class="flex gap-2">
							{#if programs.length > 0}
								<select
									class="rounded-md border bg-background px-2 py-1 text-sm"
									onchange={(event) => addToProgram(meditation.id, event)}
								>
									<option value="">Add to…</option>
									{#each programs as program (program.id)}
										<option value={program.id}>{program.title}</option>
									{/each}
								</select>
							{/if}
							<a href="/player/?id={meditation.id}">
								<Button variant="outline" size="sm" type="button">Play</Button>
							</a>
						</div>
					</li>
				{/each}
			</ul>
		</Card.Content>
	</Card.Root>

	<section class="grid gap-4 border-t pt-8">
		<h2 class="text-2xl font-semibold">Soundscape library</h2>
		<form class="flex flex-wrap gap-2" onsubmit={(event) => { event.preventDefault(); createSoundCategory(); }}>
			<Input bind:value={soundscapeCategoryName} placeholder="New sound category" aria-label="New sound category" />
			<Button type="submit">Add category</Button>
		</form>
		<ul class="flex flex-wrap gap-2">
			{#each soundscapeCategories as category (category.id)}
				<li class="flex items-center gap-1 rounded border px-2 py-1 text-sm"><span>{category.name}</span><Button size="sm" variant="ghost" onclick={() => renameSoundCategory(category)}>Rename</Button><Button size="sm" variant="ghost" onclick={() => removeSoundCategory(category)}>Delete</Button></li>
			{/each}
		</ul>
		<form class="grid gap-3 rounded border p-4" onsubmit={submitSoundUpload}>
			<Label for="sound-name">Sound name</Label><Input id="sound-name" bind:value={soundscapeName} required />
			<Label for="sound-category">Category</Label><select id="sound-category" class="rounded border p-2" bind:value={soundscapeCategoryId} required><option value={null}>Choose a category</option>{#each soundscapeCategories as category}<option value={category.id}>{category.name}</option>{/each}</select>
			<Label for="sound-file">Audio file</Label><Input id="sound-file" type="file" accept="audio/*,.mp3,.wav,.ogg" onchange={(event) => { soundscapeFile = (event.currentTarget as HTMLInputElement).files?.[0] ?? null; }} required />
			<Button type="submit">Upload sound</Button>
		</form>
		<ul class="grid gap-2">{#each soundscapeSounds as sound (sound.id)}<li class="flex flex-wrap items-center justify-between gap-2 rounded border p-3"><span>{sound.name} · {soundscapeCategories.find((category) => category.id === sound.category_id)?.name ?? "Unknown"}{#if sound.builtin}<span class="ml-1 rounded bg-muted px-1 text-xs text-muted-foreground">built-in</span>{/if}</span><span class="flex gap-2"><Button size="sm" variant="outline" onclick={() => editSound(sound)}>Edit</Button><Button size="sm" variant="destructive" onclick={() => removeSound(sound)}>Delete</Button></span></li>{/each}</ul>
	</section>
</main>
