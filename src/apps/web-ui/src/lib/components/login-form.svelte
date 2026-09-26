<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";

	let email = $state("");
	let password = $state("");
	let busy = $state(false);
	let error = $state("");
	let retryAfter = $state<number | null>(null);
	let countdown = $state(0);

	$effect(() => {
		if (retryAfter === null || countdown <= 0) return;
		const timer = setInterval(() => {
			countdown -= 1;
			if (countdown <= 0) retryAfter = null;
		}, 1000);
		return () => clearInterval(timer);
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (busy) return;
		busy = true;
		error = "";
		try {
			const response = await fetch("/api/identity/login", {
				method: "POST",
				headers: { "content-type": "application/json" },
				body: JSON.stringify({ email, password }),
			});
			if (response.ok) {
				window.location.assign("/");
				return;
			}
			const body = await response.json().catch(() => ({ error: "login failed" }));
			const retryHeader = Number(
				response.headers.get("retry-after") ?? body.retry_after ?? "0",
			);
			if (response.status === 429 && retryHeader > 0) {
				retryAfter = Math.ceil(retryHeader);
				countdown = Math.ceil(retryHeader);
			}
			error = body.error ?? "login failed";
		} catch {
			error = "could not reach the server";
		} finally {
			busy = false;
		}
	}
</script>

<form onsubmit={submit} class="mx-auto mt-24 max-w-sm space-y-4">
	<Card.Root>
		<Card.Header>
			<Card.Title>Sign in</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="space-y-2">
				<Label for="email">Email</Label>
				<Input id="email" type="email" bind:value={email} required />
			</div>
			<div class="space-y-2">
				<Label for="password">Password</Label>
				<Input id="password" type="password" bind:value={password} required />
			</div>
			{#if error}
				<p class="text-sm text-red-600">
					{error}
					{#if countdown > 0}(retry in {countdown}s){/if}
				</p>
			{/if}
			<Button type="submit" class="w-full" disabled={busy || retryAfter !== null}>
				{busy ? "Signing in…" : "Sign in"}
			</Button>
			<p class="text-xs text-neutral-500">
				Forgot your password? Ask the host operator to run
				<code>medioss reset-password</code>.
			</p>
		</Card.Content>
	</Card.Root>
</form>
