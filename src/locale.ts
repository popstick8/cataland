import { createContext, useContext, useMemo } from "react";
import type { Language, Text } from "./bindings";
import english from "./en.json";

export const Locale = createContext<Language>("zh-CN");

export function translator(language: Language) {
	const catalogue: Record<string, string> = english;
	const list = new Intl.ListFormat(language, { style: "short" });
	const format = (key: string, values: (string | number)[]) => {
		const template = language === "en" ? (catalogue[key] ?? key) : key;
		return template.replace(
			/\{(\d+)\}/g,
			(match, index: string) => values[Number(index)]?.toString() ?? match,
		);
	};
	const render = (value: Text): string => {
		if (typeof value === "string") return value;
		if ("items" in value) return list.format(value.items.map(render));
		return format(value.key, value.args.map(render));
	};
	return (key: Text, ...values: (string | number)[]) =>
		typeof key === "string" ? format(key, values) : render(key);
}

export type Translate = ReturnType<typeof translator>;

export function errorText(cause: unknown): Text {
	if (cause && typeof cause === "object" && "key" in cause && "args" in cause)
		return cause as Text;
	return cause instanceof Error ? cause.message : String(cause);
}

export function useText() {
	const language = useContext(Locale);
	return useMemo(() => translator(language), [language]);
}
