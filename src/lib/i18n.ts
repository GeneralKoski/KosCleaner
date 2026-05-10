import { derived, writable } from 'svelte/store';

export type Locale = 'en' | 'it';

const dict = {
	en: {
		alpha: 'alpha',
		category_system: 'System',
		category_browsers: 'Browsers',
		category_packages: 'Packages',
		category_trash: 'Trash',
		soon: 'soon',
		scan: 'Scan',
		scanning: 'Scanning…',
		clean: 'Clean…',
		empty_intro: 'Click {scan} to see what can be recovered.',
		nothing_to_clean: 'Nothing to clean.',
		recoverable: 'Recoverable',
		items_count: '{n} item|items',
		skipped: '{n} skipped',
		scan_error: 'Scan failed',
		coming_soon: 'Coming soon.',
		no_cleaners: 'No cleaners available in this category yet.',
		cleaners_available: '{n} cleaner|cleaners available',
		path: 'Path',
		size: 'Size',
		more_items: '+{n} more items not shown',
		entries_skipped: '{n} entries skipped',
		confirm_title: 'Confirm deletion',
		confirm_body:
			'This will permanently delete {n} file|files, freeing {size}. This action cannot be undone.',
		cancel: 'Cancel',
		delete_n: 'Delete {n} item|items',
		deleting: 'Deleting…',
		done: 'Done',
		freed: '{size} freed',
		deleted_summary: '{deleted} of {attempted} item|items deleted',
		failed: '{n} failed',
		audit_log: 'Audit log: {path}',
		show_failures: 'Show {n} failure|failures',
		theme_dark: 'Dark',
		theme_light: 'Light',
		language: 'Language'
	},
	it: {
		alpha: 'alpha',
		category_system: 'Sistema',
		category_browsers: 'Browser',
		category_packages: 'Pacchetti',
		category_trash: 'Cestino',
		soon: 'presto',
		scan: 'Scansione',
		scanning: 'Scansione in corso…',
		clean: 'Pulisci…',
		empty_intro: 'Clicca {scan} per vedere cosa è recuperabile.',
		nothing_to_clean: 'Niente da pulire.',
		recoverable: 'Recuperabili',
		items_count: '{n} elemento|elementi',
		skipped: '{n} saltati',
		scan_error: 'Scansione fallita',
		coming_soon: 'Presto disponibile.',
		no_cleaners: 'Nessun cleaner disponibile in questa categoria.',
		cleaners_available: '{n} cleaner disponibile|cleaner disponibili',
		path: 'Percorso',
		size: 'Dimensione',
		more_items: '+{n} elementi non mostrati',
		entries_skipped: '{n} elementi saltati',
		confirm_title: 'Conferma cancellazione',
		confirm_body:
			'Verranno eliminati definitivamente {n} file|file, liberando {size}. L’operazione è irreversibile.',
		cancel: 'Annulla',
		delete_n: 'Elimina {n} elemento|elementi',
		deleting: 'Eliminazione…',
		done: 'Completato',
		freed: '{size} liberati',
		deleted_summary: '{deleted} di {attempted} elemento eliminato|elementi eliminati',
		failed: '{n} falliti',
		audit_log: 'Log di audit: {path}',
		show_failures: 'Mostra {n} fallimento|fallimenti',
		theme_dark: 'Scuro',
		theme_light: 'Chiaro',
		language: 'Lingua'
	}
} as const satisfies Record<Locale, Record<string, string>>;

type DictKey = keyof (typeof dict)['en'];

const STORAGE_KEY = 'koscleaner.locale';

function detectLocale(): Locale {
	if (typeof localStorage !== 'undefined') {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved === 'en' || saved === 'it') return saved;
	}
	if (typeof navigator !== 'undefined' && navigator.language?.toLowerCase().startsWith('it')) {
		return 'it';
	}
	return 'en';
}

export const locale = writable<Locale>(detectLocale());

if (typeof window !== 'undefined') {
	locale.subscribe((l) => {
		try {
			localStorage.setItem(STORAGE_KEY, l);
		} catch {
			/* localStorage may be unavailable; ignore */
		}
	});
}

// Sostituisce {var} e gestisce la pluralizzazione "singular|plural" in base a {n}.
function format(template: string, vars: Record<string, string | number>): string {
	let s = template;
	const n = typeof vars.n === 'number' ? vars.n : undefined;
	if (s.includes('|') && n !== undefined) {
		// Forma "singular|plural": prendi il ramo giusto.
		s = s
			.split(' ')
			.map((word) => {
				if (!word.includes('|')) return word;
				const [sing, plur] = word.split('|');
				return n === 1 ? sing : plur;
			})
			.join(' ');
	}
	for (const [k, v] of Object.entries(vars)) {
		s = s.replaceAll(`{${k}}`, String(v));
	}
	return s;
}

export const t = derived(locale, ($l) => {
	return (key: DictKey, vars: Record<string, string | number> = {}) => {
		const template = dict[$l][key] ?? dict.en[key] ?? key;
		return format(template, vars);
	};
});
