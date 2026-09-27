<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { fetchContinueProgram, fetchMe, type ContinueProgram } from "$lib/api/meditation";

	let role = $state<string | null>(null);
	let ready = $state(false);
	let continueProgram = $state<ContinueProgram | null>(null);

	onMount(async () => {
		try {
			const me = await fetchMe();
			role = me.role;
		} catch {
			window.location.assign("/login");
			return;
		}
		ready = true;
		try {
			continueProgram = await fetchContinueProgram();
		} catch {
			// Continuing is a nicety, not core navigation — a failure here must
			// not block the rest of the home page.
		}
	});
</script>

{#if ready}
	<main class="mx-auto flex min-h-screen max-w-2xl flex-col justify-center gap-6 px-6">
		<h1 class="text-4xl font-semibold tracking-tight">MediOSS</h1>
		<p class="text-lg">A private, self-hosted space for meditation and rest.</p>
		{#if continueProgram}
			<Card.Root>
				<Card.Header>
					<Card.Title>Continue program</Card.Title>
					<Card.Description>{continueProgram.program.title}</Card.Description>
				</Card.Header>
				<Card.Content>
					<Button
						onclick={() =>
							(window.location.href = `/program/?id=${continueProgram!.program.id}`)}
					>
						Continue program
					</Button>
				</Card.Content>
			</Card.Root>
		{/if}
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
