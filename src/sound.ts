import { useEffect, useRef } from "react";
import type { Preferences, RoomView } from "./bindings";

class Sound {
	readonly context = new AudioContext();
	readonly master = this.context.createGain();
	readonly music = this.context.createGain();
	readonly ambience = this.context.createGain();
	readonly effects = this.context.createGain();
	readonly noise = this.context.createBuffer(
		1,
		this.context.sampleRate,
		this.context.sampleRate,
	);
	disposed = false;

	constructor(preferences: Preferences) {
		this.master.connect(this.context.destination);
		for (const channel of [this.music, this.ambience, this.effects])
			channel.connect(this.master);
		const noise = this.noise.getChannelData(0);
		for (let i = 0; i < noise.length; i++) noise[i] = Math.random() * 2 - 1;
		this.configure(preferences);
	}

	configure(preferences: Preferences) {
		const now = this.context.currentTime;
		this.master.gain.setTargetAtTime(preferences.master, now, 0.03);
		this.music.gain.setTargetAtTime(preferences.music, now, 0.03);
		this.ambience.gain.setTargetAtTime(preferences.ambience, now, 0.03);
		this.effects.gain.setTargetAtTime(preferences.effects, now, 0.03);
	}

	async prepare() {
		await Promise.all(
			(
				[
					["islands", this.music],
					["shore", this.ambience],
				] as const
			).map(async ([name, channel]) => {
				const response = await fetch(`/audio/${name}.wav`);
				if (!response.ok) throw new Error(`无法读取声音资源：${name}`);
				const buffer = await this.context.decodeAudioData(
					await response.arrayBuffer(),
				);
				if (this.disposed) return;
				const source = this.context.createBufferSource();
				source.buffer = buffer;
				source.loop = true;
				source.connect(channel);
				source.start();
			}),
		);
	}

	tone(
		frequency: number,
		time: number,
		duration: number,
		volume: number,
		end = frequency,
	) {
		const source = this.context.createOscillator();
		const envelope = this.context.createGain();
		source.frequency.setValueAtTime(frequency, time);
		source.frequency.exponentialRampToValueAtTime(end, time + duration);
		envelope.gain.setValueAtTime(0, time);
		envelope.gain.linearRampToValueAtTime(volume, time + 0.004);
		envelope.gain.exponentialRampToValueAtTime(0.0001, time + duration);
		source.connect(envelope).connect(this.effects);
		source.start(time);
		source.stop(time + duration);
		source.onended = () => {
			source.disconnect();
			envelope.disconnect();
		};
	}

	brush(frequency: number, time: number, duration: number, volume: number) {
		const source = this.context.createBufferSource();
		const filter = this.context.createBiquadFilter();
		const envelope = this.context.createGain();
		source.buffer = this.noise;
		filter.type = "bandpass";
		filter.frequency.value = frequency;
		filter.Q.value = 0.7;
		envelope.gain.setValueAtTime(volume, time);
		envelope.gain.exponentialRampToValueAtTime(0.0001, time + duration);
		source.connect(filter).connect(envelope).connect(this.effects);
		source.start(time);
		source.stop(time + duration);
		source.onended = () => {
			source.disconnect();
			filter.disconnect();
			envelope.disconnect();
		};
	}

	play(kind: string, delay = 0) {
		const time = this.context.currentTime + delay;
		switch (kind) {
			case "roll":
				for (let i = 0; i < 6; i++)
					this.brush(850 + i * 140, time + i * i * 0.012, 0.04, 0.22);
				break;
			case "build":
			case "road":
			case "wall":
				this.tone(250, time, 0.13, 0.17, 110);
				this.brush(650, time, 0.07, 0.12);
				break;
			case "knight":
				this.tone(720, time, 0.25, 0.055);
				this.tone(1145, time, 0.16, 0.035);
				break;
			case "robber":
			case "pillage":
			case "barbarians":
				this.tone(125, time, 0.48, 0.12, 82);
				this.brush(220, time, 0.3, 0.13);
				break;
			case "card":
			case "progress":
				this.brush(2300, time, 0.11, 0.09);
				break;
			case "production":
			case "trade":
			case "merchant":
				this.tone(660, time, 0.16, 0.04);
				this.tone(880, time + 0.065, 0.26, 0.035);
				break;
			case "award":
			case "defense":
			case "victory":
			case "win":
				for (const [index, note] of [523.25, 659.25, 783.99, 1046.5].entries())
					this.tone(note, time + index * 0.1, 0.6, 0.045);
				break;
			case "click":
				this.tone(520, time, 0.045, 0.025, 420);
				break;
		}
	}

	destroy() {
		this.disposed = true;
		void this.context.close().catch(console.error);
	}
}

export function useSound(
	preferences: Preferences | undefined,
	room: RoomView | null | undefined,
	report: (cause: unknown) => void,
) {
	const sound = useRef<Sound | null>(null);
	const latest = useRef(preferences);
	const cursor = useRef({ room: "", seq: 0 });
	useEffect(() => {
		latest.current = preferences;
		if (preferences) sound.current?.configure(preferences);
	}, [preferences]);
	useEffect(() => {
		const activate = (event: Event) => {
			if (!sound.current && latest.current) {
				sound.current = new Sound(latest.current);
				void sound.current.prepare().catch(report);
			}
			if (sound.current?.context.state === "suspended")
				void sound.current.context.resume().catch(report);
			if (
				event.target instanceof Element &&
				event.target.closest("button:not(:disabled)")
			)
				sound.current?.play("click");
		};
		document.addEventListener("pointerdown", activate);
		document.addEventListener("keydown", activate);
		return () => {
			document.removeEventListener("pointerdown", activate);
			document.removeEventListener("keydown", activate);
			sound.current?.destroy();
			sound.current = null;
		};
	}, [report]);
	useEffect(() => {
		const events = room?.game?.events ?? [];
		if (cursor.current.room === room?.id) {
			const kinds = new Set(
				events
					.filter((event) => event.seq > cursor.current.seq)
					.map((event) => event.kind),
			);
			let delay = 0;
			for (const kind of kinds) {
				sound.current?.play(kind, delay);
				delay += 0.035;
			}
		}
		cursor.current = { room: room?.id ?? "", seq: events.at(-1)?.seq ?? 0 };
	}, [room]);
}
