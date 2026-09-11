export type Point = { x: number; y: number };

export function createCamera(
	element: HTMLElement,
	surface: HTMLElement,
	width: number,
	height: number,
	strength: number,
	select: (point: Point) => void,
	hover: (point: Point) => boolean,
	render: (scale: number) => void,
) {
	let magnification = 1;
	let offset: Point = { x: 0, y: 0 };
	let matrix = new DOMMatrix();
	let motion: Animation | undefined;
	let drag: { id: number; start: Point; offset: Point; moved: boolean } | null =
		null;
	const events = new AbortController();
	const fitScale = () =>
		Math.min(element.clientWidth / width, element.clientHeight / height);
	const visual = () =>
		motion ? new DOMMatrix(getComputedStyle(surface).transform) : matrix;
	const stop = () => {
		const current = visual();
		if (motion) {
			motion.onfinish = null;
			motion.cancel();
			motion = undefined;
		}
		return current;
	};
	const layout = (smooth: boolean) => {
		const before = stop();
		const scale = fitScale() * magnification;
		matrix = new DOMMatrix([
			scale,
			0,
			0,
			scale,
			(element.clientWidth - width * scale) / 2 + offset.x,
			(element.clientHeight - height * scale) / 2 + offset.y,
		]);
		surface.style.transform = matrix.toString();
		if (smooth && strength > 0) {
			motion = surface.animate(
				[{ transform: before.toString() }, { transform: matrix.toString() }],
				{ duration: 190 * strength, easing: "cubic-bezier(0.2, 0.7, 0.2, 1)" },
			);
			motion.onfinish = () => {
				motion = undefined;
				render(scale);
			};
		} else {
			render(scale);
		}
	};
	const point = (x: number, y: number): Point => {
		const current = visual();
		return { x: (x - current.e) / current.a, y: (y - current.f) / current.d };
	};
	const local = (event: MouseEvent): Point => {
		const rect = element.getBoundingClientRect();
		return { x: event.clientX - rect.left, y: event.clientY - rect.top };
	};
	const zoom = (
		factor: number,
		x = element.clientWidth / 2,
		y = element.clientHeight / 2,
	) => {
		const anchor = point(x, y);
		magnification = Math.max(0.55, Math.min(3.5, magnification * factor));
		const scale = fitScale() * magnification;
		offset = {
			x: x - anchor.x * scale - (element.clientWidth - width * scale) / 2,
			y: y - anchor.y * scale - (element.clientHeight - height * scale) / 2,
		};
		layout(true);
	};
	const fit = () => {
		magnification = 1;
		offset = { x: 0, y: 0 };
		layout(true);
	};
	element.addEventListener(
		"pointerdown",
		(event) => {
			if (event.button !== 0 && event.button !== 1) return;
			matrix = stop();
			surface.style.transform = matrix.toString();
			magnification = matrix.a / fitScale();
			offset = {
				x: matrix.e - (element.clientWidth - width * matrix.a) / 2,
				y: matrix.f - (element.clientHeight - height * matrix.d) / 2,
			};
			drag = { id: event.pointerId, start: local(event), offset, moved: false };
			element.setPointerCapture(event.pointerId);
			element.style.cursor = "grabbing";
		},
		{ signal: events.signal },
	);
	element.addEventListener(
		"pointermove",
		(event) => {
			const current = local(event);
			if (drag?.id === event.pointerId) {
				const dx = current.x - drag.start.x;
				const dy = current.y - drag.start.y;
				drag.moved ||= Math.hypot(dx, dy) >= 5;
				if (drag.moved) {
					offset = { x: drag.offset.x + dx, y: drag.offset.y + dy };
					layout(false);
				}
			} else {
				element.style.cursor = hover(point(current.x, current.y))
					? "pointer"
					: "grab";
			}
		},
		{ signal: events.signal },
	);
	element.addEventListener(
		"pointerup",
		(event) => {
			if (drag?.id !== event.pointerId) return;
			const current = local(event);
			if (!drag.moved && event.button === 0)
				select(point(current.x, current.y));
			drag = null;
			element.releasePointerCapture(event.pointerId);
			element.style.cursor = "grab";
			render(matrix.a);
		},
		{ signal: events.signal },
	);
	element.addEventListener(
		"pointercancel",
		() => {
			drag = null;
			element.style.cursor = "grab";
		},
		{ signal: events.signal },
	);
	element.addEventListener(
		"wheel",
		(event) => {
			event.preventDefault();
			const current = local(event);
			const delta =
				event.deltaY *
				(event.deltaMode === 1
					? 16
					: event.deltaMode === 2
						? element.clientHeight
						: 1);
			zoom(Math.exp(-delta * 0.002), current.x, current.y);
		},
		{ passive: false, signal: events.signal },
	);
	const resize = new ResizeObserver(() => layout(false));
	resize.observe(element);
	layout(false);
	return {
		zoom,
		fit,
		configure(value: number) {
			if (strength === value) return;
			strength = value;
			if (strength === 0) layout(false);
		},
		destroy() {
			stop();
			events.abort();
			resize.disconnect();
		},
	};
}
