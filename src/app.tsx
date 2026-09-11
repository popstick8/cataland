import { Compass, X } from "lucide-react";
import { GameTable } from "./game";
import { Home, Lobby } from "./lobby";
import { ConnectionNotice } from "./rooms";
import { useSession } from "./session";
import "./style.css";

export function App() {
	const session = useSession();
	return (
		<div className="app">
			<header className="app-bar">
				<div className="brand">
					<Compass size={24} />
					<span>Cataland</span>
				</div>
				<span className="app-caption">群岛之约</span>
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
