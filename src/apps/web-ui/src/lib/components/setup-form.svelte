<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";

	let sessionSecret = $state("");
	let email = $state("");
	let password = $state("");
	let busy = $state(false);
	let error = $state("");
	let done = $state(false);
	let alreadyDone = $state(false);

	$effect(() => {
		fetch("/api/identity/setup-status")
			.then((response) => response.json())
			.then((body: { setup_required: boolean }) => {
				if (!body.setup_required) alreadyDone = true;
			})
			.catch(() => {
				error = "could not reach the server";
			});
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (busy) return;
		busy = true;
		error = "";
		try {
			const response = await fetch("/api/identity/setup", {
				method: "POST",
				headers: {
					"content-type": "application/json",
					"x-setup-secret": sessionSecret,
				},
				body: JSON.stringify({ email, password }),
			});
			if (response.ok) {
				done = true;
			} else {
				const body = await response.json().catch(() => ({ error: "setup failed" }));
				error = body.error ?? "setup failed";
			}
		} catch {
			error = "could not reach the server";
		} finally {
			// The secret lives only in component memory and is cleared right
			// after the response, never persisted to storage.
			sessionSecret = "";
			busy = false;
		}
	}
</script>

{#if alreadyDone}
	<div class="mx-auto mt-24 max-w-sm p-4 text-center text-sm text-neutral-500">
		This instance is already set up. Continue to <a class="underline" href="/login">login</a>.
	</div>
{:else if done}
	<div class="mx-auto mt-24 max-w-sm p-4 text-center text-sm text-neutral-500">
		Administrator created. Continue to <a class="underline" href="/login">login</a>.
	</div>
{:else}
	<form onsubmit={submit} class="mx-auto mt-24 max-w-sm space-y-4">
		<Card.Root>
			<Card.Header>
				<Card.Title>First-run setup</Card.Title>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="space-y-2">
					<Label for="session-secret">Session secret</Label>
					<Input
						id="session-secret"
						type="password"
						autocomplete="off"
						bind:value={sessionSecret}
						required
					/>
					<p class="text-xs text-neutral-500">
						Prove host possession with this instance's SESSION_SECRET. It is never stored by the browser.
					</p>
				</div>
				<div class="space-y-2">
					<Label for="email">Admin email</Label>
					<Input id="email" type="email" bind:value={email} required />
				</div>
				<div class="space-y-2">
					<Label for="password">Admin password</Label>
					<Input id="password" type="password" bind:value={password} required minlength={8} />
				</div>
				{#if error}
					<p class="text-sm text-red-600">{error}</p>
				{/if}
				<Button type="submit" class="w-full" disabled={busy}>
					{busy ? "Creating…" : "Create administrator"}
				</Button>
			</Card.Content>
		</Card.Root>
	</form>
{/if}
