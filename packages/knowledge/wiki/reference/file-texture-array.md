<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-texture-array"></a>
### `file:texture-array`

Checked `file["texture-array"](input)` declaration from `@origingame/engine/author`.

```ts
/** An ordered array of equally-sized external images. File resolution,
dimensions and encoder compatibility remain texture-array domain checks. */
export type Input = ({
readonly "kind"?: (InputDefinitions["TextureKind"]);
/** Explicit canonical paths relative to the owning module (e.g.
assets/tiles/grass.png), without sibling or asset-root fallback. */
readonly "layers": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface InputDefinitions {
readonly "TextureKind": ("color" | "normal" | "linear" | "single" | "pixel") & (string);
}
```
