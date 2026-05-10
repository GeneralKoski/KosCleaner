<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { formatBytes } from '$lib/format';
	import type { CleanerInfo, ScanReport } from '$lib/types';

	type Category = {
		id: 'system' | 'browsers' | 'packages' | 'trash';
		label: string;
		// Step 6 implementa solo "system"; le altre restano disabilitate.
		comingSoon: boolean;
	};

	const categories: Category[] = [
		{ id: 'system', label: 'System', comingSoon: false },
		{ id: 'browsers', label: 'Browsers', comingSoon: true },
		{ id: 'packages', label: 'Packages', comingSoon: true },
		{ id: 'trash', label: 'Trash', comingSoon: true }
	];

	let activeCategory = $state<Category['id']>('system');
	let cleaners = $state<CleanerInfo[]>([]);
	let selectedCleanerId = $state<string | null>(null);
	let report = $state<ScanReport | null>(null);
	let scanning = $state(false);
	let scanError = $state<string | null>(null);

	$effect(() => {
		invoke<CleanerInfo[]>('list_cleaners')
			.then((list) => {
				cleaners = list;
				if (!selectedCleanerId && list.length > 0) {
					selectedCleanerId = list[0].id;
				}
			})
			.catch((e) => (scanError = String(e)));
	});

	const visibleCleaners = $derived(
		cleaners.filter((c) => c.category.toLowerCase() === activeCategory)
	);

	async function runScan() {
		if (!selectedCleanerId) return;
		scanning = true;
		scanError = null;
		report = null;
		try {
			report = await invoke<ScanReport>('scan_cleaner', { id: selectedCleanerId });
		} catch (e) {
			scanError = String(e);
		} finally {
			scanning = false;
		}
	}
</script>

<div class="flex h-screen bg-neutral-950 text-neutral-100">
	<aside class="flex w-64 flex-col border-r border-neutral-800 bg-neutral-900/40">
		<div class="border-b border-neutral-800 px-5 py-4">
			<h1 class="text-lg font-semibold tracking-tight">KosCleaner</h1>
			<p class="text-xs uppercase tracking-widest text-neutral-500">alpha</p>
		</div>
		<nav class="flex-1 overflow-y-auto py-3">
			{#each categories as cat (cat.id)}
				<button
					type="button"
					class="flex w-full items-center justify-between px-5 py-2.5 text-left text-sm transition
						{activeCategory === cat.id
						? 'bg-neutral-800 text-white'
						: 'text-neutral-400 hover:bg-neutral-800/40 hover:text-neutral-200'}
						{cat.comingSoon ? 'cursor-not-allowed opacity-50' : ''}"
					disabled={cat.comingSoon}
					onclick={() => {
						activeCategory = cat.id;
						const first = cleaners.find((c) => c.category.toLowerCase() === cat.id);
						selectedCleanerId = first?.id ?? null;
						report = null;
						scanError = null;
					}}
				>
					<span>{cat.label}</span>
					{#if cat.comingSoon}
						<span class="text-[10px] uppercase tracking-wider text-neutral-600">soon</span>
					{/if}
				</button>
			{/each}
		</nav>
	</aside>

	<main class="flex flex-1 flex-col overflow-hidden">
		<header class="flex items-center justify-between border-b border-neutral-800 px-8 py-5">
			<div>
				<h2 class="text-2xl font-semibold capitalize">{activeCategory}</h2>
				<p class="text-sm text-neutral-500">
					{visibleCleaners.length === 0
						? 'No cleaners available in this category yet.'
						: `${visibleCleaners.length} cleaner${visibleCleaners.length === 1 ? '' : 's'} available`}
				</p>
			</div>
			<button
				type="button"
				class="rounded-md bg-emerald-600 px-5 py-2 text-sm font-medium text-white transition
					hover:bg-emerald-500 disabled:cursor-not-allowed disabled:bg-neutral-700 disabled:text-neutral-500"
				disabled={!selectedCleanerId || scanning}
				onclick={runScan}
			>
				{scanning ? 'Scanning…' : 'Scan'}
			</button>
		</header>

		<section class="flex flex-1 flex-col overflow-hidden px-8 py-6">
			{#if visibleCleaners.length === 0}
				<div class="flex flex-1 items-center justify-center text-sm text-neutral-500">
					Coming soon.
				</div>
			{:else}
				<div class="mb-6 grid gap-3">
					{#each visibleCleaners as c (c.id)}
						<label
							class="flex cursor-pointer items-start gap-3 rounded-md border border-neutral-800 p-4 transition
								{selectedCleanerId === c.id
								? 'border-emerald-600/60 bg-emerald-600/5'
								: 'hover:border-neutral-700 hover:bg-neutral-900/40'}"
						>
							<input
								type="radio"
								name="cleaner"
								class="mt-1 accent-emerald-500"
								checked={selectedCleanerId === c.id}
								onchange={() => {
									selectedCleanerId = c.id;
									report = null;
									scanError = null;
								}}
							/>
							<div>
								<div class="text-sm font-medium">{c.name}</div>
								<div class="text-xs text-neutral-500">{c.description}</div>
							</div>
						</label>
					{/each}
				</div>

				{#if scanError}
					<div class="rounded-md border border-red-900/60 bg-red-950/40 px-4 py-3 text-sm text-red-300">
						{scanError}
					</div>
				{:else if report}
					<div class="flex items-baseline justify-between border-b border-neutral-800 pb-3">
						<div>
							<div class="text-xs uppercase tracking-widest text-neutral-500">Recoverable</div>
							<div class="text-3xl font-semibold tabular-nums">{formatBytes(report.total_size)}</div>
						</div>
						<div class="text-sm text-neutral-500">
							{report.items.length} item{report.items.length === 1 ? '' : 's'}
							{#if report.errors.length > 0}
								· <span class="text-amber-400">{report.errors.length} skipped</span>
							{/if}
						</div>
					</div>

					<div class="mt-4 flex-1 overflow-y-auto rounded-md border border-neutral-800">
						{#if report.items.length === 0}
							<div class="px-4 py-8 text-center text-sm text-neutral-500">
								Nothing to clean. Your temp directory is already empty.
							</div>
						{:else}
							<table class="w-full text-left text-xs">
								<thead class="sticky top-0 bg-neutral-900 text-neutral-400">
									<tr>
										<th class="px-4 py-2 font-medium">Path</th>
										<th class="px-4 py-2 text-right font-medium">Size</th>
									</tr>
								</thead>
								<tbody>
									{#each report.items.slice(0, 500) as item, i (item.path + i)}
										<tr class="border-t border-neutral-800/60">
											<td class="px-4 py-1.5 font-mono text-[11px] text-neutral-300">
												{item.path}
												{#if item.is_symlink}
													<span class="ml-2 text-[10px] uppercase text-neutral-500">symlink</span>
												{/if}
											</td>
											<td class="px-4 py-1.5 text-right tabular-nums text-neutral-400">
												{formatBytes(item.size)}
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
							{#if report.items.length > 500}
								<div class="border-t border-neutral-800/60 px-4 py-2 text-center text-[11px] text-neutral-500">
									+{report.items.length - 500} more items not shown
								</div>
							{/if}
						{/if}
					</div>

					{#if report.errors.length > 0}
						<details class="mt-3 rounded-md border border-amber-900/40 bg-amber-950/20 text-sm">
							<summary class="cursor-pointer px-4 py-2 text-amber-300">
								{report.errors.length} entries skipped
							</summary>
							<ul class="max-h-48 overflow-y-auto px-4 pb-3 text-xs text-amber-200/80">
								{#each report.errors as err, i (err.path + i)}
									<li class="border-t border-amber-900/30 py-1.5 font-mono">
										{err.path} — {err.message}
									</li>
								{/each}
							</ul>
						</details>
					{/if}
				{:else}
					<div class="flex flex-1 items-center justify-center text-sm text-neutral-500">
						Click <span class="mx-1.5 rounded bg-neutral-800 px-2 py-0.5 text-xs">Scan</span> to see what can be recovered.
					</div>
				{/if}
			{/if}
		</section>
	</main>
</div>
