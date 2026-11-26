import type { og } from "../types/index.d.ts";

/** The runtime API's pure mechanisms, without a world or host handles. */
export declare const procgen: typeof og.procgen;

/** Inputs are declared in generate.input or --input; scripts cannot open paths. */
export interface GeneratorContext<P = Record<string, unknown>> {
  readonly params: P;
  readonly seed: number;
  readonly inputs: Readonly<Record<string, readonly number[]>>;
}

/** Row-major heights in metres, from -z to +z and -x to +x. */
export interface HeightfieldSource {
  readonly columns: number;
  readonly rows: number;
  readonly heights: Float32Array | readonly number[];
}
