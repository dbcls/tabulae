export type Column = {
	column_name: string;
	column_type: string;
	min: string | null;
	max: string | null;
	approx_unique: number;
	avg: number | null;
	std: number | null;
	q25: number | null;
	q50: number | null;
	q75: number | null;
	count: number;
	null_percentage: number;
	exact_unique: number;
};

export type Table = {
	name: string;
	columns: Column[];
	num_rows: number;
	collection: string;
	query: string | null;
	sizes: Record<string, number>;
};

export type Manifest = {
	tables: Table[];
};
