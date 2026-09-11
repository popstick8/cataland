import { getCurrentWindow } from "@tauri-apps/api/window";
import { Compass, Settings, X } from "lucide-react";
import { useEffect, useState } from "react";
import { GameTable } from "./game";
import { Home, Lobby } from "./lobby";
import { PreferencesPanel } from "./preferences";
import { ConnectionNotice } from "./rooms";
import { useSession } from "./session";
import { useSound } from "./sound";
import "./style.css";

export function App() {
	const session = useSession();
	const [settings, setSettings] = useState(false);
	const { report } = session;
	useSound(session.view?.preferences, session.view?.room, report);
	useEffect(() => {
		const key = (event: KeyboardEvent) => {
			if (event.key !== "F11") return;
			event.preventDefault();
			const window = getCurrentWindow();
			void window
				.isFullscreen()
				.then((full) => window.setFullscreen(!full))
				.catch(report);
		};
		window.addEventListener("keydown", key);
		return () => window.removeEventListener("keydown", key);
	}, [report]);
	useEffect(() => {
		if (session.view)
			document.documentElement.dataset.animation = String(
				session.view.preferences.animation,
			);
	}, [session.view]);
	return (
		<div className="app">
			{settings && session.view && (
				<PreferencesPanel
					initial={session.view.preferences}
					close={() => setSettings(false)}
				/>
			)}
			<header className="app-bar">
				<div className="brand">
					<Compass size={24} />
					<span>Cataland</span>
				</div>
				<div className="app-tools">
					<span className="app-caption">群岛之约</span>
					<button
						type="button"
						className="icon-button"
						aria-label="设置"
						disabled={!session.view}
						onClick={() => setSettings(true)}
					>
						<Settings size={20} />
					</button>
				</div>
			</header>
			{session.view && (
				<ConnectionNotice view={session.view} session={session} />
			)}
			{session.error && (
				<div className="error" role="alert">
					<span>{session.error}</span>
					<button
						type="button"
						className="icon-button"
						aria-label="关闭消息"
						onClick={session.clearError}
					>
						<X size={18} />
					</button>
				</div>
			)}
			{session.view ? (
				session.view.room ? (
					session.view.room.game ? (
						<GameTable
							key={session.view.room.id}
							room={session.view.room}
							game={session.view.room.game}
							session={session}
						/>
					) : (
						<Lobby
							key={session.view.room.id}
							room={session.view.room}
							session={session}
						/>
					)
				) : (
					<Home view={session.view} session={session} />
				)
			) : (
				<main className="loading">
					<Compass size={38} />
					<p>正在打开群岛…</p>
					{session.error && (
						<button type="button" onClick={() => void session.run("session")}>
							重新连接
						</button>
					)}
				</main>
			)}
		</div>
	);
}
