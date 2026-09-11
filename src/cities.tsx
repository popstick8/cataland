import { Coins, Crown, FlaskConical, Landmark } from "lucide-react";
import type { Action, GameView, RoomView, Track } from "./bindings";
import { Cost } from "./resources";
import "./cities.css";

export const tracks = {
	trade: {
		name: "贸易",
		Icon: Coins,
		color: "#ab8746",
		ability: "三级商会：任意两张相同商品可兑换一张资源或商品。",
	},
	politics: {
		name: "政治",
		Icon: Landmark,
		color: "#6b8aab",
		ability: "三级城堡：强力骑士可晋升为强大骑士。",
	},
	science: {
		name: "科学",
		Icon: FlaskConical,
		color: "#6d9b72",
		ability:
			"三级引水渠：非 7 生产中没有收到资源或商品时，可选择一张基础资源。",
	},
} satisfies Record<
	Track,
	{ name: string; Icon: typeof Coins; color: string; ability: string }
>;

export function CityPanel({
	game,
	room,
	act,
	busy,
}: {
	game: GameView;
	room: RoomView;
	act: (action: Action) => void;
	busy: boolean;
}) {
	const cities = game.cities;
	if (!cities) return null;
	const own = game.private;
	return (
		<section className="game-panel cities-panel">
			<h3>城市发展</h3>
			{own?.improvements.map((improvement, index) => {
				const track = tracks[improvement.track];
				const upgrade = game.actions.find(
					(choice) =>
						choice.action.type === "improve" &&
						choice.action.track === improvement.track,
				);
				const metropolis = cities.metropolises[index];
				const holder =
					metropolis === undefined || metropolis === null
						? null
						: game.buildings[metropolis]?.player;
				return (
					<div className="city-improvement" key={improvement.track}>
						<div className="improvement-heading">
							<track.Icon size={18} style={{ color: track.color }} />
							<strong>{track.name}</strong>
							<span>{improvement.name}</span>
						</div>
						<div
							className="improvement-levels"
							role="img"
							aria-label={`${track.name} ${improvement.level} 级`}
						>
							{[1, 2, 3, 4, 5].map((level) => (
								<i
									key={level}
									style={{
										background:
											level <= improvement.level ? track.color : undefined,
									}}
								/>
							))}
						</div>
						<p
							className={`improvement-ability ${improvement.level >= 3 ? "active" : ""}`}
						>
							{track.ability}
						</p>
						{holder !== null && holder !== undefined && (
							<p className="metropolis-holder">
								<Crown size={13} />
								{game.players[holder]?.name}的{track.name}大都会
							</p>
						)}
						{improvement.next && (
							<button
								type="button"
								className="wide"
								disabled={busy || !upgrade}
								onClick={() => {
									if (upgrade) act(upgrade.action);
								}}
							>
								建成{improvement.next}
								<Cost cards={improvement.cost} />
							</button>
						)}
					</div>
				);
			})}
			{own && (
				<p className="city-hand-limit">手牌超过 {own.handLimit} 张时弃掉一半</p>
			)}
			<details className="city-overview">
				<summary>全岛城市发展</summary>
				<table>
					<thead>
						<tr>
							<th>玩家</th>
							<th>贸易</th>
							<th>政治</th>
							<th>科学</th>
						</tr>
					</thead>
					<tbody>
						{game.players.slice(0, game.humans).map((player, index) => (
							<tr
								key={player.color}
								className={index === room.you ? "own" : ""}
							>
								<th>{player.name}</th>
								{([0, 1, 2] as const).map((track) => (
									<td key={track}>{cities.upgrades[index]?.[track]}</td>
								))}
							</tr>
						))}
					</tbody>
				</table>
			</details>
		</section>
	);
}
