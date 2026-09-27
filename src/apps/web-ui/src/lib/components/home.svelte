<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import { fetchMe } from "$lib/api/meditation";

	let role = $state<string | null>(null);
	let ready = $state(false);

	onMount(async () => {
		try {
			const me = await fetchMe();
			role = me.role;
		} catch {
			window.location.assign("/login");
			return;
		}
		ready = true;
	});
</script>

{#if ready}
	<main class="mx-auto flex min-h-screen max-w-2xl flex-col justify-center gap-6 px-6">
		<h1 class="text-4xl font-semibold tracking-tight">MediOSS</h1>
		<p class="text-lg">A private, self-hosted space for meditation and rest.</p>
		<div class="flex flex-wrap gap-3">
			<Button size="lg" onclick={() => (window.location.href = "/library")}>
				Begin
			</Button>
			{#if role === "admin"}
				<Button size="lg" variant="outline" onclick={() => (window.location.href = "/admin")}>
					Admin
				</Button>
			{/if}
		</div>
	</main>
{/if}
