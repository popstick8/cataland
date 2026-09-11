import atlas from "./art.json";

export type Illustration = keyof typeof atlas.frames;

export function Art({
	name,
	size = 40,
	className = "",
}: {
	name: Illustration;
	size?: number | string;
	className?: string;
}) {
	const frame = atlas.frames[name];
	return (
		<span
			aria-hidden="true"
			className={`illustration ${className}`}
			style={{
				display: "inline-block",
				flexShrink: 0,
				width: size,
				aspectRatio: `${frame.w} / ${frame.h}`,
				backgroundImage: 'url("/art/illustrations.webp")',
				backgroundSize: `${(atlas.width / frame.w) * 100}% ${(atlas.height / frame.h) * 100}%`,
				backgroundPosition: `${(frame.x / (atlas.width - frame.w)) * 100}% ${(frame.y / (atlas.height - frame.h)) * 100}%`,
			}}
		/>
	);
}
