// Audio on the native host (crates/og-audio/README.md §Host contract): one
// sink per engine over the addon's og_node_audio_* members. `null` drains
// the command stream and renders nothing (headless runs stay deterministic
// and their queue empty); `capture` renders it up to the world's tick for a
// WAV; `device` plays it on the default output (an addon built with the
// `native-audio` feature, cpal). Pump after every advance or frame.

import { native } from "./index.ts";
import type { Result } from "../runtime.ts";

/** The addon's audio members (crates/og-host-node/src/audio.rs). */
export interface NativeAudio {
  og_node_audio_open(engine: number, mode: string, sampleRate?: number): string;
  og_node_audio_pump(engine: number): string;
  og_node_audio_stats(engine: number): string;
  og_node_audio_wav(engine: number): Uint8Array;
  og_node_audio_close(engine: number): boolean;
  og_node_audio_insert_qoa(engine: number, id: string, qoa: Uint8Array): string;
}

export type AudioMode = "null" | "capture" | "device";

/** What a sink has done (fields present depend on the mode). */
export interface AudioStats {
  readonly mode: AudioMode;
  readonly sampleRate: number;
  /** Interleaved stereo frames rendered (0 for null). */
  readonly frames: number;
  /** Commands consumed. */
  readonly commands: number;
  /** Voices sounding now. */
  readonly voices: number;
  readonly voicesPeak: number;
  /** Packets a slow sink lost (each forces a resync). */
  readonly dropped: number;
  /** FNV-1a of every packet received, hex (null and capture). */
  readonly receivedHash?: string;
  readonly packets?: number;
  readonly resyncs?: number;
  readonly device?: string;
}

export interface AudioSink {
  readonly mode: AudioMode;
  /** Drain the command stream (and render, for capture) up to the world's tick. */
  pump(): AudioStats;
  stats(): AudioStats;
  /** The captured audio as a 16-bit stereo WAV (empty unless capture). */
  wav(): Uint8Array;
  close(): void;
}

function audio(): NativeAudio {
  return native() as unknown as NativeAudio;
}

function unwrap<T>(text: string): Result<T> {
  return JSON.parse(text) as Result<T>;
}

/** Open the engine's audio sink. `sampleRate` applies to capture (default 48000). */
export function openAudio(engine: number, mode: AudioMode = "null", sampleRate?: number): Result<AudioSink> {
  const a = audio();
  const opened = unwrap<string>(sampleRate === undefined ? a.og_node_audio_open(engine, mode) : a.og_node_audio_open(engine, mode, sampleRate));
  if (!opened.ok) return opened;
  const read = (text: string): AudioStats => {
    const r = unwrap<AudioStats>(text);
    if (!r.ok) throw new Error(r.reason);
    return r.value;
  };
  return {
    ok: true,
    value: {
      mode,
      pump: () => read(a.og_node_audio_pump(engine)),
      stats: () => read(a.og_node_audio_stats(engine)),
      wav: () => a.og_node_audio_wav(engine),
      close: () => void a.og_node_audio_close(engine),
    },
  };
}

/** Add a sound (a QOA file: a cooked sound's `qoa` section) to the engine's bank; its handle is what AudioSource.sound takes. */
export function insertSound(engine: number, id: string, qoa: Uint8Array): Result<number> {
  return unwrap<number>(audio().og_node_audio_insert_qoa(engine, id, qoa));
}
