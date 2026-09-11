import { Crown, Shield, Ship } from "lucide-react";
import { Art } from "./art";
import type { Action, EventDie, GameView } from "./bindings";
import { tracks } from "./cities";
import "./progress.css";

export function EventSymbol({ event }: { event: EventDie }) {
	const Icon = event === "barbarians" ? Ship : tracks[event].Icon;
	return (
		<span
			className={`event-die event-die-${event}`}
			title={event === "barbarians" ? "野蛮人船" : `${tracks[event].name}城门`}
		>
			<Icon size={22} />
		</span>
	);
}

export function IslandEvents({ game }: { game: GameView }) {
	const cities = game.cities;
	if (!cities) return null;
	const strength = game.buildings.filter(
		(building) => building?.kind === "city",
	).length;
	const defense = cities.defense.reduce((sum, power) => sum + power, 0);
	return (
		<section className="game-panel island-events">
			<h3>
				<Ship size={17} /> 海上来客
			</h3>
			<div
				className="barbarian-track"
				role="img"
				aria-label={`野蛮人船位于第 ${cities.barbarians} 格，共七格`}
			>
				{[0, 1, 2, 3, 4, 5, 6, 7].map((step) => (
					<span
						key={step}
						className={step <= cities.barbarians ? "reached" : ""}
					>
						{step === 7 ? <Shield size={12} /> : step}
					</span>
				))}
				<Ship
					className="barbarian-ship"
					size={24}
					style={{ left: `${((cities.barbarians + 0.5) / 8) * 100}%` }}
				/>
			</div>
			<div className="battle-strength">
				<span>
					野蛮人 <strong>{strength}</strong>
				</span>
				<span>
					岛屿防御 <strong>{defense}</strong>
				</span>
			</div>
			<p className="muted">
				{cities.attacks === 0
					? "第一次来袭后，强盗进入岛屿。"
					: `已经经历 ${cities.attacks} 次来袭。`}
			</p>
			<div className="progress-decks">
				{(["trade", "politics", "science"] as const).map((track, index) => {
					const { Icon, name, color } = tracks[track];
					return (
						<span key={track} style={{ color }}>
							<Icon size={14} />
							{name}
							<strong>{cities.decks[index]}</strong>
						</span>
					);
				})}
			</div>
			{game.players
				.filter((player) => player.defender > 0 || player.revealed.length > 0)
				.map((player) => (
					<p className="public-achievements" key={player.color}>
						<Crown size={13} />
						{player.name}：
						{[
							...(player.defender ? [`守护者 ${player.defender} 分`] : []),
							...player.revealed.map((card) =>
								card === "constitution" ? "宪法" : "印刷术",
							),
						].join(" · ")}
					</p>
				))}
		</section>
	);
}

export function ProgressCards({
	game,
	act,
	busy,
}: {
	game: GameView;
	act: (action: Action) => void;
	busy: boolean;
}) {
	const cards = game.private?.progress ?? [];
	const kinds = cards.filter(
		(card, index) =>
			cards.findIndex((item) => item.card === card.card) === index,
	);
	const harbor = game.actions.filter(
		(choice) => choice.action.type === "harborTrade",
	);
	if (!game.cities || !game.private) return null;
	return (
		<section className="game-panel">
			<h3>
				进步卡 <small className="muted">{cards.length} / 4</small>
			</h3>
			<div className="progress-cards">
				{kinds.map((card) => {
					const { color } = tracks[card.track];
					return (
						<button
							key={card.card}
							type="button"
							className="progress-card"
							style={{ borderLeftColor: color }}
							disabled={busy || !card.playable}
							onClick={() => act({ type: "playProgress", card: card.card })}
						>
							<Art name={card.card} size={72} />
							<span>
								<strong>
									{card.name}
									<small>
										×{cards.filter((held) => held.card === card.card).length}
									</small>
								</strong>
								<small>{card.description}</small>
							</span>
						</button>
					);
				})}
				{cards.length === 0 && (
					<p className="muted">发展城市，在对应城门出现时取得进步卡。</p>
				)}
			</div>
			{harbor.length > 0 && (
				<div className="harbor-offers">
					<h4>商业港</h4>
					{harbor.map((choice) => (
						<button
							key={choice.label}
							type="button"
							disabled={busy}
							onClick={() => act(choice.action)}
						>
							{choice.label}
						</button>
					))}
				</div>
			)}
		</section>
	);
}
