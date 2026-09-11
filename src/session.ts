import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import type { ClientView, RoomAction } from "./bindings";

export function useSession() {
	const [view, setView] = useState<ClientView | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [busy, setBusy] = useState(false);

	const run = useCallback(
		async (command: string, args?: Record<string, unknown>) => {
			setBusy(true);
			setError(null);
			try {
				if (command === "session") setView(await invoke<ClientView>(command));
				else await invoke(command, args);
			} catch (cause) {
				setError(String(cause));
			} finally {
				setBusy(false);
			}
		},
		[],
	);

	useEffect(() => {
		let active = true;
		let received = false;
		let unlisten: (() => void)[] = [];
		Promise.all([
			listen<ClientView>("session", ({ payload }) => {
				received = true;
				if (active) setView(payload);
			}),
			listen<string>("notice", ({ payload }) => {
				if (active) setError(payload);
			}),
		])
			.then(async (stops) => {
				if (!active) {
					for (const stop of stops) stop();
					return;
				}
				unlisten = stops;
				const initial = await invoke<ClientView>("session");
				if (active && !received) setView(initial);
			})
			.catch((cause) => {
				if (active) setError(String(cause));
			});
		return () => {
			active = false;
			for (const stop of unlisten) stop();
		};
	}, []);

	const act = useCallback(
		(action: RoomAction) => run("room_action", { action }),
		[run],
	);
	return { view, error, busy, run, act, clearError: () => setError(null) };
}

export type Session = ReturnType<typeof useSession>;
