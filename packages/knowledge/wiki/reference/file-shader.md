<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-shader"></a>
### `file:shader`

Checked `file["shader"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
/** WGSL hooks or vertex/fragment entries; use input.text for external WGSL. */
readonly "code": (string);
readonly "mode"?: (((InputDefinitions["Mode"])) | ((null)));
/** Names, reserved identifiers and backend limits are checked by the domain. */
readonly "params"?: (ReadonlyArray<(InputDefinitions["Param"])> | (number extends ((InputDefinitions["Param"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface InputDefinitions {
readonly "Mode": (((({
readonly "kind": ("hooks") & (string);
})) & { readonly "lit"?: never; }) | ((({
readonly "kind": ("sprite") & (string);
})) & { readonly "lit"?: never; }) | (({
readonly "kind": ("full") & (string);
readonly "lit": (boolean);
})));
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
