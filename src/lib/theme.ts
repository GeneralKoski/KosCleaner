import { writable } from 'svelte/store';

export type Theme = 'light' | 'dark';

const STORAGE_KEY = 'koscleaner.theme';

function initial(): Theme {
	if (typeof localStorage !== 'undefined') {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved === 'light' || saved === 'dark') return saved;
	}
	if (typeof window !== 'undefined' && window.matchMedia) {
		return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
	}
	return 'dark';
}

export const theme = writable<Theme>(initial());

export function applyTheme(value: Theme) {
	if (typeof document === 'undefined') return;
	document.documentElement.classList.toggle('dark', value === 'dark');
	try {
		localStorage.setItem(STORAGE_KEY, value);
	} catch {
		/* ignore */
	}
}

if (typeof window !== 'undefined') {
	theme.subscribe(applyTheme);
}
