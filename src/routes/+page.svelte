<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { formatBytes } from '$lib/format';
	import type { CleanerInfo, ExecutionReport, ScanReport } from '$lib/types';
	import { locale, t, type Locale } from '$lib/i18n';
	import { theme, type Theme } from '$lib/theme';

	type Category = {
		id: 'system' | 'browsers' | 'packages' | 'trash';
		labelKey:
			| 'category_system'
			| 'category_browsers'
			| 'category_packages'
			| 'category_trash';
		comingSoon: boolean;
	};

	const categories: Category[] = [
		{ id: 'system', labelKey: 'category_system', comingSoon: false },
		{ id: 'browsers', labelKey: 'category_browsers', comingSoon: false },
		{ id: 'packages', labelKey: 'category_packages', comingSoon: false },
		{ id: 'trash', labelKey: 'category_trash', comingSoon: false }
	];

	let activeCategory = $state<Category['id']>('system');
	let cleaners = $state<CleanerInfo[]>([]);
	let selectedCleanerId = $state<string | null>(null);
	let report = $state<ScanReport | null>(null);
	let scanning = $state(false);
	let scanError = $state<string | null>(null);

	let confirmOpen = $state(false);
	let executing = $state(false);
	let execReport = $state<ExecutionReport | null>(null);
	let execError = $state<string | null>(null);

	$effect(() => {
		invoke<CleanerInfo[]>('list_cleaners')
			.then((list) => {
				cleaners = list;
				if (!selectedCleanerId && list.length > 0) {
					const first = list.find((c) => c.category.toLowerCase() === activeCategory);
					selectedCleanerId = first?.id ?? null;
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
		execReport = null;
		execError = null;
		try {
			report = await invoke<ScanReport>('scan_cleaner', { id: selectedCleanerId });
		} catch (e) {
			scanError = String(e);
		} finally {
			scanning = false;
		}
	}

	async function runExecute() {
		if (!selectedCleanerId || !report || report.items.length === 0) return;
		executing = true;
		execError = null;
		try {
			execReport = await invoke<ExecutionReport>('execute_cleaner', {
				id: selectedCleanerId,
				paths: report.items.map((i) => i.path)
			});
			confirmOpen = false;
			report = null;
		} catch (e) {
			execError = String(e);
		} finally {
			executing = false;
		}
	}

	function setLocale(l: Locale) {
		locale.set(l);
	}
	function setTheme(v: Theme) {
		theme.set(v);
	}
</script>

<div class="flex h-screen bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
	<aside
		class="flex w-64 flex-col border-r border-neutral-200 bg-neutral-50 dark:border-neutral-800 dark:bg-neutral-900/40"
	>
		<div class="border-b border-neutral-200 px-5 py-4 dark:border-neutral-800">
			<h1 class="text-lg font-semibold tracking-tight">KosCleaner</h1>
			<p class="text-xs uppercase tracking-widest text-neutral-500">{$t('alpha')}</p>
		</div>
		<nav class="flex-1 overflow-y-auto py-3">
			{#each categories as cat (cat.id)}
				<button
					type="button"
					class="flex w-full items-center justify-between px-5 py-2.5 text-left text-sm transition
						{activeCategory === cat.id
						? 'bg-neutral-200 text-neutral-900 dark:bg-neutral-800 dark:text-white'
						: 'text-neutral-600 hover:bg-neutral-100 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/40 dark:hover:text-neutral-200'}
						{cat.comingSoon ? 'cursor-not-allowed opacity-50' : ''}"
					disabled={cat.comingSoon}
					onclick={() => {
						activeCategory = cat.id;
						const first = cleaners.find((c) => c.category.toLowerCase() === cat.id);
						selectedCleanerId = first?.id ?? null;
						report = null;
						scanError = null;
						execReport = null;
					}}
				>
					<span>{$t(cat.labelKey)}</span>
					{#if cat.comingSoon}
						<span class="text-[10px] uppercase tracking-wider text-neutral-500">{$t('soon')}</span>
					{/if}
				</button>
			{/each}
		</nav>
		<div
			class="flex items-center justify-between gap-2 border-t border-neutral-200 px-5 py-3 text-xs dark:border-neutral-800"
		>
			<div class="flex gap-1">
				<button
					type="button"
					class="rounded px-2 py-1 transition {$locale === 'en'
						? 'bg-neutral-200 text-neutral-900 dark:bg-neutral-700 dark:text-white'
						: 'text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200'}"
					onclick={() => setLocale('en')}
				>
					EN
				</button>
				<button
					type="button"
					class="rounded px-2 py-1 transition {$locale === 'it'
						? 'bg-neutral-200 text-neutral-900 dark:bg-neutral-700 dark:text-white'
						: 'text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200'}"
					onclick={() => setLocale('it')}
				>
					IT
				</button>
			</div>
			<button
				type="button"
				class="rounded px-2 py-1 text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200"
				onclick={() => setTheme($theme === 'dark' ? 'light' : 'dark')}
				aria-label={$theme === 'dark' ? $t('theme_light') : $t('theme_dark')}
				title={$theme === 'dark' ? $t('theme_light') : $t('theme_dark')}
			>
				{$theme === 'dark' ? '☀' : '☾'}
			</button>
		</div>
	</aside>

	<main class="flex flex-1 flex-col overflow-hidden">
		<header
			class="flex items-center justify-between border-b border-neutral-200 px-8 py-5 dark:border-neutral-800"
		>
			<div>
				<h2 class="text-2xl font-semibold">
					{$t(`category_${activeCategory}` as 'category_system')}
				</h2>
				<p class="text-sm text-neutral-500">
					{visibleCleaners.length === 0
						? $t('no_cleaners')
						: $t('cleaners_available', { n: visibleCleaners.length })}
				</p>
			</div>
			<div class="flex gap-2">
				<button
					type="button"
					class="rounded-md border border-neutral-300 px-4 py-2 text-sm font-medium transition
						hover:bg-neutral-100 disabled:cursor-not-allowed disabled:opacity-50
						dark:border-neutral-700 dark:hover:bg-neutral-800"
					disabled={!selectedCleanerId || scanning}
					onclick={runScan}
				>
					{scanning ? $t('scanning') : $t('scan')}
				</button>
				<button
					type="button"
					class="rounded-md bg-emerald-600 px-5 py-2 text-sm font-medium text-white transition
						hover:bg-emerald-500 disabled:cursor-not-allowed disabled:bg-neutral-300 disabled:text-neutral-500
						dark:disabled:bg-neutral-700"
					disabled={!report || report.items.length === 0 || scanning}
					onclick={() => (confirmOpen = true)}
				>
					{$t('clean')}
				</button>
			</div>
		</header>

		<section class="flex flex-1 flex-col overflow-hidden px-8 py-6">
			{#if visibleCleaners.length === 0}
				<div class="flex flex-1 items-center justify-center text-sm text-neutral-500">
					{$t('coming_soon')}
				</div>
			{:else}
				<div class="mb-6 grid gap-3">
					{#each visibleCleaners as c (c.id)}
						<label
							class="flex cursor-pointer items-start gap-3 rounded-md border p-4 transition
								{selectedCleanerId === c.id
								? 'border-emerald-600/60 bg-emerald-50 dark:bg-emerald-600/5'
								: 'border-neutral-200 hover:border-neutral-300 hover:bg-neutral-50 dark:border-neutral-800 dark:hover:border-neutral-700 dark:hover:bg-neutral-900/40'}"
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
									execReport = null;
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
					<div
						class="rounded-md border border-red-300 bg-red-50 px-4 py-3 text-sm text-red-700
							dark:border-red-900/60 dark:bg-red-950/40 dark:text-red-300"
					>
						{scanError}
					</div>
				{:else if report}
					<div
						class="flex items-baseline justify-between border-b border-neutral-200 pb-3 dark:border-neutral-800"
					>
						<div>
							<div class="text-xs uppercase tracking-widest text-neutral-500">{$t('recoverable')}</div>
							<div class="text-3xl font-semibold tabular-nums">{formatBytes(report.total_size)}</div>
						</div>
						<div class="text-sm text-neutral-500">
							{$t('items_count', { n: report.items.length })}
							{#if report.errors.length > 0}
								·
								<span class="text-amber-600 dark:text-amber-400"
									>{$t('skipped', { n: report.errors.length })}</span
								>
							{/if}
						</div>
					</div>

					<div
						class="mt-4 flex-1 overflow-y-auto rounded-md border border-neutral-200 dark:border-neutral-800"
					>
						{#if report.items.length === 0}
							<div class="px-4 py-8 text-center text-sm text-neutral-500">
								{$t('nothing_to_clean')}
							</div>
						{:else}
							<table class="w-full text-left text-xs">
								<thead
									class="sticky top-0 bg-neutral-100 text-neutral-600 dark:bg-neutral-900 dark:text-neutral-400"
								>
									<tr>
										<th class="px-4 py-2 font-medium">{$t('path')}</th>
										<th class="px-4 py-2 text-right font-medium">{$t('size')}</th>
									</tr>
								</thead>
								<tbody>
									{#each report.items.slice(0, 500) as item, i (item.path + i)}
										<tr class="border-t border-neutral-100 dark:border-neutral-800/60">
											<td class="px-4 py-1.5 font-mono text-[11px] text-neutral-700 dark:text-neutral-300">
												{item.path}
												{#if item.is_symlink}
													<span class="ml-2 text-[10px] uppercase text-neutral-500">symlink</span>
												{/if}
											</td>
											<td class="px-4 py-1.5 text-right tabular-nums text-neutral-500">
												{formatBytes(item.size)}
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
							{#if report.items.length > 500}
								<div
									class="border-t border-neutral-100 px-4 py-2 text-center text-[11px] text-neutral-500 dark:border-neutral-800/60"
								>
									{$t('more_items', { n: report.items.length - 500 })}
								</div>
							{/if}
						{/if}
					</div>

					{#if report.errors.length > 0}
						<details
							class="mt-3 rounded-md border border-amber-300 bg-amber-50 text-sm
								dark:border-amber-900/40 dark:bg-amber-950/20"
						>
							<summary class="cursor-pointer px-4 py-2 text-amber-700 dark:text-amber-300">
								{$t('entries_skipped', { n: report.errors.length })}
							</summary>
							<ul class="max-h-48 overflow-y-auto px-4 pb-3 text-xs text-amber-800/80 dark:text-amber-200/80">
								{#each report.errors as err, i (err.path + i)}
									<li class="border-t border-amber-200/60 py-1.5 font-mono dark:border-amber-900/30">
										{err.path} — {err.message}
									</li>
								{/each}
							</ul>
						</details>
					{/if}
				{:else if execReport}
					<div class="flex flex-1 flex-col items-center justify-center gap-3 text-center">
						<div class="text-xs uppercase tracking-widest text-emerald-600 dark:text-emerald-400">
							{$t('done')}
						</div>
						<div class="text-3xl font-semibold tabular-nums">
							{$t('freed', { size: formatBytes(execReport.freed_bytes) })}
						</div>
						<div class="text-sm text-neutral-500">
							{$t('deleted_summary', {
								deleted: execReport.deleted,
								attempted: execReport.attempted,
								n: execReport.attempted
							})}
							{#if execReport.failures.length > 0}
								·
								<span class="text-amber-600 dark:text-amber-400"
									>{$t('failed', { n: execReport.failures.length })}</span
								>
							{/if}
						</div>
						{#if execReport.audit_log_path}
							<div class="font-mono text-[10px] text-neutral-500">
								{$t('audit_log', { path: execReport.audit_log_path })}
							</div>
						{/if}
						{#if execReport.failures.length > 0}
							<details
								class="mt-2 max-w-2xl rounded-md border border-amber-300 bg-amber-50 text-left text-sm
									dark:border-amber-900/40 dark:bg-amber-950/20"
							>
								<summary class="cursor-pointer px-4 py-2 text-amber-700 dark:text-amber-300">
									{$t('show_failures', { n: execReport.failures.length })}
								</summary>
								<ul
									class="max-h-48 overflow-y-auto px-4 pb-3 text-xs text-amber-800/80 dark:text-amber-200/80"
								>
									{#each execReport.failures as err, i (err.path + i)}
										<li class="border-t border-amber-200/60 py-1.5 font-mono dark:border-amber-900/30">
											{err.path} — {err.message}
										</li>
									{/each}
								</ul>
							</details>
						{/if}
					</div>
				{:else}
					<div class="flex flex-1 items-center justify-center text-sm text-neutral-500">
						{@html $t('empty_intro', {
							scan: `<span class="mx-1.5 rounded bg-neutral-200 px-2 py-0.5 text-xs dark:bg-neutral-800">${$t('scan')}</span>`
						})}
					</div>
				{/if}
			{/if}
		</section>
	</main>
</div>

{#if confirmOpen && report}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
		role="dialog"
		aria-modal="true"
		aria-labelledby="confirm-title"
	>
		<div
			class="w-full max-w-md rounded-lg border border-neutral-200 bg-white p-6 shadow-2xl
				dark:border-neutral-800 dark:bg-neutral-950"
		>
			<h3 id="confirm-title" class="text-lg font-semibold">{$t('confirm_title')}</h3>
			<p class="mt-2 text-sm text-neutral-500">
				{$t('confirm_body', {
					n: report.items.length,
					size: formatBytes(report.total_size)
				})}
			</p>
			<div
				class="mt-4 max-h-40 overflow-y-auto rounded border border-neutral-200 bg-neutral-50 p-2 font-mono text-[11px] text-neutral-600
					dark:border-neutral-800 dark:bg-neutral-900/40 dark:text-neutral-400"
			>
				{#each report.items.slice(0, 8) as item (item.path)}
					<div class="truncate">{item.path}</div>
				{/each}
				{#if report.items.length > 8}
					<div class="mt-1 text-neutral-500">+{report.items.length - 8}</div>
				{/if}
			</div>
			{#if execError}
				<div
					class="mt-3 rounded border border-red-300 bg-red-50 px-3 py-2 text-xs text-red-700
						dark:border-red-900/60 dark:bg-red-950/40 dark:text-red-300"
				>
					{execError}
				</div>
			{/if}
			<div class="mt-5 flex justify-end gap-2">
				<button
					type="button"
					class="rounded-md border border-neutral-300 px-4 py-2 text-sm transition
						hover:bg-neutral-100 disabled:opacity-50
						dark:border-neutral-700 dark:hover:bg-neutral-800"
					disabled={executing}
					onclick={() => (confirmOpen = false)}
				>
					{$t('cancel')}
				</button>
				<button
					type="button"
					class="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white transition
						hover:bg-red-500 disabled:cursor-not-allowed disabled:bg-neutral-300
						dark:disabled:bg-neutral-700"
					disabled={executing}
					onclick={runExecute}
				>
					{executing ? $t('deleting') : $t('delete_n', { n: report.items.length })}
				</button>
			</div>
		</div>
	</div>
{/if}
