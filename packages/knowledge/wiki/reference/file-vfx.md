<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-vfx"></a>
### `file:vfx`

Checked `file["vfx"](input)` declaration from `@origingame/engine/author`.

```ts
/** A composed effect timeline. Element end times, parameter identities and
duplicate event names are checked by the domain validator. */
export type Input = ({
readonly "duration": (number);
readonly "elements": (ReadonlyArray<(InputDefinitions["VfxElement"])> | (number extends ((InputDefinitions["VfxElement"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "events"?: ((ReadonlyArray<(InputDefinitions["VfxEvent"])> | (number extends ((InputDefinitions["VfxEvent"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "looping"?: ((boolean) | (null));
readonly "params"?: ((ReadonlyArray<(InputDefinitions["VfxParam"])> | (number extends ((InputDefinitions["VfxParam"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
});
export interface InputDefinitions {
readonly "Blend": ("alpha" | "additive" | "multiply" | "subtractive") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
/** A scalar constant, per-instance range or ordered lifetime curve. */
readonly "EffectValue": (((number)) | ((InputDefinitions["ValueInput"])));
readonly "RingInput": ({
readonly "band"?: ((number) | (null));
readonly "colors"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 4 })> & { readonly length: 3 }) | (null));
readonly "height"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "inner"?: ((number) | (null));
readonly "length"?: ((number) | (null));
readonly "segments"?: ((number) | (null));
readonly "start"?: ((number) | (null));
});
readonly "SpriteBlend": ("alpha" | "additive") & (string);
readonly "ValueInput": (((({
readonly "kind": ("random") & (string);
readonly "max": (number);
readonly "min": (number);
})) & { readonly "keys"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("curve") & (string);
})) & { readonly "max"?: never; readonly "min"?: never; }));
readonly "VfxElement": ({
readonly "acceleration"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "amount"?: ((string) | (null));
readonly "at"?: ((number) | (null));
readonly "life"?: ((number) | (null));
readonly "node": (InputDefinitions["VfxNode"]);
readonly "offset"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "rotate"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** End time instead of life; both together are a domain error. */
readonly "until"?: ((number) | (null));
readonly "velocity"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "when"?: ((string) | (null));
});
readonly "VfxEvent": ({
/** At most 63 UTF-8 bytes (domain validation, not character count). */
readonly "name": (string);
readonly "time": (number);
});
readonly "VfxNode": (((({
readonly "asset": (InputDefinitions["ContentRef"]);
readonly "kind": ("emitter") & (string);
readonly "seed"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "blend"?: never; readonly "color"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "effect": (InputDefinitions["ContentRef"]);
readonly "faceCamera"?: ((boolean) | (null));
readonly "kind": ("mesh") & (string);
readonly "scale"?: (((InputDefinitions["EffectValue"])) | ((null)));
readonly "shape": (InputDefinitions["VfxShape"]);
readonly "spin"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "stretch"?: ((string) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "distance"?: never; readonly "emissive"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "size"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "effect"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "kind": ("trail") & (string);
readonly "lifetime"?: ((number) | (null));
/** Radius in metres and turns per second about local Y. */
readonly "orbit"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "rise"?: ((number) | (null));
readonly "width"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "distance"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "pitch"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; }) | ((({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "effect"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "jitter"?: ((number) | (null));
readonly "kind": ("beam") & (string);
readonly "segments"?: ((number) | (null));
readonly "stretch"?: ((string) | (null));
readonly "to"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "width"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "distance"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "texture"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; }) | ((({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "distance"?: ((number) | (null));
readonly "intensity"?: (((InputDefinitions["EffectValue"])) | ((null)));
readonly "kind": ("light") & (string);
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "angleFade"?: ((number) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
/** Linear emission; HDR values above one are allowed. */
readonly "emissive"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "fade"?: ((number) | (null));
readonly "kind": ("decal") & (string);
/** Omitted preserves the receiving surface's metallic value. */
readonly "metallic"?: ((number) | (null));
/** Tangent-space glTF normal map: +X along U, +Y toward image top.
Shares the colour texture's UV region; omitted leaves normals alone. */
readonly "normal"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "normalScale"?: ((number) | (null));
/** Omitted preserves the receiving surface's roughness. */
readonly "roughness"?: ((number) | (null));
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Omitted means a solid-colour decal. */
readonly "texture"?: (((InputDefinitions["ContentRef"])) | ((null)));
/** Seeded per-instance rotation interval about the projection axis. */
readonly "turn"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Region of this decal's texture and normal, not a global atlas. */
readonly "uv"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "wet"?: ((boolean) | (null));
})) & { readonly "asset"?: never; readonly "blend"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "faceCamera"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "volume"?: never; readonly "width"?: never; }) | ((({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "kind": ("glow") & (string);
readonly "size"?: (((InputDefinitions["EffectValue"])) | ((null)));
readonly "stretch"?: ((string) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "spin"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "kind": ("shake") & (string);
readonly "trauma"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "asset"?: never; readonly "blend"?: never; readonly "color"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "asset": (InputDefinitions["ContentRef"]);
readonly "kind": ("sound") & (string);
/** Playback-rate interval; 1 is the recording's original pitch. */
readonly "pitch"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "volume"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "blend"?: never; readonly "color"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "size"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "wet"?: never; readonly "width"?: never; }) | ((({
readonly "asset": (InputDefinitions["ContentRef"]);
readonly "blend"?: (((InputDefinitions["SpriteBlend"])) | ((null)));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "kind": ("flipbook") & (string);
readonly "size"?: ((number) | (null));
})) & { readonly "angleFade"?: never; readonly "distance"?: never; readonly "effect"?: never; readonly "emissive"?: never; readonly "faceCamera"?: never; readonly "fade"?: never; readonly "intensity"?: never; readonly "jitter"?: never; readonly "lifetime"?: never; readonly "metallic"?: never; readonly "normal"?: never; readonly "normalScale"?: never; readonly "orbit"?: never; readonly "pitch"?: never; readonly "rise"?: never; readonly "roughness"?: never; readonly "scale"?: never; readonly "seed"?: never; readonly "segments"?: never; readonly "shape"?: never; readonly "spin"?: never; readonly "stretch"?: never; readonly "texture"?: never; readonly "to"?: never; readonly "trauma"?: never; readonly "turn"?: never; readonly "uv"?: never; readonly "volume"?: never; readonly "wet"?: never; readonly "width"?: never; }));
readonly "VfxParam": ({
readonly "default": (number);
readonly "name": (string);
});
readonly "VfxShape": (((({
readonly "kind": ("sphere") & (string);
})) & { readonly "asset"?: never; readonly "ring"?: never; }) | ((({
readonly "kind": ("quad") & (string);
})) & { readonly "asset"?: never; readonly "ring"?: never; }) | ((({
readonly "kind": ("box") & (string);
})) & { readonly "asset"?: never; readonly "ring"?: never; }) | ((({
readonly "kind": ("cylinder") & (string);
})) & { readonly "asset"?: never; readonly "ring"?: never; }) | ((({
readonly "kind": ("ring") & (string);
readonly "ring"?: (((InputDefinitions["RingInput"])) | ((null)));
})) & { readonly "asset"?: never; }) | ((({
readonly "asset": (InputDefinitions["ContentRef"]);
readonly "kind": ("model") & (string);
})) & { readonly "ring"?: never; }));
}
```
