import { useRef } from "react";
import { DownloadLink } from "./DownloadLink";

export function DuckdbDialog({
	open,
	onClose,
	baseUrl,
}: { open: boolean; onClose: () => void; baseUrl: string }) {
	const dialogRef = useRef<HTMLDialogElement>(null);
	const layer1Url = `${baseUrl}layer1.duckdb`;
	const layer2Url = `${baseUrl}layer2.duckdb`;

	return (
		<dialog ref={dialogRef} className="modal" open={open}>
			<div className="modal-box w-11/12 max-w-[1536px] max-h-11/12 overflow-hidden">
				<h3 className="font-bold text-lg">DuckDB Databases</h3>
				<div className="mt-4 flex flex-col gap-4">
					<DownloadLink url={layer1Url} label="Layer 1" filesize={undefined} />
					<DownloadLink url={layer2Url} label="Layer 2" filesize={undefined} />
				</div>
			</div>
			<form method="dialog" className="modal-backdrop">
				<button type="button" onClick={onClose}>
					close
				</button>
			</form>
		</dialog>
	);
}
