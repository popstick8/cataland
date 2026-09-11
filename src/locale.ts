import { createContext, useContext, useMemo } from "react";
import type { Language } from "./bindings";
import english from "./en.json";

export const Locale = createContext<Language>("zh-CN");

export function translator(language: Language) {
	const catalogue: Record<string, string> = english;
	return (key: string, ...values: (string | number)[]) => {
		const template = language === "en" ? (catalogue[key] ?? key) : key;
		return template.replace(
			/\{(\d+)\}/g,
			(match, index: string) => values[Number(index)]?.toString() ?? match,
		);
	};
}

export function useText() {
	const language = useContext(Locale);
	return useMemo(() => translator(language), [language]);
}
