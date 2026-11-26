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

/** Tile ids, not packed cells: -1 is empty. Names reference authored layers. */
export interface TilemapSource {
  readonly width: number;
  readonly height: number;
  readonly layers: readonly {
    readonly name: string;
    readonly tiles: Int32Array | readonly number[];
  }[];
}

/** Finite interleaved float PCM; the normal sound cook owns clamping/codecs. */
export interface SoundSource {
  readonly rate: number;
  readonly channels: 1 | 2;
  readonly samples: Float32Array | readonly number[];
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

/** Reflected fields, not runtime component layouts or asset handles. */
export type SceneFields = Readonly<Record<string,
  number | boolean | string | readonly (number | boolean | string)[]>>;

export interface SceneEntitySource {
  readonly name: string;
  readonly model?: string;
  readonly scene?: string;
  readonly prefab?: string;
  readonly static?: boolean;
  readonly components?: Readonly<Record<string, SceneFields>>;
  readonly children?: readonly SceneEntitySource[];
}

/** Appended to authored declarations, then checked by the normal scene cook. */
export interface SceneSource {
  readonly entities: readonly SceneEntitySource[];
  readonly resources?: Readonly<Record<string, SceneFields>>;
}
