<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-effect"></a>
### `file:effect`

Checked `file["effect"](input)` declaration from `@origingame/engine/author`.

```ts
/** Effect material input, not text containing another author language. */
export type Input = ({
readonly "alpha"?: (((InputDefinitions["EffectValue"])) | ((null)));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "custom"?: ((ReadonlyArray<(InputDefinitions["EffectValue"])> | (number extends ((InputDefinitions["EffectValue"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "doubleSided"?: ((boolean) | (null));
readonly "erosion"?: (((InputDefinitions["EffectValue"])) | ((null)));
readonly "erosionEdge"?: (((InputDefinitions["ErosionEdge"])) | ((null)));
readonly "erosionSoftness"?: ((number) | (null));
readonly "flow"?: (((InputDefinitions["FlowInput"])) | ((null)));
readonly "flowMap"?: (((InputDefinitions["EffectTextureSlot"])) | ((null)));
readonly "fresnel"?: (((InputDefinitions["FresnelInput"])) | ((null)));
readonly "intensity"?: (((InputDefinitions["EffectValue"])) | ((null)));
readonly "main"?: (((InputDefinitions["EffectTextureSlot"])) | ((null)));
readonly "mask"?: (((InputDefinitions["EffectTextureSlot"])) | ((null)));
readonly "noise"?: (((InputDefinitions["EffectTextureSlot"])) | ((null)));
readonly "offsetU"?: (((InputDefinitions["EffectValue"])) | ((null)));
readonly "offsetV"?: (((InputDefinitions["EffectValue"])) | ((null)));
readonly "params"?: ((Readonly<Record<string, (InputDefinitions["ShaderParam"])>>) | (null));
readonly "ramp"?: ((ReadonlyArray<(readonly [(number), (ReadonlyArray<(number)> & { readonly length: 4 })])> | (number extends ((readonly [(number), (ReadonlyArray<(number)> & { readonly length: 4 })])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "shader"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "softDepth"?: ((number) | (null));
});
export interface InputDefinitions {
readonly "Blend": ("alpha" | "additive" | "multiply" | "subtractive") & (string);
readonly "Channel": ("rgba" | "red" | "green" | "blue" | "alpha") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "EffectTextureSlot": ({
readonly "channel"?: (((InputDefinitions["Channel"])) | ((null)));
readonly "texture"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "uv"?: (((InputDefinitions["UvInput"])) | ((null)));
});
/** A scalar constant, per-instance range or ordered lifetime curve. */
readonly "EffectValue": (((number)) | ((InputDefinitions["ValueInput"])));
readonly "ErosionEdge": ({
readonly "color": (ReadonlyArray<(number)> & { readonly length: 4 });
readonly "width": (number);
});
readonly "FlowInput": ({
readonly "frequency"?: ((number) | (null));
readonly "strength"?: ((number) | (null));
});
readonly "FresnelInput": ({
readonly "invert"?: ((boolean) | (null));
readonly "power": (number);
readonly "strength"?: ((number) | (null));
});
readonly "ShaderParam": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** UV transform evaluated over the surface's life. */
readonly "UvInput": ({
readonly "offset"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "polar"?: ((boolean) | (null));
readonly "rotate"?: ((number) | (null));
readonly "scroll"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "tile"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "ValueInput": (((({
readonly "kind": ("random") & (string);
readonly "max": (number);
readonly "min": (number);
})) & { readonly "keys"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("curve") & (string);
})) & { readonly "max"?: never; readonly "min"?: never; }));
}
```
