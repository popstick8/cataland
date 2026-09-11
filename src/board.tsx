import { Focus, Minus, Plus } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type { Action, GameView } from "./bindings";
import type { BoardOption, Scene } from "./scene";

export function BoardCanvas({
	game,
	options,
	act,
}: {
	game: GameView;
	options: BoardOption[];
	act: (action: Action) => void;
}) {
	const element = useRef<HTMLDivElement>(null);
	const scene = useRef<Scene | null>(null);
	const current = useRef({ game, options, act });
	const [error, setError] = useState<string | null>(null);
	useEffect(() => {
		current.current = { game, options, act };
		scene.current?.update(game, options, act);
	}, [game, options, act]);
	useEffect(() => {
		const root = element.current;
		if (!root) return;
		let disposed = false;
		let instance: Scene | undefined;
		const initial = current.current;
		import("./scene")
			.then(({ createScene }) =>
				disposed
					? null
					: createScene(root, initial.game, initial.options, initial.act),
			)
			.then((created) => {
				if (!created) return;
				if (disposed) {
					created.destroy();
					return;
				}
				instance = created;
				scene.current = created;
				const latest = current.current;
				created.update(latest.game, latest.options, latest.act);
			})
			.catch((cause) => {
				if (!disposed) setError(String(cause));
			});
		return () => {
			disposed = true;
			instance?.destroy();
			if (scene.current === instance) scene.current = null;
		};
	}, []);
	return (
		<div className="board-shell">
			<div className="board-canvas" ref={element} />
			{error && (
				<div className="board-error" role="alert">
					棋盘无法绘制：{error}
				</div>
			)}
			<div className="board-navigation">
				<button
					type="button"
					className="icon-button"
					aria-label="缩小棋盘"
					onClick={() => scene.current?.zoom(0.8)}
				>
					<Minus size={18} />
				</button>
				<button
					type="button"
					className="icon-button"
					aria-label="查看完整棋盘"
					onClick={() => scene.current?.fit()}
				>
					<Focus size={18} />
				</button>
				<button
					type="button"
					className="icon-button"
					aria-label="放大棋盘"
					onClick={() => scene.current?.zoom(1.25)}
				>
					<Plus size={18} />
				</button>
			</div>
		</div>
	);
}
