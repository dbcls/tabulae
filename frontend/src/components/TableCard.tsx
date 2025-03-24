import {
	Columns3,
	Copy,
	Rows3,
	Search,
	Table as TableIcon,
} from "lucide-react";

import type { Table } from "../../lib/types";
import { useRef } from "react";
import { ShikiCode } from "./ShikiCode";
import { DownloadLink } from "./DownloadLink";

function QueryModalButton({ table }: { table: Table }) {
	const dialogRef = useRef<HTMLDialogElement>(null);

	const language = table.collection === "layer1" ? "SPARQL" : "SQL";
	const shikiLang = table.collection === "layer1" ? "sparql" : "sql";

	return (
		<>
			<button
				type="button"
				className="btn btn-sm"
				onClick={() => dialogRef.current?.showModal()}
			>
				<Search size={16} />
				{language}
			</button>
			<dialog ref={dialogRef} className="modal">
				<div className="modal-box w-11/12 max-w-[1536px] max-h-11/12 overflow-hidden flex flex-col">
					<h3 className="font-bold text-lg mb-3 flex gap-1 items-center">
						<div className="text-primary">
							<TableIcon />
						</div>
						<div>{table.name}</div>
					</h3>

					<div className="overflow-auto relative">
						<button
							type="button"
							className="btn btn-xs btn-ghost absolute right-2 top-2 tooltip tooltip-left"
							data-tip="Copy"
							onClick={() => navigator.clipboard.writeText(table.query || "")}
						>
							<Copy size={14} />
						</button>
						<ShikiCode
							code={table.query || ""}
							lang={shikiLang}
							theme="github-light"
						/>
					</div>
				</div>
				<form method="dialog" className="modal-backdrop">
					<button type="button" onClick={() => dialogRef.current?.close()}>
						close
					</button>
				</form>
			</dialog>
		</>
	);
}

function downloadUrl(baseUrl: string, table: Table, ext: string) {
	const collection = table.collection === "layer1" ? "layer1" : "layer2";

	return `${baseUrl}${collection}/${table.name}.${ext}`;
}

export function TableCard({
	table,
	baseUrl,
}: { table: Table; baseUrl: string }) {
	return (
		<div className="card bg-base-100 my-3">
			<div className="card-body">
				<h2 className="card-title">
					<span className="text-primary">
						<TableIcon />
					</span>
					<span className="text-2xl font-extrabold">{table.name}</span>
					<button
						type="button"
						className="btn btn-xs btn-ghost tooltip tooltip-top"
						data-tip="Copy table name"
						onClick={() => navigator.clipboard.writeText(table.name)}
					>
						<Copy size={14} />
					</button>
					{table.collection === "layer1" ? (
						<div className="badge badge-soft badge-secondary">Layer 1</div>
					) : table.collection === "layer2" ? (
						<div className="badge badge-soft badge-secondary">Layer 2</div>
					) : null}
				</h2>

				<div className="stats shadow max-w-192 flex">
					<div className="stat">
						<div className="stat-figure text-secondary">
							<Rows3 />
						</div>
						<div className="stat-title">Rows</div>
						<div className="stat-value">{table.num_rows.toLocaleString()}</div>
					</div>

					<div className="stat">
						<div className="stat-figure text-secondary">
							<Columns3 />
						</div>
						<div className="stat-title">Columns</div>
						<div className="stat-value">
							{table.columns.length.toLocaleString()}
						</div>
					</div>
				</div>

				<div className="overflow-x-auto">
					<table className="table table-sm">
						<thead>
							<tr>
								<th className="z-40" />
								<th>Type</th>
								<th>Min</th>
								<th>Max</th>
								<th># Unique</th>
								<th>Avg</th>
								<th>Std</th>
								<th>Q25</th>
								<th>Q50</th>
								<th>Q75</th>
								<th>Null %</th>
							</tr>
						</thead>
						<tbody>
							{table.columns.map((column) => (
								<tr key={column.column_name}>
									<th>{column.column_name}</th>
									<td>{column.column_type}</td>
									<td>{column.min}</td>
									<td>{column.max}</td>
									<td className="text-right">
										{column.exact_unique.toLocaleString()}
									</td>
									<td className="text-right">{column.avg}</td>
									<td className="text-right">{column.std}</td>
									<td className="text-right">{column.q25}</td>
									<td className="text-right">{column.q50}</td>
									<td className="text-right">{column.q75}</td>
									<td className="text-right">
										{column.null_percentage.toLocaleString()}
									</td>
								</tr>
							))}
						</tbody>
					</table>
				</div>

				<div className="card-actions flex justify-between items-center">
					<div>{table.query && <QueryModalButton table={table} />}</div>
					<div className="flex gap-5">
						<DownloadLink
							url={downloadUrl(baseUrl, table, "csv")}
							filesize={table.sizes.csv}
							label="CSV"
						/>
						<DownloadLink
							url={downloadUrl(baseUrl, table, "tsv")}
							filesize={table.sizes.tsv}
							label="TSV"
						/>
						<DownloadLink
							url={downloadUrl(baseUrl, table, "parquet")}
							filesize={table.sizes.parquet}
							label="Parquet"
						/>
					</div>
				</div>
			</div>
		</div>
	);
}
