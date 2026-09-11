import {
	Application,
	Assets,
	Container,
	Graphics,
	Rectangle,
	Sprite,
	type Spritesheet,
	Text,
} from "pixi.js";
import type { Action, Board, GameView, Target, Terrain } from "./bindings";
import { createCamera, type Point } from "./camera";
import type { Translate } from "./locale";
import { colors } from "./palette";

export type BoardOption = { target: Target; action: Action };

const land: Record<Terrain, { color: number; name: string }> = {
	forest: { color: 0x719a73, name: "森林" },
	hills: { color: 0xbb8563, name: "丘陵" },
	pasture: { color: 0xaec385, name: "牧场" },
	fields: { color: 0xd9bd71, name: "农田" },
	mountains: { color: 0x96aaa5, name: "山脉" },
	desert: { color: 0xd9c799, name: "沙漠" },
};

function coordinates(board: Board, vertex: number) {
	const point = board.vertices[vertex];
	if (!point) throw new Error("棋盘顶点不存在");
	return { x: point.x * 80, y: point.y * 66 };
}

function playerColor(view: GameView, player: number) {
	const seat = view.players[player];
	const color = seat && colors[seat.color];
	if (!color) throw new Error("棋盘玩家颜色无效");
	return color;
}

function caption(
	text: string,
	x: number,
	y: number,
	size: number,
	color = 0x364c41,
) {
	const label = new Text({
		text,
		style: {
			fontFamily: "system-ui, PingFang SC, Microsoft YaHei, sans-serif",
			fontSize: size,
			fill: color,
		},
	});
	label.anchor.set(0.5);
	label.position.set(x, y);
	return label;
}

export async function createScene(
	element: HTMLElement,
	initial: GameView,
	initialOptions: BoardOption[],
	initialAction: (action: Action) => void,
	initialText: Translate,
	initialStrength: number,
) {
	const app = new Application();
	await app.init({
		width: element.clientWidth,
		height: element.clientHeight,
		autoStart: false,
		antialias: true,
		autoDensity: true,
		resolution: devicePixelRatio,
		backgroundAlpha: 0,
	});
	const terrain = await Assets.load<Spritesheet>(
		new URL("/art/terrain.json", document.baseURI).href,
	);
	const surface = document.createElement("div");
	surface.className = "board-world";
	surface.append(app.canvas);
	element.append(surface);
	app.canvas.setAttribute("aria-label", initialText("游戏棋盘"));
	app.canvas.style.touchAction = "none";
	const world = new Container();
	const pieces = new Container();
	const hints = new Container();
	app.stage.addChild(world);
	let options = initialOptions;
	let act = initialAction;
	const board = initial.board;
	const numbers: Text[] = [];
	const labels: { label: Text; key: string; suffix: string }[] = [];
	const pips: Graphics[] = [];
	for (const [id, hex] of board.hexes.entries()) {
		const tile = new Container();
		const texture = terrain.textures[`${hex.terrain}-${id % 3}`];
		if (!texture) throw new Error(`缺少地形资源：${hex.terrain}`);
		const surface = new Sprite(texture);
		surface.anchor.set(0.5, 278 / 512);
		surface.position.set(hex.x * 80, hex.y * 66);
		surface.scale.set(80 / 224);
		tile.addChild(surface);
		const x = hex.x * 80;
		const y = hex.y * 66;
		const label = caption(initialText(land[hex.terrain].name), x, y + 35, 12);
		labels.push({ label, key: land[hex.terrain].name, suffix: "" });
		tile.addChild(label);
		if (hex.number > 0) {
			tile.addChild(
				new Graphics()
					.circle(x, y + 2, 23)
					.fill({ color: 0x3c5040, alpha: 0.15 })
					.circle(x, y, 22)
					.fill(0xfaf0cc)
					.stroke({ color: 0xfff9e0, width: 1.5 }),
			);
		}
		const number = caption(
			hex.number > 0 ? String(hex.number) : "",
			x,
			y - 3,
			24,
		);
		numbers.push(number);
		tile.addChild(number);
		const pip = new Graphics();
		pips.push(pip);
		tile.addChild(pip);
		world.addChild(tile);
	}
	for (const harbor of board.harbors) {
		const edge = board.edges[harbor.edge];
		if (!edge) throw new Error("港口对应的海岸不存在");
		const a = coordinates(board, edge.vertices[0]);
		const b = coordinates(board, edge.vertices[1]);
		const x = (a.x + b.x) / 2;
		const y = (a.y + b.y) / 2;
		const distance = Math.hypot(x, y);
		const px = x + (x / distance) * 48;
		const py = y + (y / distance) * 48;
		world.addChild(
			new Graphics()
				.moveTo(x, y)
				.lineTo(px, py)
				.stroke({ color: 0xc4dcce, width: 2, alpha: 0.6 }),
		);
		const boat = new Graphics()
			.poly([-17, 2, 17, 2, 10, 12, -9, 12])
			.fill(0xcba677)
			.moveTo(0, 1)
			.lineTo(0, -23)
			.stroke({ color: 0xf3e5ba, width: 2 })
			.poly([2, -23, 2, -1, 17, -1])
			.fill(0xfff1d0);
		boat.position.set(px, py);
		world.addChild(boat);
		const names = {
			wood: "木材",
			brick: "砖块",
			wool: "羊毛",
			grain: "小麦",
			ore: "矿石",
			cloth: "布匹",
			coin: "铸币",
			paper: "纸张",
		};
		const key = harbor.resource ? names[harbor.resource] : "3:1";
		const suffix = harbor.resource ? " 2:1" : "";
		const label = caption(initialText(key) + suffix, px, py + 27, 12, 0xe8f1db);
		labels.push({ label, key, suffix });
		world.addChild(label);
	}
	world.addChild(pieces, hints);
	const bounds = world.getLocalBounds();
	const width = Math.ceil(bounds.width) + 100;
	const height = Math.ceil(bounds.height) + 100;
	world.position.set(50 - bounds.minX, 50 - bounds.minY);
	app.renderer.resize(width, height);
	surface.style.width = `${width}px`;
	surface.style.height = `${height}px`;
	const paint = () => app.render();
	const hit = (x: number, y: number) => {
		const px = x - world.x;
		const py = y - world.y;
		return options.find(({ target }) => {
			if (target.type === "vertex") {
				const point = coordinates(board, target.id);
				return Math.hypot(px - point.x, py - point.y) < 16;
			}
			if (target.type === "hex") {
				const hex = board.hexes[target.id];
				if (!hex) return false;
				const dx = Math.abs(px / 80 - hex.x);
				const dy = Math.abs(py / 66 - hex.y);
				return dx < Math.sqrt(3) / 2 && dy + dx / Math.sqrt(3) < 1;
			}
			const edge = board.edges[target.id];
			if (!edge) return false;
			const a = coordinates(board, edge.vertices[0]);
			const b = coordinates(board, edge.vertices[1]);
			const dx = b.x - a.x;
			const dy = b.y - a.y;
			const t = Math.max(
				0.12,
				Math.min(
					0.88,
					((px - a.x) * dx + (py - a.y) * dy) / (dx * dx + dy * dy),
				),
			);
			return Math.hypot(px - a.x - t * dx, py - a.y - t * dy) < 13;
		});
	};
	const camera = createCamera(
		element,
		surface,
		width,
		height,
		initialStrength,
		({ x, y }) => {
			const option = hit(x, y);
			if (option) act(option.action);
		},
		({ x, y }) => Boolean(hit(x, y)),
		(scale) => {
			const resolution = Math.max(1, devicePixelRatio * scale);
			if (app.renderer.resolution === resolution) return;
			app.renderer.resolution = resolution;
			app.renderer.resize(width, height);
			paint();
		},
	);
	const motions = new Set<() => void>();
	let previous: GameView | undefined;
	let sequence = initial.events.at(-1)?.seq ?? 0;
	let pieceState = "";
	const animatePiece = (shape: Container, strength: number, from?: Point) => {
		if (strength === 0) return;
		const bounds = shape.getLocalBounds();
		const frame = new Rectangle(
			Math.floor(bounds.minX) - 2,
			Math.floor(bounds.minY) - 2,
			Math.ceil(bounds.maxX) - Math.floor(bounds.minX) + 4,
			Math.ceil(bounds.maxY) - Math.floor(bounds.minY) + 4,
		);
		const snapshot = app.renderer.extract.canvas({
			target: shape,
			frame,
			resolution: app.renderer.resolution,
			antialias: true,
		}) as HTMLCanvasElement;
		snapshot.className = "piece-motion";
		snapshot.ariaHidden = "true";
		snapshot.style.left = `${world.x + shape.x + frame.x}px`;
		snapshot.style.top = `${world.y + shape.y + frame.y}px`;
		snapshot.style.width = `${frame.width}px`;
		snapshot.style.height = `${frame.height}px`;
		surface.append(snapshot);
		shape.visible = false;
		const frames: Keyframe[] = from
			? [
					{
						transform: `translate(${from.x - shape.x}px, ${from.y - shape.y}px)`,
					},
					{ transform: "translate(0, 0)" },
				]
			: [
					{
						transform: `translateY(${-10 * strength}px) scale(${1 - 0.18 * strength})`,
						opacity: 0,
					},
					{ transform: "translateY(0) scale(1.025)", opacity: 1, offset: 0.75 },
					{ transform: "none", opacity: 1 },
				];
		const animation = snapshot.animate(frames, {
			duration: (from ? 220 : 150) + 130 * strength,
			easing: "cubic-bezier(0.2, 0.7, 0.2, 1)",
		});
		const cancel = () => {
			animation.onfinish = null;
			animation.cancel();
			snapshot.remove();
			if (!shape.destroyed) shape.visible = true;
			motions.delete(cancel);
		};
		motions.add(cancel);
		animation.onfinish = () => {
			cancel();
			paint();
		};
	};
	const update = (
		view: GameView,
		nextOptions: BoardOption[],
		onAction: (action: Action) => void,
		t: Translate,
		strength: number,
	) => {
		options = nextOptions;
		camera.configure(strength);
		app.canvas.setAttribute("aria-label", t("游戏棋盘"));
		for (const { label, key, suffix } of labels) label.text = t(key) + suffix;
		act = onAction;
		for (const child of hints.removeChildren())
			child.destroy({ children: true });
		view.board.hexes.forEach((hex, id) => {
			const number = numbers[id];
			const pip = pips[id];
			if (!number || !pip) return;
			number.text = hex.number > 0 ? String(hex.number) : "";
			number.style.fill =
				hex.number === 6 || hex.number === 8 ? 0x9f5238 : 0x374c3f;
			pip.clear();
			const count = hex.number > 0 ? 6 - Math.abs(7 - hex.number) : 0;
			for (let i = 0; i < count; i++)
				pip
					.circle(hex.x * 80 + (i - (count - 1) / 2) * 5, hex.y * 66 + 13, 1.5)
					.fill(0x89684e);
		});
		const recent = view.events.filter((event) => event.seq > sequence);
		sequence = Math.max(sequence, view.events.at(-1)?.seq ?? 0);
		const state = JSON.stringify([
			view.roads,
			view.buildings,
			view.cities?.knights,
			view.cities?.walls,
			view.cities?.metropolises,
			view.cities?.merchant,
			view.robber,
		]);
		if (strength === 0) for (const cancel of motions) cancel();
		if (state !== pieceState) {
			pieceState = state;
			for (const cancel of motions) cancel();
			for (const child of pieces.removeChildren())
				child.destroy({ children: true });
			view.roads.forEach((player, id) => {
				if (player === null) return;
				const edge = board.edges[id];
				if (!edge) throw new Error("道路对应的棋盘边不存在");
				const a = coordinates(board, edge.vertices[0]);
				const b = coordinates(board, edge.vertices[1]);
				const line = new Graphics()
					.moveTo(a.x, a.y + 3)
					.lineTo(b.x, b.y + 3)
					.stroke({ color: 0x3c4a36, alpha: 0.3, width: 12, cap: "round" })
					.moveTo(a.x, a.y)
					.lineTo(b.x, b.y)
					.stroke({ color: 0xf1e5c3, width: 11, cap: "round" })
					.moveTo(a.x, a.y)
					.lineTo(b.x, b.y)
					.stroke({ color: playerColor(view, player), width: 7, cap: "round" });
				pieces.addChild(line);
				if (previous && previous.roads[id] !== player)
					animatePiece(line, strength);
			});
			view.buildings.forEach((building, id) => {
				if (!building) return;
				const point = coordinates(board, id);
				const color = playerColor(view, building.player);
				const city = building.kind === "city";
				const shape = new Graphics()
					.ellipse(0, 5, city ? 24 : 17, 9)
					.fill({ color: 0x263e35, alpha: 0.24 });
				if (view.cities?.walls.includes(id)) {
					shape
						.poly([-24, 3, -8, -8, 26, 4, 10, 17])
						.fill(0x8d9890)
						.stroke({ color: 0xe9e1c7, width: 2 });
				}
				shape
					.poly([-12, -12, 1, -5, 1, 8, -12, 1])
					.fill(0xf7e8ba)
					.poly([1, -5, 13, -12, 13, 1, 1, 8])
					.fill(0xc6b990)
					.poly([-15, -12, -2, -26, 15, -16, 1, -5])
					.fill(color)
					.stroke({ color: 0xfff1d4, width: 1.4 });
				shape.rect(-7, -5, 4, 8).fill(0x53675c);
				if (city)
					shape
						.poly([7, -28, 20, -23, 20, 1, 7, 7])
						.fill(0xe8d6ad)
						.poly([4, -29, 13, -41, 23, -26, 14, -22])
						.fill(color)
						.stroke({ color: 0xfff1d4, width: 1.2 })
						.rect(12, -15, 4, 5)
						.fill(0x53675c);
				if (view.cities?.metropolises.includes(id)) {
					shape
						.poly([3, -45, 7, -39, 12, -49, 17, -39, 22, -45, 20, -33, 5, -33])
						.fill(0xdabd66)
						.stroke({ color: 0xffefb9, width: 1.5 });
				}
				shape.position.set(point.x, point.y);
				pieces.addChild(shape);
				if (
					previous &&
					(JSON.stringify(previous.buildings[id]) !==
						JSON.stringify(building) ||
						previous.cities?.walls.includes(id) !==
							view.cities?.walls.includes(id) ||
						previous.cities?.metropolises.includes(id) !==
							view.cities?.metropolises.includes(id))
				)
					animatePiece(shape, strength);
			});
			view.cities?.knights.forEach((knight, vertex) => {
				if (!knight) return;
				const point = coordinates(board, vertex);
				const color = playerColor(view, knight.player);
				const figure = new Container();
				const base = new Graphics()
					.ellipse(0, 5, 16, 7)
					.fill({ color: 0x263e35, alpha: 0.25 });
				base
					.ellipse(0, 1, 14, 6)
					.fill(color)
					.stroke({ color: 0xf3e6c4, width: 2 });
				const body = new Graphics()
					.poly([-10, -3, -8, -21, 0, -27, 8, -21, 10, -3])
					.fill(0xd5d9ce)
					.poly([-8, -19, 0, -15, 8, -19, 6, -7, 0, -2, -6, -7])
					.fill(color)
					.roundRect(-7, -34, 14, 13, 5)
					.fill(0xf1e9cd)
					.moveTo(-7, -27)
					.lineTo(7, -27)
					.stroke({ color: 0x495859, width: 3 });
				for (let i = 0; i < knight.level; i++)
					body
						.circle((i - (knight.level - 1) / 2) * 5, -10, 1.7)
						.fill(0xffefba);
				if (knight.active)
					body.poly([-5, -34, -9, -43, 2, -40, 4, -34]).fill(color);
				else {
					body.rotation = -0.7;
					body.position.set(4, 4);
				}
				figure.addChild(base, body);
				figure.position.set(point.x, point.y);
				pieces.addChild(figure);
				if (previous) {
					const move = recent.findLast(
						(event) =>
							event.kind === "knight" &&
							event.target?.type === "vertex" &&
							event.target.id === vertex &&
							event.player === knight.player &&
							event.origin?.type === "vertex",
					);
					if (move?.origin?.type === "vertex")
						animatePiece(figure, strength, coordinates(board, move.origin.id));
					else if (
						JSON.stringify(previous.cities?.knights[vertex]) !==
						JSON.stringify(knight)
					)
						animatePiece(figure, strength);
				}
			});
			if (view.cities?.merchant) {
				const merchant = view.cities.merchant;
				const hex = board.hexes[merchant.hex];
				if (hex) {
					const trader = new Graphics()
						.ellipse(0, 4, 13, 6)
						.fill({ color: 0x203c37, alpha: 0.22 })
						.roundRect(-9, -20, 18, 24, 5)
						.fill(playerColor(view, merchant.player))
						.circle(0, -24, 7)
						.fill(0xf1d7a6)
						.ellipse(0, -29, 13, 3)
						.fill(0xb08d46)
						.roundRect(-7, -36, 14, 7, 3)
						.fill(0xd4b46a)
						.roundRect(5, -11, 11, 13, 3)
						.fill(0xa9794f);
					trader.position.set(hex.x * 80 - 32, hex.y * 66 + 8);
					pieces.addChild(trader);
					if (
						previous &&
						JSON.stringify(previous.cities?.merchant) !==
							JSON.stringify(merchant)
					) {
						const old = previous.cities?.merchant;
						const from =
							old && old.hex !== merchant.hex
								? board.hexes[old.hex]
								: undefined;
						animatePiece(
							trader,
							strength,
							from ? { x: from.x * 80 - 32, y: from.y * 66 + 8 } : undefined,
						);
					}
				}
			}
			if (view.robber !== null) {
				const hex = board.hexes[view.robber];
				if (hex) {
					const thief = new Graphics()
						.ellipse(0, 3, 13, 6)
						.fill({ color: 0x203c37, alpha: 0.25 })
						.roundRect(-9, -19, 18, 22, 6)
						.fill(0x3a4b4c)
						.circle(0, -23, 8)
						.fill(0x293a3c)
						.moveTo(-7, -13)
						.lineTo(7, -13)
						.stroke({ color: 0x81918a, width: 2 });
					thief.position.set(hex.x * 80 + 33, hex.y * 66 + 6);
					pieces.addChild(thief);
					if (previous && previous.robber !== view.robber) {
						const from =
							previous.robber === null
								? undefined
								: board.hexes[previous.robber];
						animatePiece(
							thief,
							strength,
							from ? { x: from.x * 80 + 33, y: from.y * 66 + 6 } : undefined,
						);
					}
				}
			}
		}
		previous = view;
		for (const option of options) {
			const target = option.target;
			const hint = new Graphics();
			if (target.type === "vertex") {
				const point = coordinates(board, target.id);
				hint
					.circle(point.x, point.y, 11)
					.fill({ color: 0xfff6d6, alpha: 0.7 })
					.stroke({ color: 0xfffce7, width: 2.5 });
			} else if (target.type === "edge") {
				const edge = board.edges[target.id];
				if (!edge) continue;
				const a = coordinates(board, edge.vertices[0]);
				const b = coordinates(board, edge.vertices[1]);
				hint
					.moveTo(a.x * 0.86 + b.x * 0.14, a.y * 0.86 + b.y * 0.14)
					.lineTo(b.x * 0.86 + a.x * 0.14, b.y * 0.86 + a.y * 0.14)
					.stroke({ color: 0xfff8de, width: 9, cap: "round", alpha: 0.7 });
			} else {
				const hex = board.hexes[target.id];
				if (!hex) continue;
				hint
					.poly(
						hex.vertices.flatMap((id) => {
							const point = coordinates(board, id);
							return [point.x, point.y];
						}),
					)
					.fill({ color: 0xffffe7, alpha: 0.1 })
					.stroke({ color: 0xfff8d5, width: 3 });
			}
			hints.addChild(hint);
		}
		paint();
	};
	update(initial, initialOptions, initialAction, initialText, initialStrength);
	return {
		update,
		fit: camera.fit,
		zoom: camera.zoom,
		destroy() {
			for (const cancel of motions) cancel();
			camera.destroy();
			surface.remove();
			app.destroy(true, { children: true });
		},
	};
}

export type Scene = Awaited<ReturnType<typeof createScene>>;
