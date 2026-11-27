<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-materials"></a>
### `file:materials`

Checked `file["materials"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
/** Order is stable; duplicate names and generation requirements are checked
together by the material-library validator. */
readonly "materials": (ReadonlyArray<(InputDefinitions["LibraryMaterial"])> | (number extends ((InputDefinitions["LibraryMaterial"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface InputDefinitions {
readonly "Generate": ({
/** Authored image used to prepare the explicitly named map destinations. */
readonly "source": (string);
});
readonly "LibraryMaterial": ({
readonly "brief"?: ((string) | (null));
readonly "facade"?: ((boolean) | (null));
/** Explicit image-map preparation input, not a TS executor or runtime handle. */
readonly "generate"?: (((InputDefinitions["Generate"])) | ((null)));
/** Material labels produced by grammar surfaces. */
readonly "labels": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
readonly "occlusion"?: (((InputDefinitions["Occlusion"])) | ((null)));
readonly "relief"?: ((number) | (null));
readonly "roughnessRange"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** glTF material fields shared with authored models and fracture surfaces. */
readonly "surface": (InputDefinitions["Material"]);
/** Physical tile dimensions. Both dimensions are strictly positive. */
readonly "tile"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "vary"?: (((InputDefinitions["Variation"])) | ((null)));
readonly "weight"?: ((number) | (null));
});
/** Omitted fields remain absent until the material domain normalizes them. */
readonly "Material": ({
/** Alpha-mask cutoff takes precedence over opacity blending. */
readonly "alphaTest"?: ((number) | (null));
/** Ambient occlusion in red. */
readonly "aoMap"?: ((string) | (null));
readonly "autumnColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "autumnColor2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "clearcoat"?: ((number) | (null));
readonly "clearcoatRoughness"?: ((number) | (null));
/** Linear base RGB (glTF factor); not a runtime colour handle. */
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "domeNormals"?: ((number) | (null));
readonly "doubleSided"?: ((boolean) | (null));
/** Linear emitted RGB, multiplied by emissiveIntensity. */
readonly "emissive"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "emissiveIntensity"?: ((number) | (null));
readonly "grime"?: ((number) | (null));
readonly "grimeColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "grimeHeight"?: ((number) | (null));
readonly "ior"?: ((number) | (null));
/** Source image inputs; not cooked texture references. Resolution and path
containment are checked at the input boundary. */
readonly "map"?: ((string) | (null));
readonly "metalness"?: ((number) | (null));
/** Roughness in green, metalness in blue. */
readonly "metalnessMap"?: ((string) | (null));
readonly "normalMap"?: ((string) | (null));
readonly "normalScale"?: ((number) | (null));
readonly "opacity"?: ((number) | (null));
readonly "roughness"?: ((number) | (null));
/** Zero evergreen; one deciduous. */
readonly "season"?: ((number) | (null));
readonly "seasonShift"?: ((number) | (null));
readonly "stains"?: ((number) | (null));
readonly "stochastic"?: ((number) | (null));
readonly "thickness"?: ((number) | (null));
/** Diffuse transmission through thin leaves; enables both faces. */
readonly "translucency"?: ((number) | (null));
readonly "transmission"?: ((number) | (null));
});
readonly "Occlusion": ({
readonly "radius": (number);
readonly "strength": (number);
});
readonly "Variation": ({
readonly "value"?: ((number) | (null));
readonly "warmth"?: ((number) | (null));
});
}
```
