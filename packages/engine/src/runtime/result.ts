// ── results ───────────────────────────────────────────────────────────────

/** A CSS `cursor: url()` image the engine read from its texture: row-major RGBA8 (sRGB, straight alpha) in base64, and the pixel that is the pointer's position. */
export interface CursorImage {
  width: number;
  height: number;
  rgba: string;
  hotspot: [number, number];
}

export type Result<T> = { readonly ok: true; readonly value: T } | { readonly ok: false; readonly reason: string };

export function parse<T>(text: string): T {
  return JSON.parse(text) as T;
}

export function refused<T>(reason: string): Result<T> {
  return { ok: false, reason };
}

export function json(value: unknown): string | undefined {
  return value === undefined ? undefined : JSON.stringify(value);
}

/** Bytes the engine sent as base64 text (a room message's, og.load.dataBytes'). */
export function fromBase64(text: string): Uint8Array {
  const bin = atob(text);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}
