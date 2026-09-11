import {
	ArrowRight,
	BookOpen,
	Clock3,
	DoorOpen,
	Radio,
	RotateCcw,
	WifiOff,
} from "lucide-react";
import { useState } from "react";
import type { ClientView, RoomInfo } from "./bindings";
import { useText } from "./locale";
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
	const t = useText();
	const [address, setAddress] = useState("");
	const connect = (room: RoomInfo) => {
		const current = view.nearby.find((item) => item.id === room.id) ?? room;
		void session.run("join", { addresses: current.addresses, name, color });
	};
	return (
		<section className="room-browser">
			<div className="paper room-list">
				<h2>
					<Radio size={21} />
					{t("加入房间")}
				</h2>
				{view.nearby.length === 0 && (
					<p className="muted">
						{t("同一局域网中的房间会显示在这里，也可以输入主机地址连接。")}
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
								{room.mode === "cities" ? t("城市与骑士") : t("基础规则")} ·{" "}
								{t("{0} / {1} 人", room.players, room.capacity)}
								{room.started ? t(" · 观战") : ""}
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
						{t("主机地址")}
						<input
							value={address}
							onChange={(event) => setAddress(event.target.value)}
							placeholder={t("192.168.1.10:端口")}
							required
						/>
					</label>
					<button type="submit" disabled={session.busy || !address.trim()}>
						<DoorOpen size={17} />
						{t("连接")}
					</button>
				</form>
				{view.recent.length > 0 && (
					<details>
						<summary>{t("最近加入")}</summary>
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
					<Clock3 size={21} />
					{t("继续游戏")}
				</h2>
				{view.saves.length === 0 && (
					<p className="muted">
						{t(
							"本机创建的对局会自动保存，可以从这里重新开放房间，继续上一次的游戏。",
						)}
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
							{save.finished && <small>{t("已结束 · 查看对局")}</small>}
							<small>{save.players.join(t("、"))}</small>
							<small>
								{new Date(save.updated).toLocaleString(
									view.preferences.language,
								)}
							</small>
						</span>
						{save.finished ? <BookOpen size={18} /> : <RotateCcw size={18} />}
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
	const t = useText();
	if (["home", "connected", "review"].includes(view.connection)) return null;
	const connecting = view.connection === "connecting";
	return (
		<div className="connection-overlay">
			<section className="paper connection-panel" role="status">
				{connecting ? <Radio size={30} /> : <WifiOff size={30} />}
				<h2>{connecting ? t("正在连接房间") : t("房间连接已断开")}</h2>
				<p className="muted">
					{connecting
						? t("正在与主机建立连接。")
						: view.room
							? t("席位与对局仍在原房间。房主恢复游戏后即可继续。")
							: t("可以重新连接，或返回选择其他房间。")}
				</p>
				<div className="form-row">
					<button type="button" onClick={() => void session.run("leave")}>
						{t("返回首页")}
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
							{t("重新连接")}
						</button>
					)}
				</div>
			</section>
		</div>
	);
}
