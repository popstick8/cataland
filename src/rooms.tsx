import {
	ArrowRight,
	Clock3,
	DoorOpen,
	Radio,
	RotateCcw,
	WifiOff,
} from "lucide-react";
import { useState } from "react";
import type { ClientView, RoomInfo } from "./bindings";
import type { Session } from "./session";
import "./rooms.css";

export function RoomBrowser({
	view,
	session,
	name,
	color,
}: {
	view: ClientView;
	session: Session;
	name: string;
	color: number;
}) {
	const [address, setAddress] = useState("");
	const connect = (room: RoomInfo) => {
		const current = view.nearby.find((item) => item.id === room.id) ?? room;
		void session.run("join", { addresses: current.addresses, name, color });
	};
	return (
		<section className="room-browser">
			<div className="paper room-list">
				<h2>
					<Radio size={21} /> 加入房间
				</h2>
				{view.nearby.length === 0 && (
					<p className="muted">
						同一局域网中的房间会显示在这里，也可以输入主机地址连接。
					</p>
				)}
				{view.nearby.map((room) => (
					<button
						type="button"
						className="room-entry"
						key={room.id}
						disabled={session.busy}
						onClick={() => connect(room)}
					>
						<span className="room-entry-text">
							<strong>{room.name}</strong>
							<small>
								{room.mode === "cities" ? "城市与骑士" : "基础规则"} ·{" "}
								{room.players} / {room.capacity} 人
								{room.started ? " · 观战" : ""}
							</small>
						</span>
						<ArrowRight size={18} />
					</button>
				))}
				<form
					className="address-form"
					onSubmit={(event) => {
						event.preventDefault();
						void session.run("join", { addresses: [address], name, color });
					}}
				>
					<label>
						主机地址
						<input
							value={address}
							onChange={(event) => setAddress(event.target.value)}
							placeholder="192.168.1.10:端口"
							required
						/>
					</label>
					<button type="submit" disabled={session.busy || !address.trim()}>
						<DoorOpen size={17} /> 连接
					</button>
				</form>
				{view.recent.length > 0 && (
					<details>
						<summary>最近加入</summary>
						{view.recent.map((room) => (
							<button
								type="button"
								className="room-entry"
								key={room.id}
								disabled={session.busy}
								onClick={() => connect(room)}
							>
								<span className="room-entry-text">
									<strong>{room.name}</strong>
									<small>{room.addresses[0]}</small>
								</span>
								<ArrowRight size={18} />
							</button>
						))}
					</details>
				)}
			</div>
			<div className="paper room-list">
				<h2>
					<Clock3 size={21} /> 继续游戏
				</h2>
				{view.saves.length === 0 && (
					<p className="muted">
						本机创建的对局会自动保存，可以从这里重新开放房间，继续上一次的游戏。
					</p>
				)}
				{view.saves.map((save) => (
					<button
						type="button"
						className="room-entry"
						key={save.id}
						disabled={session.busy}
						onClick={() => void session.run("resume", { id: save.id })}
					>
						<span className="room-entry-text">
							<strong>{save.name}</strong>
							<small>{save.players.join("、")}</small>
							<small>{new Date(save.updated).toLocaleString()}</small>
						</span>
						<RotateCcw size={18} />
					</button>
				))}
			</div>
		</section>
	);
}

export function ConnectionNotice({
	view,
	session,
}: {
	view: ClientView;
	session: Session;
}) {
	if (view.connection === "home" || view.connection === "connected")
		return null;
	const connecting = view.connection === "connecting";
	return (
		<div className="connection-overlay">
			<section className="paper connection-panel" role="status">
				{connecting ? <Radio size={30} /> : <WifiOff size={30} />}
				<h2>{connecting ? "正在连接房间" : "房间连接已断开"}</h2>
				<p className="muted">
					{connecting
						? "正在与主机建立连接。"
						: view.room
							? "席位与对局仍在原房间。房主恢复游戏后即可继续。"
							: "可以重新连接，或返回选择其他房间。"}
				</p>
				<div className="form-row">
					<button type="button" onClick={() => void session.run("leave")}>
						返回首页
					</button>
					{!connecting && (
						<button
							type="button"
							className="primary"
							disabled={session.busy}
							onClick={() =>
								void session.run("join", {
									addresses: view.addresses,
									name: view.identity.name,
									color: view.identity.color,
								})
							}
						>
							重新连接
						</button>
					)}
				</div>
			</section>
		</div>
	);
}
