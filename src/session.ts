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
				setView(await invoke<ClientView>(command, args));
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
		let unlisten: (() => void) | undefined;
		listen<ClientView>("session", ({ payload }) => {
			if (active) setView(payload);
		})
			.then((stop) => {
				if (active) {
					unlisten = stop;
					void run("session");
				} else {
					stop();
				}
			})
			.catch((cause) => {
				if (active) setError(String(cause));
			});
		return () => {
			active = false;
			unlisten?.();
		};
	}, [run]);

	const act = useCallback(
		(action: RoomAction) => run("room_action", { action }),
		[run],
	);
	return { view, error, busy, run, act, clearError: () => setError(null) };
}

export type Session = ReturnType<typeof useSession>;
