import { Application, Container, Graphics, Text } from "pixi.js";
import type { Action, Board, GameView, Target, Terrain } from "./bindings";
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
	element.append(app.canvas);
	app.canvas.setAttribute("aria-label", "游戏棋盘");
	app.canvas.style.touchAction = "none";
	const world = new Container();
	const pieces = new Container();
	const hints = new Container();
	app.stage.addChild(world);
	let options = initialOptions;
	let act = initialAction;
	const board = initial.board;
	const numbers: Text[] = [];
	const pips: Graphics[] = [];
	for (const hex of board.hexes) {
		const points = hex.vertices.map((vertex) => coordinates(board, vertex));
		const polygon = points.flatMap(({ x, y }) => [x, y]);
		const tile = new Container();
		tile.addChild(
			new Graphics()
				.poly(points.flatMap(({ x, y }) => [x, y + 10]))
				.fill(0x667357),
		);
		tile.addChild(
			new Graphics()
				.poly(polygon)
				.fill(land[hex.terrain].color)
				.stroke({ color: 0xe7d7a6, width: 3 }),
		);
		const x = hex.x * 80;
		const y = hex.y * 66;
		tile.addChild(caption(land[hex.terrain].name, x, y + 35, 12));
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
		world.addChild(
			caption(
				harbor.resource ? `${names[harbor.resource]} 2:1` : "3:1",
				px,
				py + 27,
				12,
				0xe8f1db,
			),
		);
	}
	world.addChild(pieces, hints);
	const xs = board.vertices.map((point) => point.x * 80);
	const ys = board.vertices.map((point) => point.y * 66);
	const width = Math.max(...xs) - Math.min(...xs) + 170;
	const height = Math.max(...ys) - Math.min(...ys) + 170;
	let magnification = 1;
	let offset = { x: 0, y: 0 };
	let drag: {
		id: number;
		x: number;
		y: number;
		startX: number;
		startY: number;
	} | null = null;
	const events = new AbortController();
	const paint = () => app.render();
	const layout = () => {
		world.scale.set(
			Math.min(app.screen.width / width, app.screen.height / height) *
				magnification,
		);
		world.position.set(
			app.screen.width / 2 + offset.x,
			app.screen.height / 2 + offset.y,
		);
		paint();
	};
	const fit = () => {
		magnification = 1;
		offset = { x: 0, y: 0 };
		layout();
	};
	const zoom = (
		factor: number,
		x = app.screen.width / 2,
		y = app.screen.height / 2,
	) => {
		const before = world.scale.x;
		const next = Math.max(0.55, Math.min(3.5, magnification * factor));
		const ratio = next / magnification;
		magnification = next;
		offset = {
			x: x - ((x - world.x) / before) * before * ratio - app.screen.width / 2,
			y: y - ((y - world.y) / before) * before * ratio - app.screen.height / 2,
		};
		layout();
	};
	const local = (event: PointerEvent) => {
		const rect = app.canvas.getBoundingClientRect();
		return { x: event.clientX - rect.left, y: event.clientY - rect.top };
	};
	const hit = (x: number, y: number) => {
		const px = (x - world.x) / world.scale.x;
		const py = (y - world.y) / world.scale.y;
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
	app.canvas.addEventListener(
		"pointerdown",
		(event) => {
			if (event.button !== 0 && event.button !== 1) return;
			const point = local(event);
			drag = {
				id: event.pointerId,
				...point,
				startX: offset.x,
				startY: offset.y,
			};
			app.canvas.setPointerCapture(event.pointerId);
			app.canvas.style.cursor = "grabbing";
		},
		{ signal: events.signal },
	);
	app.canvas.addEventListener(
		"pointermove",
		(event) => {
			const point = local(event);
			if (drag?.id === event.pointerId) {
				offset = {
					x: drag.startX + point.x - drag.x,
					y: drag.startY + point.y - drag.y,
				};
				layout();
			} else {
				app.canvas.style.cursor = hit(point.x, point.y) ? "pointer" : "grab";
			}
		},
		{ signal: events.signal },
	);
	app.canvas.addEventListener(
		"pointerup",
		(event) => {
			const point = local(event);
			if (
				drag?.id === event.pointerId &&
				Math.hypot(point.x - drag.x, point.y - drag.y) < 5
			) {
				const option = hit(point.x, point.y);
				if (option) act(option.action);
			}
			drag = null;
			app.canvas.style.cursor = "grab";
		},
		{ signal: events.signal },
	);
	app.canvas.addEventListener(
		"pointercancel",
		() => {
			drag = null;
		},
		{ signal: events.signal },
	);
	app.canvas.addEventListener(
		"wheel",
		(event) => {
			event.preventDefault();
			const rect = app.canvas.getBoundingClientRect();
			zoom(
				Math.exp(-event.deltaY * 0.002),
				event.clientX - rect.left,
				event.clientY - rect.top,
			);
		},
		{ passive: false, signal: events.signal },
	);
	const resize = new ResizeObserver(() => {
		app.renderer.resolution = devicePixelRatio;
		app.renderer.resize(element.clientWidth, element.clientHeight);
		layout();
	});
	resize.observe(element);
	const update = (
		view: GameView,
		nextOptions: BoardOption[],
		onAction: (action: Action) => void,
	) => {
		options = nextOptions;
		act = onAction;
		for (const child of pieces.removeChildren())
			child.destroy({ children: true });
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
				body.circle((i - (knight.level - 1) / 2) * 5, -10, 1.7).fill(0xffefba);
			if (knight.active)
				body.poly([-5, -34, -9, -43, 2, -40, 4, -34]).fill(color);
			else {
				body.rotation = -0.7;
				body.position.set(4, 4);
			}
			figure.addChild(base, body);
			figure.position.set(point.x, point.y);
			pieces.addChild(figure);
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
			}
		}
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
	fit();
	update(initial, initialOptions, initialAction);
	return {
		update,
		fit,
		zoom,
		destroy() {
			events.abort();
			resize.disconnect();
			app.destroy(true, { children: true });
		},
	};
}

export type Scene = Awaited<ReturnType<typeof createScene>>;
