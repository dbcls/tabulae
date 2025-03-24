import githubLight from "@shikijs/themes/github-light";
import { useEffect, useState } from "react";
import type { HighlighterCore } from "shiki";

import { createHighlighterCore } from "shiki/core";
import { createOnigurumaEngine } from "shiki/engine/oniguruma";

type Props = {
	code: string;
	lang: string;
	theme: string;
};

let highlighterInstance: HighlighterCore | null = null;

export const useShikiHighlighter = () => {
	const [highlighter, setHighlighter] = useState<HighlighterCore | null>(
		highlighterInstance,
	);

	useEffect(() => {
		const initializeHighlighter = async () => {
			if (!highlighterInstance) {
				highlighterInstance = await createHighlighterCore({
					themes: [githubLight],
					langs: [
						import("@shikijs/langs/sparql"),
						import("@shikijs/langs/sql"),
					],
					engine: createOnigurumaEngine(import("shiki/wasm")),
				});
			}
			setHighlighter(highlighterInstance);
		};

		initializeHighlighter();
	}, []);

	return highlighter;
};

export function ShikiCode({ code, lang, theme }: Props) {
	const highlighter = useShikiHighlighter();
	const [html, setHtml] = useState<string | undefined>();

	useEffect(() => {
		(async () => {
			const out = highlighter?.codeToHtml(code, {
				lang,
				theme,
				colorReplacements: {
					"#fff": "var(--color-base-200)",
				},
			});
			setHtml(out);
		})();
	}, [code, lang, theme, highlighter]);

	if (!html) {
		return (
			<pre>
				<code>{code}</code>
			</pre>
		);
	}

	return (
		// biome-ignore lint/security/noDangerouslySetInnerHtml: html is written by shiki
		<div dangerouslySetInnerHTML={{ __html: html }} />
	);
}
