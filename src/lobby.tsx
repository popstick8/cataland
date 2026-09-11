import {
	ArrowLeft,
	Check,
	Compass,
	Crown,
	Send,
	Users,
	Waves,
} from "lucide-react";
import { useState } from "react";
import island from "../src-tauri/icons/icon.png";
import type { ClientView, RoomSettings, RoomView } from "./bindings";
import { useText } from "./locale";
import { colorNames, colors } from "./palette";
import { RoomBrowser } from "./rooms";
import type { Session } from "./session";

function ColorPicker({
	value,
	occupied = [],
	onChange,
}: {
	value: number;
	occupied?: number[];
	onChange: (color: number) => void;
}) {
	const t = useText();
	return (
		<fieldset className="colors">
			<legend>{t("玩家颜色")}</legend>
			{colors.map((color, index) => (
				<button
					key={color}
					type="button"
					className="color"
					style={{ background: color }}
					aria-label={t(colorNames[index] ?? "")}
					aria-pressed={value === index}
					disabled={occupied.includes(index)}
					onClick={() => onChange(index)}
				>
					{value === index && <Check size={18} />}
				</button>
			))}
		</fieldset>
	);
}

export function Home({
	view,
	session,
}: {
	view: ClientView;
	session: Session;
}) {
	const t = useText();
	const [name, setName] = useState(view.identity.name);
	const [color, setColor] = useState(view.identity.color);
	const [settings, setSettings] = useState<RoomSettings>({
		name: t("群岛之约"),
		mode: "base",
		capacity: 4,
		starter: null,
	});
	return (
		<main className="home">
			<section className="hero">
				<div className="eyebrow">
					<Compass size={16} />
					{t("一片海，一场新故事")}
				</div>
				<h1>
					Cataland<span className="hero-subtitle">{t("群岛之约")}</span>
				</h1>
				<p className="hero-description">
					{t(
						"从第一座村庄开始，沿着海岸铺设道路，在收获与交换之间，建起属于这场相遇的群岛。",
					)}
				</p>
				<div className="hero-facts">
					<span>
						<Users size={17} />
						{t("2–6 名玩家")}
					</span>
					<span>
						<Waves size={18} />
						{t("局域网同游")}
					</span>
				</div>
				<img
					className="hero-island"
					src={island}
					alt={t("森林、麦田与海湾环绕的群岛聚落")}
				/>
			</section>
			<form
				className="paper create-room"
				onSubmit={(event) => {
					event.preventDefault();
					void session.run("host", { settings, name, color });
				}}
			>
				<div className="eyebrow">{t("新的航程")}</div>
				<h2>{t("创建房间")}</h2>
				<label>
					{t("玩家名称")}
					<input
						value={name}
						maxLength={24}
						required
						onChange={(event) => setName(event.target.value)}
						autoComplete="nickname"
					/>
				</label>
				<ColorPicker value={color} onChange={setColor} />
				<label>
					{t("房间名称")}
					<input
						value={settings.name}
						maxLength={48}
						required
						onChange={(event) =>
							setSettings({ ...settings, name: event.target.value })
						}
					/>
				</label>
				<div className="form-row">
					<label>
						{t("玩法")}
						<select
							value={settings.mode}
							onChange={(event) =>
								setSettings({
									...settings,
									mode: event.target.value === "cities" ? "cities" : "base",
								})
							}
						>
							<option value="base">{t("基础规则")}</option>
							<option value="cities">{t("城市与骑士")}</option>
						</select>
					</label>
					<label>
						{t("人数")}
						<select
							value={settings.capacity}
							onChange={(event) =>
								setSettings({
									...settings,
									capacity: Number(event.target.value),
								})
							}
						>
							{[2, 3, 4, 5, 6].map((count) => (
								<option key={count} value={count}>
									{t("{0} 人", count)}
								</option>
							))}
						</select>
					</label>
				</div>
				<p className="muted">{t("同一局域网中的朋友可以加入房间。")}</p>
				<button type="submit" className="primary wide" disabled={session.busy}>
					{t("创建房间")}
					<Compass size={18} />
				</button>
			</form>
			<RoomBrowser view={view} session={session} name={name} color={color} />
		</main>
	);
}

export function ChatPanel({
	room,
	session,
}: {
	room: RoomView;
	session: Session;
}) {
	const t = useText();
	const [text, setText] = useState("");
	return (
		<section className="chat paper">
			<h3>{t("围桌闲聊")}</h3>
			<div className="chat-messages" role="log" aria-label={t("房间聊天")}>
				{room.chat.length === 0 && (
					<p className="muted">{t("海风吹过，朋友正在赶来。")}</p>
				)}
				{room.chat.map((message) => (
					<p key={`${message.time}-${message.name}`}>
						<strong>{message.name}</strong>
						<span>{message.text}</span>
					</p>
				))}
			</div>
			{session.view?.connection === "connected" && (
				<form
					className="chat-compose"
					onSubmit={(event) => {
						event.preventDefault();
						if (text.trim()) {
							void session.act({ type: "chat", text });
							setText("");
						}
					}}
				>
					<input
						value={text}
						onChange={(event) => setText(event.target.value)}
						placeholder={t("聊聊下一步的打算…")}
						aria-label={t("聊天内容")}
						maxLength={1000}
					/>
					<button
						type="submit"
						className="icon-button"
						aria-label={t("发送")}
						disabled={!text.trim() || session.busy}
					>
						<Send size={18} />
					</button>
				</form>
			)}
		</section>
	);
}

export function Lobby({ room, session }: { room: RoomView; session: Session }) {
	const t = useText();
	const [settings, setSettings] = useState(room.settings);
	const own = room.you === null ? null : room.seats[room.you];
	const [name, setName] = useState(own?.name ?? t("观众"));
	const [color, setColor] = useState(own?.color ?? 0);
	const occupied = room.seats
		.filter((seat) => seat !== own)
		.map((seat) => seat.color);
	return (
		<main className="lobby">
			<header className="room-heading">
				<button
					type="button"
					className="quiet"
					onClick={() => void session.run("leave")}
				>
					<ArrowLeft size={18} />
					{t("返回")}
				</button>
				<div>
					<div className="eyebrow">{t("房间大厅")}</div>
					<h1>{room.settings.name}</h1>
				</div>
				<span className="badge">
					<Users size={16} /> {room.seats.length} / {room.settings.capacity}
				</span>
			</header>
			{room.host && (
				<button
					type="button"
					className="primary"
					style={{ marginBottom: 22 }}
					disabled={
						session.busy ||
						room.seats.length < 2 ||
						room.seats.some((seat) => !seat.connected || !seat.ready)
					}
					onClick={() => void session.act({ type: "start" })}
				>
					{t("开始游戏")}
				</button>
			)}
			<div className="lobby-grid">
				<section className="paper seats">
					<h2>{t("围坐的人们")}</h2>
					<p className="muted">
						{room.settings.mode === "cities"
							? t("城市与骑士 · 13 分获胜")
							: t("基础规则 · 10 分获胜")}
					</p>
					{room.seats.map((seat, index) => (
						<div className="seat" key={seat.color}>
							<div
								className="avatar"
								style={{ background: colors[seat.color] }}
							>
								{seat.name.slice(0, 1)}
							</div>
							<div className="seat-name">
								<strong>{seat.name}</strong>
								<small>
									{index === room.you
										? t("自己的席位")
										: t("玩家 {0}", index + 1)}
								</small>
							</div>
							{index === 0 && <Crown size={17} className="gold" />}
							<span className={`seat-status ${seat.ready ? "ready" : ""}`}>
								{!seat.connected
									? t("已断线")
									: seat.ready
										? t("已准备")
										: t("准备中")}
							</span>
						</div>
					))}
					{[0, 1, 2, 3, 4, 5]
						.slice(room.seats.length, room.settings.capacity)
						.map((slot) => (
							<div className="seat vacant" key={slot}>
								<div className="avatar">
									<Users size={20} />
								</div>
								<span>{t("留给下一位旅人")}</span>
							</div>
						))}
					<div className="room-addresses">
						{t("主机地址")}
						{session.view?.addresses.map((address) => (
							<code key={address}>{address}</code>
						))}
					</div>
					{room.spectators.length > 0 && (
						<p className="muted">
							{t("观战：{0}", room.spectators.join(t("、")))}
						</p>
					)}
				</section>
				<div className="lobby-controls">
					<form
						className="paper"
						onSubmit={(event) => {
							event.preventDefault();
							void session.act({ type: "profile", name, color });
						}}
					>
						<h3>{t("自己的席位")}</h3>
						<label>
							{t("名称")}
							<input
								value={name}
								onChange={(event) => setName(event.target.value)}
								maxLength={24}
								required
							/>
						</label>
						<ColorPicker
							value={color}
							occupied={occupied}
							onChange={setColor}
						/>
						<div className="form-row">
							<button type="submit" disabled={session.busy}>
								{t("应用")}
							</button>
							{own && (
								<button
									type="button"
									className={own.ready ? "quiet" : "primary"}
									disabled={session.busy}
									onClick={() =>
										void session.act({ type: "ready", ready: !own.ready })
									}
								>
									{own.ready ? t("取消准备") : t("准备就绪")}
								</button>
							)}
						</div>
					</form>
					{room.host && (
						<form
							className="paper"
							onSubmit={(event) => {
								event.preventDefault();
								void session.act({ type: "configure", settings });
							}}
						>
							<h3>{t("房间设置")}</h3>
							<label>
								{t("房间名称")}
								<input
									value={settings.name}
									required
									maxLength={48}
									onChange={(event) =>
										setSettings({ ...settings, name: event.target.value })
									}
								/>
							</label>
							<div className="form-row">
								<label>
									{t("玩法")}
									<select
										value={settings.mode}
										onChange={(event) =>
											setSettings({
												...settings,
												mode:
													event.target.value === "cities" ? "cities" : "base",
											})
										}
									>
										<option value="base">{t("基础规则")}</option>
										<option value="cities">{t("城市与骑士")}</option>
									</select>
								</label>
								<label>
									{t("人数")}
									<select
										value={settings.capacity}
										onChange={(event) =>
											setSettings({
												...settings,
												capacity: Number(event.target.value),
											})
										}
									>
										{[2, 3, 4, 5, 6]
											.filter((count) => count >= room.seats.length)
											.map((count) => (
												<option key={count} value={count}>
													{t("{0} 人", count)}
												</option>
											))}
									</select>
								</label>
							</div>
							<label>
								{t("起始玩家")}
								<select
									value={settings.starter ?? "random"}
									onChange={(event) =>
										setSettings({
											...settings,
											starter:
												event.target.value === "random"
													? null
													: Number(event.target.value),
										})
									}
								>
									<option value="random">{t("随机")}</option>
									{room.seats.map((seat, index) => (
										<option key={seat.color} value={index}>
											{seat.name}
										</option>
									))}
								</select>
							</label>
							<button type="submit" disabled={session.busy}>
								{t("应用房间设置")}
							</button>
						</form>
					)}
				</div>
				<ChatPanel room={room} session={session} />
			</div>
		</main>
	);
}
