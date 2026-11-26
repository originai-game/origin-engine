import type { og } from "../types/index.d.ts";

/** Pure cook mechanisms, without a world or runtime handles. */
export declare const procgen: typeof og.procgen;

/** A cook-local stream using the engine's Rust RNG, not simulation state. */
export interface Rng {
  /** The explicit or fork-derived u32 seed. */
  readonly seed: number;
  /** Uniform in [0, 1); advances only this stream. */
  readonly next: () => number;
  /** A fresh stream derived from seed and UTF-16 label; never advances the parent. */
  readonly fork: (label: string) => Rng;
}

/** A new independent stream. Seed is required, an integer in [0, 4294967295]. */
export declare function rng(seed: number): Rng;

/** Paths are string literals relative to the evaluator's input root, not the
 * importing helper. Cook captures and hashes them before entering the realm.
 * Call these methods directly; aliases, computed paths and re-exports are refused. */
export declare const input: {
  /** An independent mutable copy of the captured bytes on each call. */
  readonly bytes: (path: string) => Uint8Array;
  /** Strict UTF-8, preserving the input's BOM and line endings. */
  readonly text: (path: string) => string;
};
