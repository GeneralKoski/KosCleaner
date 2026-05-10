<script lang="ts">
	import { getVersion } from '@tauri-apps/api/app';

	let version = $state<string>('');

	$effect(() => {
		getVersion()
			.then((v) => (version = v))
			.catch(() => (version = 'unknown'));
	});
</script>

<main class="flex min-h-screen flex-col items-center justify-center bg-neutral-950 text-neutral-100">
	<h1 class="text-4xl font-bold tracking-tight">KosCleaner</h1>
	<p class="mt-3 text-neutral-400">Safe, transparent system cleanup.</p>
	<p class="mt-8 text-xs uppercase tracking-widest text-neutral-600">
		alpha {version ? `· v${version}` : ''}
	</p>
</main>
