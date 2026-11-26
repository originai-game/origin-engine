import type { og } from "../types/index.d.ts";

/** Pure cook mechanisms, without a world or runtime handles. */
export declare const procgen: typeof og.procgen;

/** Paths are string literals relative to the evaluator's input root, not the
 * importing helper. Cook captures and hashes them before entering the realm.
 * Call these methods directly; aliases, computed paths and re-exports are refused. */
export declare const input: {
  /** An independent mutable copy of the captured bytes on each call. */
  readonly bytes: (path: string) => Uint8Array;
  /** Strict UTF-8, preserving the input's BOM and line endings. */
  readonly text: (path: string) => string;
};
