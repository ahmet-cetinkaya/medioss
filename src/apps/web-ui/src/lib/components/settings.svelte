<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { Input } from "$lib/components/ui/input";
	import { deleteOwnAccount } from "$lib/api/identity";
	import { fetchMe } from "$lib/api/meditation";

	let email = $state("");
	let confirmText = $state("");
	let busy = $state(false);
	let error = $state("");

	onMount(async () => {
		try {
			const me = await fetchMe();
			email = me.email;
		} catch {
			window.location.assign("/login");
		}
	});

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
