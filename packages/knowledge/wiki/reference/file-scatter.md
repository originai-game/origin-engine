<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-scatter"></a>
### `file:scatter`

Checked `file["scatter"](input)` declaration from `@origingame/engine/author`.

```ts
/** Ordered instance transforms. Array counts, nonzero quaternions and
terrain surface/holes are checked when the scatter is expanded. */
export type Input = ({
/** Rotate up to the projected normal before the authored quaternion. */
readonly "align"?: ((boolean) | (null));
readonly "positions": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "project"?: (((InputDefinitions["ContentRef"])) | ((null)));
/** One xyzw quaternion per position; normalized by domain expansion. */
readonly "quaternions"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "scales"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "target": (InputDefinitions["ScatterTarget"]);
});
export interface InputDefinitions {
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "ScatterTarget": ((({
readonly "asset": (InputDefinitions["ContentRef"]);
readonly "kind": ("model") & (string);
})) | (({
readonly "asset": (InputDefinitions["ContentRef"]);
readonly "kind": ("prefab") & (string);
})));
}
```
