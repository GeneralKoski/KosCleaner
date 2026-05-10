export type ScanItem = {
	path: string;
	size: number;
	is_symlink: boolean;
};

export type ScanError = {
	path: string;
	message: string;
};

export type ScanReport = {
	cleaner_id: string;
	category: string;
	name: string;
	items: ScanItem[];
	total_size: number;
	errors: ScanError[];
};

export type CleanerInfo = {
	id: string;
	category: string;
	name: string;
	description: string;
};

export type CategoryGroup = {
	id: string;
	label: string;
	enabled: boolean;
	cleaners: CleanerInfo[];
};
