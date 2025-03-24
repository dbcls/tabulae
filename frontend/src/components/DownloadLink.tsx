import { HardDriveDownload, Link } from "lucide-react";

const humanizeSize = (size: number): string => {
	const units = ["B", "KB", "MB", "GB", "TB"];
	let s = size;
	let unitIndex = 0;
	while (s >= 1024) {
		s /= 1024;
		unitIndex++;
	}
	return `${s.toFixed(2)} ${units[unitIndex]}`;
};

export function DownloadLink({
	url,
	label,
	filesize,
}: {
	label: string;
	url: string;
	filesize: number | undefined;
}) {
	return (
		<div className="join">
			<div className="join-item flex items-center justify-center mr-2 font-bold text-base-content/60">
				{label}
			</div>
			<a href={url} download>
				<button
					type="button"
					className={`btn btn-sm join-item ${filesize ? "tooltip tooltip-top" : ""}`}
					{...(filesize && { "data-tip": humanizeSize(filesize) })}
				>
					<HardDriveDownload size={16} />
				</button>
			</a>
			<button
				type="button"
				className="btn btn-sm join-item tooltip tooltip-top"
				data-tip="Copy link"
				onClick={() => navigator.clipboard.writeText(url)}
			>
				<Link size={16} />
			</button>
		</div>
	);
}
