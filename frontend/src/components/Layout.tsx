import { Database } from "lucide-react";
import type { ReactNode } from "react";
import { useState } from "react";
import { DuckdbDialog } from "./DuckdbDialog";

export function Layout({
	children,
	baseUrl,
}: { children: ReactNode; baseUrl: string }) {
	const [duckdbDialogOpen, setDuckDBDialogOpen] = useState(false);
	return (
		<>
			<div className="navbar bg-primary text-primary-content flex justify-between">
				<button type="button" className="btn btn-ghost text-xl">
					tabulae
				</button>

				<button
					type="button"
					className="btn btn-circle btn-ghost btn-sm"
					onClick={() => setDuckDBDialogOpen(true)}
				>
					<Database size={16} />
				</button>
			</div>
			<DuckdbDialog
				open={duckdbDialogOpen}
				onClose={() => {
					setDuckDBDialogOpen(false);
				}}
				baseUrl={baseUrl}
			/>

			<main className="container mx-auto py-5">{children}</main>
		</>
	);
}
