import { useState } from "react";
import { Art } from "./art";
import type { CardChoice, PrivateView } from "./bindings";
import { useText } from "./locale";

export type Hand = PrivateView["hand"];
export const emptyHand: Hand = [0, 0, 0, 0, 0, 0, 0, 0];
export const resources = [
	{ id: "wood", index: 0, name: "木材" },
	{ id: "brick", index: 1, name: "砖块" },
	{ id: "wool", index: 2, name: "羊毛" },
	{ id: "grain", index: 3, name: "小麦" },
	{ id: "ore", index: 4, name: "矿石" },
	{ id: "cloth", index: 5, name: "布匹" },
	{ id: "coin", index: 6, name: "铸币" },
	{ id: "paper", index: 7, name: "纸张" },
] as const;

export function ResourceCards({
	cards,
	commodities = false,
}: {
	cards: Hand;
	commodities?: boolean;
}) {
	const t = useText();
	return (
		<div className="resource-cards">
			{resources.slice(0, commodities ? 8 : 5).map(({ id, index, name }) => (
				<div
					data-resource={id}
					className={`resource-card ${cards[index] === 0 ? "empty" : ""}`}
					key={id}
					title={t(name)}
				>
					<Art name={id} size={27} />
					<strong>{cards[index]}</strong>
					<span>{t(name)}</span>
				</div>
			))}
		</div>
	);
}

export function Cost({ cards }: { cards: Hand }) {
	const t = useText();
	return (
		<span className="cost">
			{resources
				.filter(({ index }) => cards[index] > 0)
				.map(({ id, index, name }) => (
					<span className="cost-item" key={id} title={t(name)}>
						<Art name={id} size={15} />
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
	const t = useText();
	return (
		<fieldset className="cards-editor">
			<legend>{label}</legend>
			{resources.slice(0, commodities ? 8 : 5).map(({ id, index, name }) => (
				<label key={id} className="card-amount">
					<span>
						<Art name={id} size={19} />
						{t(name)}
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
	const t = useText();
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
				label={t("选择 {0} 张牌", selection.count)}
			/>
			<button
				type="submit"
				className="primary wide"
				disabled={busy || count !== selection.count}
			>
				{t("确定 · {0} / {1}", count, selection.count)}
			</button>
		</form>
	);
}
