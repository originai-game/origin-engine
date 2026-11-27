import type { og } from "../types/index.d.ts";

/** The runtime API's pure mechanisms, without a world or host handles. */
export declare const procgen: typeof og.procgen;

/** Body declarations (without type/version/---) of particles, effect or vfx.
 * Cook validates the result with that file type's ordinary parser and links. */
export interface EffectSource {
  readonly text: string;
}

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

/** Static geometry attributes; materials and placements stay in the model file. */
export interface MeshSource {
  readonly positions: Float32Array | readonly number[];
  readonly normals?: Float32Array | readonly number[] | null;
  readonly uvs?: Float32Array | readonly number[] | null;
  readonly colors?: Float32Array | readonly number[] | null;
  readonly tangents?: Float32Array | readonly number[] | null;
  readonly indices?: Uint32Array | readonly number[] | null;
}
