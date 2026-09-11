import { Shield, Swords } from "lucide-react";
import type { Action, GameView, RoomView } from "./bindings";
import { useText } from "./locale";
import { colors } from "./palette";
import { Cost } from "./resources";
import "./knights.css";

const operations = new Set<Action["type"]>([
	"promoteKnight",
	"activateKnight",
	"moveKnight",
	"expelRobber",
	"retireKnight",
]);

export function KnightPanel({
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
	const t = useText();
	const cities = game.cities;
	if (!cities) return null;
	const knights = cities.knights.flatMap((knight, vertex) =>
		knight?.player === room.you ? [{ ...knight, vertex }] : [],
	);
	return (
		<section className="game-panel knights-panel">
			<h3>
				<Shield size={17} />
				{t("岛屿守卫")}
			</h3>
			<div className="defense-scores">
				{game.players.slice(0, game.humans).map((player, index) => (
					<span key={player.color} title={t("{0}的防御力", player.name)}>
						<i style={{ background: colors[player.color] }} />
						{player.name}
						<strong>{cities.defense[index]}</strong>
					</span>
				))}
			</div>
			{room.you !== null && (
				<>
					<p className="muted">
						{t("供应：")}
						{[1, 2, 3]
							.map((level) =>
								t(
									"{0} 级 {1} 枚",
									level,
									2 - knights.filter((knight) => knight.level === level).length,
								),
							)
							.join(" · ")}
					</p>
					{knights.map((knight) => (
						<div className="knight-row" key={knight.vertex}>
							<div className="knight-heading">
								<Swords size={16} />
								<strong>
									{t(
										["", "基础骑士", "强力骑士", "强大骑士"][knight.level] ??
											"",
									)}
								</strong>
								<span>{knight.active ? t("已激活") : t("未激活")}</span>
							</div>
							<div className="knight-actions">
								{game.actions
									.filter(
										(choice) =>
											operations.has(choice.action.type) &&
											choice.target?.type === "vertex" &&
											choice.target.id === knight.vertex,
									)
									.map((choice) => (
										<button
											key={choice.action.type}
											type="button"
											disabled={busy}
											onClick={() => act(choice.action)}
										>
											{t(choice.label)}
											<Cost cards={choice.cost} />
										</button>
									))}
							</div>
						</div>
					))}
					<p className="muted">
						{t(
							"激活的骑士参与防御。从下一个自己的回合起，可以移动或驱逐强盗；行动后重新激活。",
						)}
					</p>
				</>
			)}
		</section>
	);
}
