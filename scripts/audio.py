#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["numpy"]
# ///

import wave
from pathlib import Path

import numpy as np

RATE = 24000
BEAT = 0.75
DURATION = BEAT * 64
rng = np.random.default_rng(741)
music = np.zeros((round(RATE * DURATION), 2), dtype=np.float64)


def add_note(
    note: int, start: float, duration: float, volume: float, pan: float
) -> None:
    frequency = 440 * 2 ** ((note - 69) / 12)
    t = np.arange(round(RATE * duration)) / RATE
    signal = np.zeros_like(t)
    for harmonic, amplitude in [(1, 1), (2, 0.28), (3, 0.13), (4, 0.055), (6, 0.018)]:
        signal += (
            amplitude
            * np.sin(2 * np.pi * frequency * harmonic * t)
            * np.exp(-t * (1.6 + harmonic * 0.4))
        )
    signal *= (1 - np.exp(-t * 350)) * np.minimum(1, (duration - t) * 20) * volume
    stereo = signal[:, None] * np.array(
        [np.cos(pan * np.pi / 2), np.sin(pan * np.pi / 2)]
    )
    for delay, attenuation in [(0, 1), (0.21, 0.16), (0.42, 0.08), (0.77, 0.04)]:
        indices = (round((start + delay) * RATE) + np.arange(len(t))) % len(music)
        music[indices] += stereo * attenuation


chords = [(48, 55, 60, 64), (53, 57, 60, 65), (45, 52, 57, 60), (43, 50, 55, 59)]
melody = [76, 74, 72, 67, 69, 72, 74, 76, 79, 76, 74, 72, 69, 67, 64, 67]
for bar in range(16):
    chord = chords[bar % 4]
    start = bar * 4 * BEAT
    add_note(chord[0], start, 3.5, 0.09, 0.5)
    for pulse in range(8):
        note = chord[[1, 2, 3, 2, 1, 3, 2, 3][pulse]] + (12 if bar >= 8 else 0)
        add_note(
            note, start + pulse * BEAT / 2, 2.2, 0.035, 0.25 if pulse % 2 else 0.75
        )
    add_note(melody[bar], start + BEAT * 1.5, 2.8, 0.048, 0.45)
    if bar % 4 == 3:
        add_note(melody[(bar + 1) % 16], start + BEAT * 3, 2, 0.028, 0.62)

length = RATE * 24
noise = rng.normal(0, 1, (length, 2))
frequencies = np.fft.rfftfreq(length, 1 / RATE)
spectrum = np.fft.rfft(noise, axis=0)
weight = np.exp(-frequencies / 4000) / np.sqrt(np.maximum(frequencies, 40))
ambient = np.fft.irfft(spectrum * weight[:, None], n=length, axis=0)
ambient /= np.max(np.abs(ambient))
t = np.arange(length) / RATE
swell = 0.15 + 0.14 * (1 + np.sin(2 * np.pi * t / 8))
ambient *= swell[:, None] * 0.16
for start in [1.7, 6.3, 13.6, 19.2]:
    chirp_time = np.arange(round(RATE * 0.45)) / RATE
    phase = 2 * np.pi * (2100 * chirp_time + 260 * np.sin(chirp_time * 11) / 11)
    chirp = np.sin(phase) * np.sin(np.pi * chirp_time / 0.45) ** 3 * 0.009
    offset = round(start * RATE)
    ambient[offset : offset + len(chirp), 0] += chirp
    ambient[offset : offset + len(chirp), 1] += chirp * 0.7

output = Path(__file__).resolve().parents[1] / "public/audio"
output.mkdir(parents=True, exist_ok=True)
for name, samples in [("shore", ambient), ("islands", music)]:
    pcm = (np.clip(samples, -1, 1) * 32767).astype("<i2")
    with wave.open(str(output / f"{name}.wav"), "wb") as stream:
        stream.setnchannels(2)
        stream.setsampwidth(2)
        stream.setframerate(RATE)
        stream.writeframes(pcm.tobytes())
