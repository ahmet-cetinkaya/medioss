<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { Input } from "$lib/components/ui/input";
	import {
		deleteOwnAccount,
		fetchMeProfile,
		removeAvatar,
		updateDisplayName,
		uploadAvatar,
	} from "$lib/api/identity";
	import { importData } from "$lib/api/portability";
	import type { ImportReport } from "$lib/api/portability";

	let email = $state("");
	let confirmText = $state("");
	let busy = $state(false);
	let error = $state("");
	let displayName = $state("");
	let profileBusy = $state(false);
	let profileError = $state("");
	let profileSaved = $state(false);
	let hasAvatar = $state(false);
	let avatarVersion = $state(0);
	let myId = $state<number | null>(null);
	let importFile = $state<File | null>(null);
	let importBusy = $state(false);
	let importError = $state("");
	let importReport = $state<ImportReport | null>(null);

	async function onImportChosen(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		importFile = input.files?.[0] ?? null;
	}

	async function runImport() {
		if (!importFile) return;
		importBusy = true;
		importError = "";
		importReport = null;
		try {
			importReport = await importData(importFile);
		} catch (failure) {
			importError = failure instanceof Error ? failure.message : "İçe aktarma başarısız oldu.";
		} finally {
			importBusy = false;
		}
	}

	onMount(async () => {
		try {
			const me = await fetchMeProfile();
			myId = me.id;
			email = me.email;
			displayName = me.display_name ?? "";
			hasAvatar = me.has_avatar;
		} catch {
			window.location.assign("/login");
		}
	});

	async function saveDisplayName() {
		profileError = "";
		profileSaved = false;
		profileBusy = true;
		try {
			const me = await updateDisplayName(displayName);
			displayName = me.display_name ?? "";
			profileSaved = true;
		} catch (failure) {
			profileError = "Görünen ad kaydedilemedi.";
		} finally {
			profileBusy = false;
		}
	}

	async function onAvatarChange(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = "";
		if (!file) return;
		profileError = "";
		if (file.size > 2 * 1024 * 1024) {
			profileError = "Avatar en fazla 2 MB olabilir.";
			return;
		}
		profileBusy = true;
		try {
			await uploadAvatar(file);
			hasAvatar = true;
			avatarVersion += 1;
		} catch (failure) {
			profileError = "Avatar yüklenemedi. Yalnızca JPG, PNG veya WebP kabul edilir.";
		} finally {
			profileBusy = false;
		}
	}

	async function onAvatarRemove() {
		if (!window.confirm("Avatarınız kaldırılsın mı?")) return;
		profileBusy = true;
		profileError = "";
		try {
			await removeAvatar();
			hasAvatar = false;
		} catch (failure) {
			profileError = "Avatar kaldırılamadı.";
		} finally {
			profileBusy = false;
		}
	}

	async function selfDelete() {
		error = "";
		if (confirmText !== email) {
			error = "Onay için e-posta adresinizi eksiksiz yazmalısınız.";
			return;
		}
		if (!window.confirm("Hesabınız ve tüm verileriniz kalıcı olarak silinecek. Devam edilsin mi?")) return;
		busy = true;
		try {
			await deleteOwnAccount();
			window.location.assign("/login");
		} catch (failure) {
			const status = (failure as { status?: number }).status;
			if (status === 400) error = "Son aktif yönetici hesabını silemezsiniz.";
			else error = "Hesap silinemedi.";
			busy = false;
		}
	}
</script>

<main class="mx-auto min-h-screen w-full max-w-3xl px-4 py-10">
	<header class="mb-8 flex items-baseline justify-between">
		<h1 class="text-3xl font-semibold tracking-tight">Settings</h1>
		<Button variant="outline" size="sm" onclick={() => (window.location.href = "/")}>Home</Button>
	</header>

	<Card.Root class="mb-6">
		<Card.Header>
			<Card.Title>Profile</Card.Title>
			<Card.Description>Görünen adınız ve avatarınız yalnızca sizin ve yöneticinin görebildiği bilgilerdir.</Card.Description>
		</Card.Header>
		<Card.Content class="grid gap-4">
			<div class="flex items-center gap-4">
				{#if hasAvatar}
					<img
						src={`/api/identity/avatar/{myId}?v={avatarVersion}`}
						alt="Avatar"
						class="h-16 w-16 rounded-full object-cover"
					/>
				{:else}
					<div class="flex h-16 w-16 items-center justify-center rounded-full bg-muted text-xl font-medium">
						{(displayName || email).charAt(0).toLocaleUpperCase("tr")}
					</div>
				{/if}
				<div class="grid gap-2">
					<Input
						type="file"
						accept="image/jpeg,image/png,image/webp"
						aria-label="Avatar yükle"
						onchange={(event) => void onAvatarChange(event)}
						disabled={profileBusy}
					/>
					{#if hasAvatar}
						<Button variant="outline" size="sm" disabled={profileBusy} onclick={() => void onAvatarRemove()}>
							Avatarı kaldır
						</Button>
					{/if}
				</div>
			</div>
			<div class="grid gap-2">
				<label class="text-sm font-medium" for="display-name">Görünen ad</label>
				<Input id="display-name" bind:value={displayName} placeholder={email} maxlength="64" disabled={profileBusy} />
				{#if profileSaved}<p class="text-sm text-muted-foreground">Kaydedildi.</p>{/if}
				{#if profileError}<p class="text-sm text-destructive">{profileError}</p>{/if}
				<Button disabled={profileBusy} onclick={() => void saveDisplayName()}>
					{profileBusy ? "Kaydediliyor…" : "Görünen adı kaydet"}
				</Button>
			</div>
		</Card.Content>
	</Card.Root>

	<Card.Root class="mb-6">
		<Card.Header>
			<Card.Title>Your data</Card.Title>
			<Card.Description>
				Verilerinizi JSON (tüm veriler) veya CSV (yalnızca mood kayıtları) olarak indirin;
				daha önce dışa aktarılmış bir dosyayı geri yükleyin.
			</Card.Description>
		</Card.Header>
		<Card.Content class="grid gap-4">
			<div class="flex flex-wrap gap-2">
				<Button variant="outline" asChild>
					<a href="/api/portability/export/json">Download JSON</a>
				</Button>
				<Button variant="outline" asChild>
					<a href="/api/portability/export/csv">Download CSV</a>
				</Button>
			</div>
			<div class="grid gap-2">
				<label class="text-sm font-medium" for="import-file">Import a previous export</label>
				<Input id="import-file" type="file" accept=".json,.csv" onchange={onImportChosen} />
				<Button disabled={!importFile || importBusy} onclick={() => void runImport()}>
					{importBusy ? "Importing…" : "Import"}
				</Button>
			</div>
			{#if importReport}
				<p class="text-sm text-muted-foreground">
					Mood kayıtları eklendi: {importReport.mood_created}, yinelenen atlandı:
					{importReport.mood_duplicates_skipped}; alıştırma kayıtları eşleşti:
					{importReport.practice_linked}, eşleşmedi: {importReport.practice_skipped}.
				</p>
			{/if}
			{#if importError}<p class="text-sm text-destructive">{importError}</p>{/if}
		</Card.Content>
	</Card.Root>

	<Card.Root class="border-destructive">
		<Card.Header>
			<Card.Title>Delete account</Card.Title>
			<Card.Description>
				Hesabınız, tüm mood kayıtlarınız, meditasyon geçmişiniz, notlarınız, mix'leriniz ve
				medyalarınız kalıcı olarak silinir. Bu işlem geri alınamaz.
			</Card.Description>
		</Card.Header>
		<Card.Content class="grid gap-4">
			<p class="text-sm text-muted-foreground">
				Onaylamak için e-posta adresinizi yazın: <span class="font-medium">{email}</span>
			</p>
			<div class="grid gap-2">
				<Input bind:value={confirmText} placeholder={email} aria-label="Confirm with your email" />
			</div>
			{#if error}<p class="text-sm text-destructive">{error}</p>{/if}
			<Button variant="destructive" disabled={busy || confirmText !== email} onclick={() => void selfDelete()}>
				{busy ? "Deleting…" : "Delete my account permanently"}
			</Button>
		</Card.Content>
	</Card.Root>
</main>
