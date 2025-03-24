import { useEffect, useState } from "react";

import { Layout } from "./components/Layout";
import { TableCard } from "./components/TableCard";
import type { Manifest } from "../lib/types";
import { CircleX } from "lucide-react";

function addTrailSlash(url: string) {
	return url.endsWith("/") ? url : `${url}/`;
}

function App() {
	const [manifest, setManifest] = useState<Manifest | null>(null);
	const [showL1, setShowL1] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const base = import.meta.env.VITE_MANIFEST_URL
		? addTrailSlash(import.meta.env.VITE_MANIFEST_URL)
		: "";
	const [baseUrl, setBaseUrl] = useState<string>(base);

	const manifestUrl = `${baseUrl}manifest.json`;

	useEffect(() => {
		if (baseUrl === "") {
			setBaseUrl(location.href.replace("index.html", ""));
		}
	}, [baseUrl]);

	useEffect(() => {
		fetch(manifestUrl)
			.then((res) => res.json())
			.then((data) => setManifest(data))
			.catch((err) => setError(err.message));
	}, [manifestUrl]);

	if (!manifest) {
		if (error) {
			return (
				<Layout baseUrl={baseUrl}>
					<div role="alert" className="alert alert-error">
						<CircleX />
						<span>
							<strong>Error!</strong> {error}, manifest.json should be at{" "}
							{manifestUrl}
						</span>
					</div>
				</Layout>
			);
		}
		return (
			<Layout baseUrl={baseUrl}>
				<div className="flex justify-center items-center h-full">
					<span className="loading loading-spinner loading-xs" />
				</div>
			</Layout>
		);
	}

	const tables = showL1
		? manifest.tables
		: manifest.tables.filter((table) => table.collection !== "layer1");

	return (
		<Layout baseUrl={baseUrl}>
			<div className="field-set w-48 mb-5">
				<label className="fieldset-label">
					<input
						type="checkbox"
						checked={showL1}
						className="toggle"
						onChange={(e) => setShowL1(e.currentTarget.checked)}
					/>
					Show Layer 1 tables
				</label>
			</div>

			{tables.map((table) => (
				<TableCard
					key={`${table.collection}.${table.name}`}
					table={table}
					baseUrl={baseUrl}
				/>
			))}
		</Layout>
	);
}

export default App;
