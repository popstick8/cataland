import { ArrowLeftRight, Check, Landmark, X } from "lucide-react";
import { useState } from "react";
import type { Action, GameView, Resource, RoomView } from "./bindings";
import {
	CardsEditor,
	Cost,
	emptyHand,
	type Hand,
	resources,
} from "./resources";
import "./trade.css";

export function Trading({
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
	const [give, setGive] = useState<Resource>("wood");
	const [take, setTake] = useState<Resource>("grain");
	const [offering, setOffering] = useState(false);
	const [offered, setOffered] = useState<Hand>([...emptyHand]);
	const [wanted, setWanted] = useState<Hand>([...emptyHand]);
	const choices = resources.slice(0, game.mode === "cities" ? 8 : 5);
	const exchange = game.actions.find(
		(choice) =>
			choice.action.type === "bankTrade" &&
			choice.action.give === give &&
			choice.action.take === take,
	);
	const trade = game.trade;
	const balance = game.private?.hand;
	const supply: Hand = [...emptyHand];
	for (const resource of choices)
		supply[resource.index] =
			resource.index < 5
				? game.board.hexes.length > 19
					? 24
					: 19
				: game.board.hexes.length > 19
					? 18
					: 12;
	const valid =
		offered.some((value) => value > 0) &&
		wanted.some((value) => value > 0) &&
		offered.every(
			(value, index) =>
				!(value > 0 && (wanted[index] ?? 0) > 0) &&
				value <= (balance?.[index] ?? 0),
		);
	const responses = game.actions.filter((choice) =>
		["respondTrade", "completeTrade", "cancelTrade"].includes(
			choice.action.type,
		),
	);
	return (
		<section className="game-panel trading">
			<details className="bank-panel">
				<summary>
					<Landmark size={17} /> 银行与港口
				</summary>
				<div className="bank-stock">
					{choices.map(({ id, index, name, Icon, color }) => (
						<span key={id} title={`${name}库存`}>
							<Icon size={17} style={{ color }} />
							{game.bank[index]}
						</span>
					))}
				</div>
				{balance && (
					<form
						onSubmit={(event) => {
							event.preventDefault();
							if (exchange) act(exchange.action);
						}}
					>
						<div className="exchange-fields">
							<label>
								给出
								<select
									value={give}
									onChange={(event) => {
										const item = choices.find(
											(item) => item.id === event.target.value,
										);
										if (item) setGive(item.id);
									}}
								>
									{choices.map(({ id, index, name }) => (
										<option key={id} value={id}>
											{name} · {game.private?.rates[index]} 张
										</option>
									))}
								</select>
							</label>
							<ArrowLeftRight size={17} />
							<label>
								换取
								<select
									value={take}
									onChange={(event) => {
										const item = choices.find(
											(item) => item.id === event.target.value,
										);
										if (item) setTake(item.id);
									}}
								>
									{choices.map(({ id, name }) => (
										<option key={id} value={id}>
											{name} · 1 张
										</option>
									))}
								</select>
							</label>
						</div>
						<button type="submit" className="wide" disabled={busy || !exchange}>
							兑换{exchange && <Cost cards={exchange.cost} />}
						</button>
					</form>
				)}
			</details>
			{trade && (
				<div className="trade-offer">
					<h3>{game.players[trade.player]?.name}的报价</h3>
					<p>
						<span>给出</span>
						<Cost cards={trade.give} />
					</p>
					<p>
						<span>索取</span>
						<Cost cards={trade.want} />
					</p>
					<div className="trade-responses">
						{trade.responses.map((response, player) =>
							player === trade.player ? null : (
								<span
									key={game.players[player]?.color}
									title={game.players[player]?.name}
								>
									{game.players[player]?.name}
									{response === true ? (
										<Check size={13} />
									) : response === false ? (
										<X size={13} />
									) : null}
								</span>
							),
						)}
					</div>
					<div className="trade-buttons">
						{responses.map((choice) => (
							<button
								type="button"
								key={choice.label}
								disabled={busy}
								onClick={() => act(choice.action)}
							>
								{choice.label}
							</button>
						))}
					</div>
				</div>
			)}
			{game.private?.canOffer && !offering && (
				<button
					type="button"
					className="wide offer-button"
					onClick={() => {
						setOffered(
							trade?.player === room.you ? [...trade.give] : [...emptyHand],
						);
						setWanted(
							trade?.player === room.you ? [...trade.want] : [...emptyHand],
						);
						setOffering(true);
					}}
				>
					<ArrowLeftRight size={16} />
					{trade ? "修改报价" : "与玩家交易"}
				</button>
			)}
			{offering && game.private?.canOffer && balance && (
				<form
					className="offer-editor"
					onSubmit={(event) => {
						event.preventDefault();
						act({ type: "offerTrade", give: offered, want: wanted });
						setOffering(false);
					}}
				>
					<CardsEditor
						label="给出"
						value={offered}
						onChange={setOffered}
						available={balance}
						commodities={game.mode === "cities"}
					/>
					<CardsEditor
						label="索取"
						value={wanted}
						onChange={setWanted}
						available={supply}
						commodities={game.mode === "cities"}
					/>
					<div className="form-row">
						<button type="button" onClick={() => setOffering(false)}>
							取消
						</button>
						<button type="submit" className="primary" disabled={busy || !valid}>
							发送报价
						</button>
					</div>
				</form>
			)}
		</section>
	);
}
