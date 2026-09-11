import { invoke } from "@tauri-apps/api/core";
import {
	availableMonitors,
	currentMonitor,
	getCurrentWindow,
	type Monitor,
	PhysicalPosition,
} from "@tauri-apps/api/window";
import { Monitor as MonitorIcon, Settings, Volume2, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type { Preferences } from "./bindings";
import { useText } from "./locale";
import "./preferences.css";

export function PreferencesPanel({
	initial,
	close,
}: {
	initial: Preferences;
	close: () => void;
}) {
	const t = useText();
	const dialog = useRef<HTMLDialogElement>(null);
	const [value, setValue] = useState(initial);
	const [monitors, setMonitors] = useState<Monitor[]>([]);
	const [monitor, setMonitor] = useState("");
	const [originalMonitor, setOriginalMonitor] = useState("");
	const [fullscreen, setFullscreen] = useState(false);
	const [busy, setBusy] = useState(false);
	const [error, setError] = useState<string | null>(null);
	useEffect(() => {
		dialog.current?.showModal();
		let active = true;
		Promise.all([
			availableMonitors(),
			currentMonitor(),
			getCurrentWindow().isFullscreen(),
		])
			.then(([displays, current, full]) => {
				if (!active) return;
				setMonitors(displays);
				setFullscreen(full);
				const index = displays.findIndex(
					(display) =>
						display.position.x === current?.position.x &&
						display.position.y === current?.position.y,
				);
				setMonitor(String(index));
				setOriginalMonitor(String(index));
			})
			.catch((cause) => {
				if (active) setError(String(cause));
			});
		return () => {
			active = false;
		};
	}, []);
	const save = async () => {
		setBusy(true);
		setError(null);
		try {
			const window = getCurrentWindow();
			if (monitor !== originalMonitor) {
				const display = monitors[Number(monitor)];
				if (display) {
					await window.setFullscreen(false);
					await window.unmaximize();
					const size = await window.outerSize();
					await window.setPosition(
						new PhysicalPosition(
							display.position.x +
								Math.max(0, (display.size.width - size.width) / 2),
							display.position.y +
								Math.max(0, (display.size.height - size.height) / 2),
						),
					);
				}
			}
			await window.setFullscreen(fullscreen);
			await invoke("preferences", { preferences: value });
			close();
		} catch (cause) {
			setError(String(cause));
		} finally {
			setBusy(false);
		}
	};
	return (
		<dialog ref={dialog} className="preferences" onClose={close}>
			<form
				onSubmit={(event) => {
					event.preventDefault();
					void save();
				}}
			>
				<header>
					<h2>
						<Settings size={21} />
						{t("设置")}
					</h2>
					<button
						type="button"
						className="icon-button"
						aria-label={t("关闭设置")}
						onClick={close}
					>
						<X size={19} />
					</button>
				</header>
				{error && (
					<p role="alert" className="settings-error">
						{error}
					</p>
				)}
				<section>
					<h3>
						<Volume2 size={16} />
						{t("声音")}
					</h3>
					{(
						[
							{ key: "master", label: t("主音量") },
							{ key: "music", label: t("背景音乐") },
							{ key: "effects", label: t("游戏音效") },
							{ key: "ambience", label: t("环境声音") },
						] as const
					).map(({ key, label }) => (
						<label className="range-setting" key={key}>
							<span>
								{label}
								<output>{Math.round(value[key] * 100)}%</output>
							</span>
							<input
								type="range"
								min={0}
								max={1}
								step={0.01}
								value={value[key]}
								onChange={(event) =>
									setValue({ ...value, [key]: Number(event.target.value) })
								}
							/>
						</label>
					))}
				</section>
				<section>
					<h3>
						<MonitorIcon size={16} />
						{t("显示")}
					</h3>
					<div className="form-row">
						<label>
							{t("显示模式")}
							<select
								value={String(fullscreen)}
								onChange={(event) =>
									setFullscreen(event.target.value === "true")
								}
							>
								<option value="false">{t("窗口")}</option>
								<option value="true">{t("全屏")}</option>
							</select>
						</label>
						<label>
							{t("显示器")}
							<select
								value={monitor}
								onChange={(event) => setMonitor(event.target.value)}
							>
								{monitors.map((display, index) => (
									<option
										key={`${display.position.x}:${display.position.y}`}
										value={String(index)}
									>
										{display.name || t("显示器 {0}", index + 1)} ·{" "}
										{display.size.width} × {display.size.height}
									</option>
								))}
							</select>
						</label>
					</div>
					<div className="form-row">
						<label>
							{t("界面缩放")}
							<select
								value={value.scale}
								onChange={(event) =>
									setValue({ ...value, scale: Number(event.target.value) })
								}
							>
								{[0.75, 0.9, 1, 1.1, 1.25, 1.5].map((scale) => (
									<option key={scale} value={scale}>
										{Math.round(scale * 100)}%
									</option>
								))}
							</select>
						</label>
						<label>
							{t("动画强度")}
							<select
								value={value.animation}
								onChange={(event) =>
									setValue({ ...value, animation: Number(event.target.value) })
								}
							>
								<option value={0}>{t("关闭")}</option>
								<option value={0.5}>{t("轻柔")}</option>
								<option value={1}>{t("标准")}</option>
							</select>
						</label>
					</div>
					<label>
						{t("语言")}
						<select
							value={value.language}
							onChange={(event) =>
								setValue({
									...value,
									language: event.target.value === "en" ? "en" : "zh-CN",
								})
							}
						>
							<option value="zh-CN">{t("简体中文")}</option>
							<option value="en">English</option>
						</select>
					</label>
					<p className="muted">{t("F11 切换全屏。窗口大小与位置自动保存。")}</p>
				</section>
				<footer>
					<button type="button" className="quiet" onClick={close}>
						{t("取消")}
					</button>
					<button type="submit" className="primary" disabled={busy}>
						{t("应用设置")}
					</button>
				</footer>
			</form>
		</dialog>
	);
}
