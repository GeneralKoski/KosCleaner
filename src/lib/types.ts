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

export type ExecutionFailure = {
	path: string;
	message: string;
};

export type ExecutionReport = {
	cleaner_id: string;
	attempted: number;
	deleted: number;
	freed_bytes: number;
	failures: ExecutionFailure[];
	audit_log_path: string | null;
};
