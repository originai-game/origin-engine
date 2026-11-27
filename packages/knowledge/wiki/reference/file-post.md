<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-post"></a>
### `file:post`

Checked `file["post"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
/** WGSL defining fn post(f: PostInput, p: Params) -> vec4<f32>. */
readonly "code": (string);
readonly "params"?: (ReadonlyArray<(InputDefinitions["Param"])> | (number extends ((InputDefinitions["Param"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface InputDefinitions {
readonly "Param": ((({
readonly "default"?: ((number) | (null));
readonly "kind": ("float") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("vec2") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("vec3") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "kind": ("vec4") & (string);
readonly "name": (string);
})) | (({
readonly "default"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("color") & (string);
readonly "name": (string);
})));
}
```
