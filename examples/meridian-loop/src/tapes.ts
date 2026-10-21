// The tapes' chunks (components.ts's TapeChunk): spawning them, reading and
// writing a whole tape as text (once a lap), and a system's view of their
// bytes (every tick).

import type { Query, World } from "@origingame/engine";
import { CHUNK, CHUNKS, TapeChunk, TAPES } from "./components.ts";
import { entity, must } from "./world.ts";

type Which = keyof typeof TAPES;

export function spawnTapes(world: World): void {
  for (const which of Object.keys(TAPES) as Which[]) {
    for (let i = 0; i < CHUNKS[which]; i++) {
      const e = must(world.spawn({ name: `tape-${which}-${i}` }));
      must(world.insert(e, TapeChunk, { tape: TAPES[which], index: i }));
    }
  }
}

/** A tape's first `bytes` bytes as text. */
export function readTape(world: World, which: Which, bytes: number): string {
  let out = "";
  for (let i = 0; i * CHUNK < bytes; i++) out += must(world.get(entity(world, `tape-${which}-${i}`), TapeChunk)).bytes;
  return out.slice(0, bytes);
}

/** A tape's text, chunk by chunk (what is past its end stays as it was). */
export function writeTape(world: World, which: Which, text: string): void {
  for (let i = 0; i * CHUNK < text.length && i < CHUNKS[which]; i++) must(world.set(entity(world, `tape-${which}-${i}`), TapeChunk, { bytes: text.slice(i * CHUNK, (i + 1) * CHUNK) }));
}

/** The chunks' bytes in a system: the column, and the offset of byte `at` of a tape (or -1 past its last chunk). */
export interface TapeView {
  bytes: Uint8Array;
  offset(which: Which, at: number): number;
}

/** From the query of a system whose components every chunk has (TapeChunk, og.Transform). */
export function tapeView(q: Query): TapeView | null {
  for (const a of q.archetypes()) {
    const col = a.column(TapeChunk);
    const rows = new Map<number, number>();
    for (let r = 0; r < a.count; r++) rows.set(col.tape[r]! * 64 + col.index[r]!, r);
    return {
      bytes: col.bytes,
      offset(which, at) {
        const row = rows.get(TAPES[which] * 64 + Math.floor(at / CHUNK));
        return row === undefined ? -1 : row * CHUNK + (at % CHUNK);
      },
    };
  }
  return null;
}
