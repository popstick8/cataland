import { ArrowLeftRight, Check, Landmark, X } from "lucide-react";
import { useState } from "react";
import { Art } from "./art";
import type { Action, GameView, Resource, RoomView } from "./bindings";
import { useText } from "./locale";
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
	const t = useText();
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
					<Landmark size={17} />
					{t("银行与港口")}
				</summary>
				<div className="bank-stock">
					{choices.map(({ id, index, name }) => (
						<span key={id} title={t("{0}库存", t(name))}>
							<Art name={id} size={17} />
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
								{t("给出")}
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
											{t("{0} × {1}", t(name), game.private?.rates[index] ?? 0)}
										</option>
									))}
								</select>
							</label>
							<ArrowLeftRight size={17} />
							<label>
								{t("换取")}
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
											{t("{0} × {1}", t(name), 1)}
										</option>
									))}
								</select>
							</label>
						</div>
						<button type="submit" className="wide" disabled={busy || !exchange}>
							{t("兑换")}
							{exchange && <Cost cards={exchange.cost} />}
						</button>
					</form>
				)}
			</details>
			{trade && (
				<div className="trade-offer">
					<h3>{t("{0}的报价", game.players[trade.player]?.name ?? "")}</h3>
					<p>
						<span>{t("给出")}</span>
						<Cost cards={trade.give} />
					</p>
					<p>
						<span>{t("索取")}</span>
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
								key={t(choice.label)}
								disabled={busy}
								onClick={() => act(choice.action)}
							>
								{t(choice.label)}
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
					{trade ? t("修改报价") : t("与玩家交易")}
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
						label={t("给出")}
						value={offered}
						onChange={setOffered}
						available={balance}
						commodities={game.mode === "cities"}
					/>
					<CardsEditor
						label={t("索取")}
						value={wanted}
						onChange={setWanted}
						available={supply}
						commodities={game.mode === "cities"}
					/>
					<div className="form-row">
						<button type="button" onClick={() => setOffering(false)}>
							{t("取消")}
						</button>
						<button type="submit" className="primary" disabled={busy || !valid}>
							{t("发送报价")}
						</button>
					</div>
				</form>
			)}
		</section>
	);
}
