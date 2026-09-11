import { type RefObject, useLayoutEffect, useRef } from "react";
import type { GameView } from "./bindings";
import { resources } from "./resources";

export function useGameMotion(
	root: RefObject<HTMLElement | null>,
	game: GameView,
	strength: number,
) {
	const previous = useRef<GameView | null>(null);
	const running = useRef(new Map<Element, () => void>());
	useLayoutEffect(
		() => () => {
			for (const cancel of running.current.values()) cancel();
		},
		[],
	);
	useLayoutEffect(() => {
		const before = previous.current;
		previous.current = game;
		const element = root.current;
		if (!element || !before) return;
		if (strength === 0) {
			for (const cancel of running.current.values()) cancel();
			return;
		}
		const animate = (
			target: Element,
			frames: Keyframe[],
			duration: number,
			dispose?: () => void,
			finish?: () => void,
		) => {
			running.current.get(target)?.();
			const animation = target.animate(frames, {
				duration,
				easing: "cubic-bezier(0.2, 0.7, 0.2, 1)",
			});
			const cancel = () => {
				animation.onfinish = null;
				animation.cancel();
				dispose?.();
				running.current.delete(target);
			};
			running.current.set(target, cancel);
			animation.onfinish = () => {
				cancel();
				finish?.();
			};
		};
		const pulse = (target: Element) =>
			animate(
				target,
				[
					{ transform: "scale(1)" },
					{ transform: `scale(${1 + 0.065 * strength})`, offset: 0.35 },
					{ transform: "scale(1)" },
				],
				180 + 120 * strength,
			);
		const recent = game.events.filter(
			(event) => event.seq > (before.events.at(-1)?.seq ?? 0),
		);
		if (recent.some((event) => event.kind === "roll")) {
			for (const [index, die] of element
				.querySelectorAll(".dice .die, .dice .event-die")
				.entries()) {
				const turn = (index % 2 === 0 ? -1 : 1) * 270 * strength;
				animate(
					die,
					[
						{
							transform: `translateY(${-14 * strength}px) rotate(${turn}deg) scale(0.85)`,
						},
						{
							transform: "translateY(2px) rotate(-6deg) scale(1.04)",
							offset: 0.75,
						},
						{ transform: "none" },
					],
					250 + 180 * strength + index * 25,
				);
			}
		}
		const board = element.querySelector(".board-shell");
		if (board && before.private && game.private) {
			const supply = board.getBoundingClientRect();
			for (const { id, index } of resources) {
				const difference =
					game.private.hand[index] - before.private.hand[index];
				if (difference === 0) continue;
				const card = element.querySelector(`[data-resource="${id}"]`);
				const art = card?.querySelector(".illustration");
				if (!card || !art) continue;
				const hand = card.getBoundingClientRect();
				const source = difference > 0 ? supply : hand;
				const destination = difference > 0 ? hand : supply;
				const dx =
					destination.x + destination.width / 2 - source.x - source.width / 2;
				const dy =
					destination.y + destination.height / 2 - source.y - source.height / 2;
				const flight = document.createElement("span");
				flight.className = "resource-flight";
				flight.ariaHidden = "true";
				flight.style.left = `${source.x + source.width / 2 - 29}px`;
				flight.style.top = `${source.y + source.height / 2 - 20}px`;
				flight.append(art.cloneNode(true), String(Math.abs(difference)));
				document.body.append(flight);
				animate(
					flight,
					[
						{ transform: "translate(0, 0) scale(0.65)", opacity: 0 },
						{ opacity: 1, offset: 0.16 },
						{
							transform: `translate(${dx}px, ${dy}px) scale(0.8)`,
							opacity: 0.15,
						},
					],
					260 + 180 * strength,
					() => flight.remove(),
					() => {
						if (card.isConnected) pulse(card);
					},
				);
			}
		}
		const oldCards = [
			...(before.private?.cards ?? []),
			...(before.private?.progress ?? []),
		];
		const cards = [
			...(game.private?.cards ?? []),
			...(game.private?.progress ?? []),
		];
		for (const card of new Set(cards.map((held) => held.card))) {
			if (
				cards.filter((held) => held.card === card).length <=
				oldCards.filter((held) => held.card === card).length
			)
				continue;
			const target = element.querySelector(`[data-card="${card}"]`);
			if (target)
				animate(
					target,
					[
						{ opacity: 0, transform: `translateY(${12 * strength}px)` },
						{ opacity: 1, transform: "none" },
					],
					220 + 150 * strength,
				);
		}
		for (const [index, player] of game.players.entries()) {
			if (
				player.points !== before.players[index]?.points ||
				player.handCount !== before.players[index]?.handCount
			) {
				const panel = element.querySelector(`[data-player="${index}"]`);
				if (panel) pulse(panel);
			}
		}
		if (before.cities && game.cities) {
			const track = element.querySelector(".barbarian-track");
			const ship = element.querySelector(".barbarian-ship");
			if (
				track &&
				ship &&
				(before.cities.barbarians !== game.cities.barbarians ||
					before.cities.attacks !== game.cities.attacks)
			) {
				const distance = track.getBoundingClientRect().width / 8;
				const position = (step: number) =>
					`translateX(calc(-50% + ${(step - (game.cities?.barbarians ?? 0)) * distance}px))`;
				const attacked = game.cities.attacks > before.cities.attacks;
				animate(
					ship,
					attacked
						? [
								{ transform: position(before.cities.barbarians), opacity: 1 },
								{ transform: position(7), opacity: 1, offset: 0.65 },
								{ transform: position(7), opacity: 0, offset: 0.78 },
								{ transform: "translateX(-50%)", opacity: 0, offset: 0.8 },
								{ transform: "translateX(-50%)", opacity: 1 },
							]
						: [
								{ transform: position(before.cities.barbarians) },
								{ transform: "translateX(-50%)" },
							],
					(attacked ? 400 : 200) + 200 * strength,
				);
				if (attacked) {
					const battle = element.querySelector(".battle-strength");
					if (battle) pulse(battle);
				}
			}
		}
		if (game.winner !== null && before.winner === null) {
			const winner = element.querySelector(".winner-panel");
			if (winner)
				animate(
					winner,
					[
						{
							opacity: 0,
							transform: `translateY(${20 * strength}px) scale(0.94)`,
						},
						{ opacity: 1, transform: "none" },
					],
					300 + 300 * strength,
				);
		}
	}, [game, root, strength]);
}
