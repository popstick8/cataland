import {
	BrickWall,
	Cloud,
	Coins,
	Gem,
	ScrollText,
	Shirt,
	Trees,
	Wheat,
} from "lucide-react";
import { useState } from "react";
import type { CardChoice, PrivateView } from "./bindings";

export type Hand = PrivateView["hand"];
export const emptyHand: Hand = [0, 0, 0, 0, 0, 0, 0, 0];
export const resources = [
	{ id: "wood", index: 0, name: "木材", Icon: Trees, color: "#64876c" },
	{ id: "brick", index: 1, name: "砖块", Icon: BrickWall, color: "#b37356" },
	{ id: "wool", index: 2, name: "羊毛", Icon: Cloud, color: "#91a77c" },
	{ id: "grain", index: 3, name: "小麦", Icon: Wheat, color: "#b69545" },
	{ id: "ore", index: 4, name: "矿石", Icon: Gem, color: "#778a8a" },
	{ id: "cloth", index: 5, name: "布匹", Icon: Shirt, color: "#b18a69" },
	{ id: "coin", index: 6, name: "铸币", Icon: Coins, color: "#b99545" },
	{ id: "paper", index: 7, name: "纸张", Icon: ScrollText, color: "#7c9d81" },
] as const;

export function ResourceCards({
	cards,
	commodities = false,
}: {
	cards: Hand;
	commodities?: boolean;
}) {
	return (
		<div className="resource-cards">
			{resources
				.slice(0, commodities ? 8 : 5)
				.map(({ id, index, name, Icon, color }) => (
					<div
						className={`resource-card ${cards[index] === 0 ? "empty" : ""}`}
						key={id}
						title={name}
					>
						<Icon size={27} style={{ color }} />
						<strong>{cards[index]}</strong>
						<span>{name}</span>
					</div>
				))}
		</div>
	);
}

export function Cost({ cards }: { cards: Hand }) {
	return (
		<span className="cost">
			{resources
				.filter(({ index }) => cards[index] > 0)
				.map(({ id, index, name, Icon, color }) => (
					<span className="cost-item" key={id} title={name}>
						<Icon size={15} style={{ color }} />
						{cards[index]}
					</span>
				))}
		</span>
	);
}

export function CardsEditor({
	value,
	onChange,
	available,
	commodities = false,
	label,
}: {
	value: Hand;
	onChange: (value: Hand) => void;
	available: Hand;
	commodities?: boolean;
	label: string;
}) {
	return (
		<fieldset className="cards-editor">
			<legend>{label}</legend>
			{resources
				.slice(0, commodities ? 8 : 5)
				.map(({ id, index, name, Icon, color }) => (
					<label key={id} className="card-amount">
						<span>
							<Icon size={19} style={{ color }} />
							{name}
						</span>
						<input
							type="number"
							min={0}
							max={available[index]}
							value={value[index]}
							onChange={(event) => {
								const next: Hand = [...value];
								next[index] = Math.max(
									0,
									Math.min(
										available[index],
										Math.trunc(Number(event.target.value)),
									),
								);
								onChange(next);
							}}
						/>
					</label>
				))}
		</fieldset>
	);
}

export function CardPicker({
	selection,
	submit,
	busy,
}: {
	selection: CardChoice;
	submit: (cards: Hand) => void;
	busy: boolean;
}) {
	const [cards, setCards] = useState<Hand>([...emptyHand]);
	const count = cards.reduce((sum, value) => sum + value, 0);
	return (
		<form
			onSubmit={(event) => {
				event.preventDefault();
				submit(cards);
			}}
		>
			<CardsEditor
				value={cards}
				onChange={setCards}
				available={selection.available}
				commodities={selection.available.slice(5).some((value) => value > 0)}
				label={`选择 ${selection.count} 张牌`}
			/>
			<button
				type="submit"
				className="primary wide"
				disabled={busy || count !== selection.count}
			>
				确定 · {count} / {selection.count}
			</button>
		</form>
	);
}
