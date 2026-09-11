import {
	ArrowLeft,
	Crown,
	Layers,
	MessageCircle,
	Route,
	ScrollText,
	Swords,
} from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { Art } from "./art";
import type { Action, GameView, RoomView } from "./bindings";
import { BoardCanvas } from "./board";
import { CityPanel } from "./cities";
import { KnightPanel } from "./knights";
import { ChatPanel } from "./lobby";
import { useText } from "./locale";
import { useGameMotion } from "./motion";
import { colors } from "./palette";
import { EventSymbol, IslandEvents, ProgressCards } from "./progress";
import { CardPicker, Cost, ResourceCards } from "./resources";
import type { BoardOption } from "./scene";
import type { Session } from "./session";
import { Trading } from "./trade";
import "./game.css";

const dots: Record<number, readonly number[]> = {
	1: [4],
	2: [0, 8],
	3: [0, 4, 8],
	4: [0, 2, 6, 8],
	5: [0, 2, 4, 6, 8],
	6: [0, 2, 3, 5, 6, 8],
};

function Die({ value, red = false }: { value: number; red?: boolean }) {
	const t = useText();
	return (
		<span
			className={`die ${red ? "red" : ""}`}
			role="img"
			aria-label={t("{0} 点", value)}
		>
			{dots[value]?.map((position) => (
				<i
					key={position}
					style={{
						gridColumn: (position % 3) + 1,
						gridRow: Math.floor(position / 3) + 1,
					}}
				/>
			))}
		</span>
	);
}

export function GameTable({
	room,
	game,
	session,
}: {
	room: RoomView;
	game: GameView;
	session: Session;
}) {
	const t = useText();
	const review = session.view?.connection === "review";
	const element = useRef<HTMLElement>(null);
	useGameMotion(element, game, session.view?.preferences.animation ?? 1);
	const [tool, setTool] = useState("");
	const [tab, setTab] = useState<"events" | "chat">("events");
	const [eventCount, setEventCount] = useState(60);
	const act = useCallback(
		(action: Action) => {
			void session.act({ type: "game", action });
		},
		[session.act],
	);
	const choices = game.actions.filter((choice) => choice.target !== null);
	const tools = choices.filter(
		(choice, index) =>
			choices.findIndex((item) => item.action.type === choice.action.type) ===
			index,
	);
	const selected = tools.some((choice) => choice.action.type === tool)
		? tool
		: tools[0]?.action.type;
	const prompt = game.prompt;
	const ownPrompt = prompt?.player === room.you;
	const options: BoardOption[] = prompt
		? prompt.choices.flatMap((choice) =>
				choice.target
					? [
							{
								target: choice.target,
								action: { type: "pick", value: choice.value } as Action,
							},
						]
					: [],
			)
		: choices.flatMap((choice) =>
				choice.target && choice.action.type === selected
					? [{ target: choice.target, action: choice.action }]
					: [],
			);
	const actor = game.players[prompt?.player ?? game.turn.player];
	const dice = game.turn.dice.at(-1);
	const previousDice =
		game.turn.dice.length > 1 ? game.turn.dice[0] : undefined;
	const primary = game.actions.filter(
		(choice) =>
			!choice.target &&
			![
				"bankTrade",
				"tokens",
				"improve",
				"playCard",
				"playProgress",
				"harborTrade",
				"respondTrade",
				"completeTrade",
				"cancelTrade",
			].includes(choice.action.type),
	);
	const tokenActions = game.actions.filter(
		(choice) => choice.action.type === "tokens",
	);
	const cards = game.private?.cards ?? [];
	const cardTypes = cards.filter(
		(card, index) =>
			cards.findIndex((item) => item.card === card.card) === index,
	);
	return (
		<main className="game" ref={element}>
			<header className="game-heading">
				<button
					type="button"
					className="quiet"
					onClick={() => void session.run("leave")}
				>
					<ArrowLeft size={18} />
					{t("返回")}
				</button>
				<h1>{room.settings.name}</h1>
				<span className="muted">
					{game.mode === "cities" ? t("城市与骑士") : t("基础规则")} ·{" "}
					{t("{0} 分获胜", game.mode === "cities" ? 13 : 10)}
				</span>
				<span className="game-round">
					{game.stage.type === "setup"
						? t("初始放置")
						: t("第 {0} 回合", game.turn.number)}
					{game.humans >= 5 &&
						game.stage.type !== "setup" &&
						(game.turn.player === game.turn.primary
							? t(" · 主回合")
							: t(" · 配对行动"))}
				</span>
			</header>
			<div className="players-strip">
				{game.players.map((player, index) => (
					<section
						key={player.color}
						data-player={index}
						className={`player-panel ${index === game.turn.player ? "current" : ""}`}
						style={{ borderTopColor: colors[player.color] }}
					>
						<div className="player-heading">
							<span
								className="player-dot"
								style={{ background: colors[player.color] }}
							/>
							<strong>
								{index < game.humans
									? player.name
									: t("中立势力 {0}", index - game.humans + 1)}
							</strong>
							{index === room.you && (
								<span className="own-mark">{t("自己")}</span>
							)}
							{index < game.humans && (
								<span className="player-score">
									{index === room.you ? game.private?.points : player.points}
									<small>{t("分")}</small>
								</span>
							)}
						</div>
						<div className="player-details">
							{index < game.humans && (
								<>
									<span title={t("资源与商品")}>
										<Layers size={13} />
										{player.handCount}
									</span>
									<span title={t("发展卡")}>
										<ScrollText size={13} />
										{player.cardCount}
									</span>
								</>
							)}
							<span title={t("最长道路")}>
								<Route size={13} />
								{game.awards.lengths[index]}
								{game.awards.road === index && <Crown size={12} />}
							</span>
							{index < game.humans && (
								<span
									title={game.cities ? t("激活骑士防御力") : t("已使用骑士")}
								>
									<Swords size={13} />
									{game.cities ? game.cities.defense[index] : player.army}
									{game.awards.army === index && <Crown size={12} />}
								</span>
							)}
							{!review && room.seats[index]?.connected === false && (
								<span className="offline">{t("断线")}</span>
							)}
						</div>
					</section>
				))}
			</div>
			<div className="game-layout">
				<section className="table-board">
					<div className="turn-bar">
						<div className="turn-description">
							<strong>
								{game.winner === null
									? actor?.name
									: game.players[game.winner]?.name}
							</strong>
							<span>
								{game.winner !== null
									? t("赢得了这场对局")
									: prompt
										? t(prompt.title)
										: game.stage.type === "setup"
											? game.stage.road === null
												? t("选择起始建筑的位置")
												: t("放置相连的道路")
											: game.stage.type === "production"
												? t("掷骰并开始生产")
												: t("建造、交易或使用卡牌")}
							</span>
						</div>
						{dice && (
							<div className="dice">
								{previousDice && (
									<small className="muted">
										{t("首次 {0}", previousDice[0] + previousDice[1])}
									</small>
								)}
								<Die value={dice[0]} red={game.mode === "cities"} />
								<Die value={dice[1]} />
								{game.cities?.event && (
									<EventSymbol event={game.cities.event} />
								)}
							</div>
						)}
						<div className="turn-actions">
							{primary.map((choice) => (
								<button
									type="button"
									key={choice.action.type}
									className={choice.action.type === "roll" ? "primary" : ""}
									disabled={session.busy}
									onClick={() => act(choice.action)}
								>
									{t(choice.label)}
									<Cost cards={choice.cost} />
								</button>
							))}
						</div>
					</div>
					<BoardCanvas
						game={game}
						options={options}
						act={act}
						strength={session.view?.preferences.animation ?? 1}
					>
						<div className="board-tools">
							{tools.map((choice) => (
								<button
									type="button"
									key={choice.action.type}
									aria-pressed={selected === choice.action.type}
									onClick={() => setTool(choice.action.type)}
								>
									{t(choice.label)}
									<Cost cards={choice.cost} />
								</button>
							))}
							{!prompt && tools.length > 0 && (
								<span>{t("点击棋盘上的高亮位置")}</span>
							)}
						</div>
					</BoardCanvas>
					{game.private ? (
						<div className="hand">
							<ResourceCards
								cards={game.private.hand}
								commodities={game.mode === "cities"}
							/>
							<div className="hand-caption">
								<span>{t("自己的手牌")}</span>
								<small>
									{t(
										"剩余棋子：道路 {0} · 村庄 {1} · 城市 {2}",
										game.players[room.you ?? 0]?.roads ?? 0,
										game.players[room.you ?? 0]?.settlements ?? 0,
										game.players[room.you ?? 0]?.cities ?? 0,
									)}
								</small>
							</div>
						</div>
					) : (
						<div className="spectator-caption">
							{review
								? t("对局记录")
								: t("观战席 · {0}", room.spectators.join(t("、")))}
						</div>
					)}
				</section>
				<aside className="game-sidebar">
					{prompt && (
						<section className="game-panel prompt-panel">
							<h3>
								{ownPrompt
									? t(prompt.title)
									: t("{0}：{1}", actor?.name ?? "", t(prompt.title))}
							</h3>
							{ownPrompt && prompt.cards && (
								<CardPicker
									key={JSON.stringify([
										prompt.player,
										prompt.title,
										prompt.cards.available,
									])}
									selection={prompt.cards}
									busy={session.busy}
									submit={(cards) => act({ type: "selectCards", cards })}
								/>
							)}
							{ownPrompt &&
								prompt.choices
									.filter((choice) => !choice.target)
									.map((choice) => (
										<button
											type="button"
											key={choice.value}
											disabled={session.busy}
											onClick={() => act({ type: "pick", value: choice.value })}
										>
											{t(choice.label)}
										</button>
									))}
							{ownPrompt && prompt.canSkip && (
								<button
									type="button"
									disabled={session.busy}
									onClick={() => act({ type: "skip" })}
								>
									{t("完成选择")}
								</button>
							)}
							{!ownPrompt && (
								<p className="muted">{t("选择完成后继续当前回合。")}</p>
							)}
						</section>
					)}
					{game.winner !== null && (
						<section className="game-panel winner-panel">
							<Crown size={32} />
							<h2>{t("{0}获胜", game.players[game.winner]?.name ?? "")}</h2>
							<p>{t("群岛上的这段故事已写下结局。")}</p>
						</section>
					)}
					<ProgressCards game={game} act={act} busy={session.busy} />
					<IslandEvents game={game} />
					<KnightPanel game={game} room={room} act={act} busy={session.busy} />
					<CityPanel game={game} room={room} act={act} busy={session.busy} />
					<Trading game={game} room={room} act={act} busy={session.busy} />
					{game.humans === 2 && room.you !== null && (
						<section className="game-panel">
							<h3>
								{t("贸易筹码 · {0}", game.players[room.you]?.tokens ?? 0)}
							</h3>
							<p className="muted">{t("供应剩余 {0} 个", game.tokens)}</p>
							<div className="development-cards">
								{tokenActions.map((choice) => (
									<button
										type="button"
										key={t(choice.label)}
										disabled={session.busy}
										onClick={() => act(choice.action)}
									>
										{t(choice.label)}
									</button>
								))}
							</div>
						</section>
					)}
					{cardTypes.length > 0 && (
						<section className="game-panel">
							<h3>{t("发展卡")}</h3>
							<div className="development-cards">
								{cardTypes.map((card) => (
									<button
										type="button"
										key={card.card}
										className="development-card"
										data-card={card.card}
										disabled={
											session.busy ||
											!cards.some(
												(item) => item.card === card.card && item.playable,
											)
										}
										onClick={() => act({ type: "playCard", card: card.card })}
									>
										<Art name={card.card} size={68} />
										<strong>
											{t(card.name)}
											<span>
												×
												{cards.filter((item) => item.card === card.card).length}
											</span>
										</strong>
										<small>{t(card.description)}</small>
									</button>
								))}
							</div>
						</section>
					)}
					<section className="game-panel timeline-panel">
						<div className="panel-tabs">
							<button
								type="button"
								aria-pressed={tab === "events"}
								onClick={() => setTab("events")}
							>
								<ScrollText size={16} />
								{t("记录")}
							</button>
							<button
								type="button"
								aria-pressed={tab === "chat"}
								onClick={() => setTab("chat")}
							>
								<MessageCircle size={16} />
								{t("聊天")}
							</button>
						</div>
						{tab === "chat" ? (
							<ChatPanel room={room} session={session} />
						) : (
							<div
								className="game-events"
								role="log"
								aria-label={t("游戏记录")}
							>
								{game.events.length > eventCount && (
									<button
										type="button"
										className="quiet"
										onClick={() => setEventCount(eventCount + 60)}
									>
										{t("更早的记录")}
									</button>
								)}
								{game.events.slice(-eventCount).map((event) => (
									<p key={event.seq} className={`event event-${event.kind}`}>
										{t(event.text)}
									</p>
								))}
							</div>
						)}
					</section>
				</aside>
			</div>
		</main>
	);
}
