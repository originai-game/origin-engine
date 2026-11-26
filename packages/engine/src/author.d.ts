import type { og } from "../types/index.d.ts";
import type { FileInputs, ProjectInput, ModuleInput } from "../types/author-inputs.d.ts";

/** Pure bootstrap data, checked before the game's entry or content is evaluated. */
export declare function defineProject(value: ProjectInput): ProjectInput;
/** A source namespace, separate from output packs and streaming groups. */
export declare function defineModule(value: ModuleInput): ModuleInput;

/** Project declarations augment this interface with contextual file inputs. */
export interface ContextualFileInputs {}

/** Project-generated content entry identities, discovered without evaluation.
 * Values contain the root-relative source file, factory and primary kind.
 * Derived subresources and checked reference edges belong to the cook graph. */
export interface ContentIndex {}

/** A cook descriptor, never a runtime asset handle. */
export interface File<K extends string, T> {
  readonly $ogFile: K;
  readonly value: T;
}

/** One schema-derived factory per content kind; hyphenated kinds use brackets. */
export declare const file: {
  readonly [K in keyof (FileInputs & ContextualFileInputs)]:
    (value: (FileInputs & ContextualFileInputs)[K]) => File<K, (FileInputs & ContextualFileInputs)[K]>;
};

/** An immutable identity. Project checking validates its target and module access. */
export declare function ref<const I extends string>(id: I): { readonly $ogRef: I };

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
 * For indexed content, the input root is the entry's declared source module;
 * paths must be canonical and start with assets/ or sources/, even in helpers.
 * Call these methods directly; aliases, computed paths and re-exports are refused. */
export declare const input: {
  /** An independent mutable copy of the captured bytes on each call. */
  readonly bytes: (path: string) => Uint8Array;
  /** Strict UTF-8, preserving the input's BOM and line endings. */
  readonly text: (path: string) => string;
};
