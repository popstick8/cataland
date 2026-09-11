import {
	ArrowLeft,
	Crown,
	Layers,
	MessageCircle,
	Route,
	ScrollText,
	Swords,
} from "lucide-react";
import { useCallback, useState } from "react";
import type { Action, GameView, RoomView } from "./bindings";
import { BoardCanvas } from "./board";
import { ChatPanel } from "./lobby";
import { colors } from "./palette";
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
	return (
		<span
			className={`die ${red ? "red" : ""}`}
			role="img"
			aria-label={`${value} 点`}
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
	const primary = game.actions.filter(
		(choice) =>
			!choice.target &&
			![
				"bankTrade",
				"playCard",
				"respondTrade",
				"completeTrade",
				"cancelTrade",
			].includes(choice.action.type),
	);
	const cards = game.private?.cards ?? [];
	const cardTypes = cards.filter(
		(card, index) =>
			cards.findIndex((item) => item.card === card.card) === index,
	);
	return (
		<main className="game">
			<header className="game-heading">
				<button
					type="button"
					className="quiet"
					onClick={() => void session.run("leave")}
				>
					<ArrowLeft size={18} /> 返回
				</button>
				<h1>{room.settings.name}</h1>
				<span className="muted">
					{game.mode === "cities" ? "城市与骑士" : "基础规则"} ·{" "}
					{game.mode === "cities" ? 13 : 10} 分获胜
				</span>
				<span className="game-round">
					{game.stage.type === "setup"
						? "初始放置"
						: `第 ${game.turn.number} 回合`}
				</span>
			</header>
			<div className="players-strip">
				{game.players.map((player, index) => (
					<section
						key={player.color}
						className={`player-panel ${index === game.turn.player ? "current" : ""}`}
						style={{ borderTopColor: colors[player.color] }}
					>
						<div className="player-heading">
							<span
								className="player-dot"
								style={{ background: colors[player.color] }}
							/>
							<strong>{player.name}</strong>
							{index === room.you && <span className="own-mark">自己</span>}
							<span className="player-score">
								{index === room.you ? game.private?.points : player.points}
								<small>分</small>
							</span>
						</div>
						<div className="player-details">
							<span title="资源与商品">
								<Layers size={13} />
								{player.handCount}
							</span>
							<span title="发展卡">
								<ScrollText size={13} />
								{player.cardCount}
							</span>
							<span title="最长道路">
								<Route size={13} />
								{game.awards.lengths[index]}
								{game.awards.road === index && <Crown size={12} />}
							</span>
							<span title="已使用骑士">
								<Swords size={13} />
								{player.army}
								{game.awards.army === index && <Crown size={12} />}
							</span>
							{room.seats[index]?.connected === false && (
								<span className="offline">断线</span>
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
									? "赢得了这场对局"
									: prompt
										? prompt.title
										: game.stage.type === "setup"
											? game.stage.road === null
												? "选择起始建筑的位置"
												: "放置相连的道路"
											: game.stage.type === "production"
												? "掷骰并开始生产"
												: "建造、交易或使用卡牌"}
							</span>
						</div>
						{dice && (
							<div className="dice">
								<Die value={dice[0]} red={game.mode === "cities"} />
								<Die value={dice[1]} />
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
									{choice.label}
									<Cost cards={choice.cost} />
								</button>
							))}
						</div>
					</div>
					<div className="board-tools">
						{tools.map((choice) => (
							<button
								type="button"
								key={choice.action.type}
								aria-pressed={selected === choice.action.type}
								onClick={() => setTool(choice.action.type)}
							>
								{choice.label}
								<Cost cards={choice.cost} />
							</button>
						))}
						{!prompt && tools.length > 0 && <span>点击棋盘上的高亮位置</span>}
					</div>
					<BoardCanvas game={game} options={options} act={act} />
					{game.private ? (
						<div className="hand">
							<ResourceCards
								cards={game.private.hand}
								commodities={game.mode === "cities"}
							/>
							<div className="hand-caption">
								<span>自己的手牌</span>
								<small>
									剩余棋子：道路 {game.players[room.you ?? 0]?.roads} · 村庄{" "}
									{game.players[room.you ?? 0]?.settlements} · 城市{" "}
									{game.players[room.you ?? 0]?.cities}
								</small>
							</div>
						</div>
					) : (
						<div className="spectator-caption">
							观战席 · {room.spectators.join("、")}
						</div>
					)}
				</section>
				<aside className="game-sidebar">
					{prompt && (
						<section className="game-panel prompt-panel">
							<h3>
								{ownPrompt ? prompt.title : `${actor?.name}：${prompt.title}`}
							</h3>
							{ownPrompt && prompt.cards && (
								<CardPicker
									key={`${prompt.player}-${prompt.title}-${prompt.cards.available.join()}`}
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
											{choice.label}
										</button>
									))}
							{ownPrompt && prompt.canSkip && (
								<button
									type="button"
									disabled={session.busy}
									onClick={() => act({ type: "skip" })}
								>
									完成放置
								</button>
							)}
							{!ownPrompt && <p className="muted">选择完成后继续当前回合。</p>}
						</section>
					)}
					{game.winner !== null && (
						<section className="game-panel winner-panel">
							<Crown size={32} />
							<h2>{game.players[game.winner]?.name}获胜</h2>
							<p>群岛上的这段故事已写下结局。</p>
						</section>
					)}
					<Trading game={game} room={room} act={act} busy={session.busy} />
					{cardTypes.length > 0 && (
						<section className="game-panel">
							<h3>发展卡</h3>
							<div className="development-cards">
								{cardTypes.map((card) => (
									<button
										type="button"
										key={card.card}
										className="development-card"
										disabled={
											session.busy ||
											!cards.some(
												(item) => item.card === card.card && item.playable,
											)
										}
										onClick={() => act({ type: "playCard", card: card.card })}
									>
										<strong>
											{card.name}
											<span>
												×
												{cards.filter((item) => item.card === card.card).length}
											</span>
										</strong>
										<small>{card.description}</small>
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
								<ScrollText size={16} /> 记录
							</button>
							<button
								type="button"
								aria-pressed={tab === "chat"}
								onClick={() => setTab("chat")}
							>
								<MessageCircle size={16} /> 聊天
							</button>
						</div>
						{tab === "chat" ? (
							<ChatPanel room={room} session={session} />
						) : (
							<div className="game-events" role="log" aria-label="游戏记录">
								{game.events.length > eventCount && (
									<button
										type="button"
										className="quiet"
										onClick={() => setEventCount(eventCount + 60)}
									>
										更早的记录
									</button>
								)}
								{game.events.slice(-eventCount).map((event) => (
									<p key={event.seq} className={`event event-${event.kind}`}>
										{event.text}
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
