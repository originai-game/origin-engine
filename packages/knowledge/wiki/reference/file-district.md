<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-district"></a>
### `file:district`

Checked `file["district"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "buildings"?: (((InputDefinitions["Buildings"])) | ((null)));
readonly "cells"?: (((InputDefinitions["Cells"])) | ((null)));
readonly "hlod"?: (((InputDefinitions["HlodInput"])) | ((null)));
readonly "lots": (InputDefinitions["Lots"]);
readonly "materials": (InputDefinitions["ContentRef"]);
/** Omitted: merge generated static geometry. */
readonly "merge"?: ((boolean) | (null));
readonly "props"?: (ReadonlyArray<(InputDefinitions["Prop"])> | (number extends ((InputDefinitions["Prop"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "region": (InputDefinitions["Region"]);
readonly "seed"?: ((number) | (null));
readonly "streets"?: (((InputDefinitions["Streets"])) | ((null)));
readonly "terrain"?: (((InputDefinitions["Terrain"])) | ((null)));
});
export interface InputDefinitions {
readonly "Buildings": ({
readonly "density"?: ((number) | (null));
readonly "height"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "heightNoise"?: (((InputDefinitions["HeightNoise"])) | ((null)));
readonly "rules"?: (ReadonlyArray<(InputDefinitions["RuleUse"])> | (number extends ((InputDefinitions["RuleUse"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Cells": ({
readonly "load"?: ((number) | (null));
readonly "size": (number);
});
readonly "Collider": (((({
readonly "kind": ("box") & (string);
readonly "size": (ReadonlyArray<(number)> & { readonly length: 3 });
})) & { readonly "height"?: never; readonly "radius"?: never; }) | ((({
readonly "height": (number);
readonly "kind": ("capsule") & (string);
readonly "radius": (number);
})) & { readonly "size"?: never; }));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Grid": ({
readonly "jitter"?: ((number) | (null));
readonly "sidewalk"?: ((number) | (null));
readonly "size": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "turn"?: ((number) | (null));
readonly "width"?: ((number) | (null));
});
readonly "HeightNoise": ({
readonly "frequency": (number);
readonly "share": (number);
});
readonly "HlodInput": ({
/** Maximum proxy displacement in metres. Omitted/zero: size / 256;
a supplied positive value must be at most size / 4 (domain check). */
readonly "error"?: ((number) | (null));
/** Leaf cell side in metres; omitted: 64. */
readonly "size"?: ((number) | (null));
/** Longest proxy texture side in pixels; omitted: 1024. */
readonly "texels"?: ((number) | (null));
});
readonly "Lots": ({
readonly "depth": (ReadonlyArray<(number)> & { readonly length: 2 });
/** Least/greatest dimensions in metres; ordering checked by layout domain. */
readonly "width": (ReadonlyArray<(number)> & { readonly length: 2 });
});
readonly "Prop": ({
readonly "collider"?: (((InputDefinitions["Collider"])) | ((null)));
readonly "corners"?: ((boolean) | (null));
readonly "every"?: ((number) | (null));
readonly "jitter"?: ((number) | (null));
readonly "kerb"?: ((number) | (null));
readonly "model": (InputDefinitions["ContentRef"]);
readonly "name": (string);
readonly "street"?: ((string) | (null));
});
readonly "Region": (((({
readonly "kind": ("area") & (string);
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "depth"?: never; readonly "street"?: never; }) | ((({
readonly "depth"?: ((number) | (null));
readonly "kind": ("along") & (string);
readonly "street": (InputDefinitions["Street"]);
})) & { readonly "points"?: never; }));
readonly "RuleUse": ({
readonly "grammar": (InputDefinitions["ContentRef"]);
readonly "street"?: ((string) | (null));
readonly "weight"?: ((number) | (null));
});
readonly "Street": ({
readonly "name"?: ((string) | (null));
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "sidewalk"?: ((number) | (null));
readonly "width"?: ((number) | (null));
});
readonly "Streets": ({
/** Grid street settings are explicit, never inferred from authored streets. */
readonly "grid"?: (((InputDefinitions["Grid"])) | ((null)));
readonly "kerb"?: ((number) | (null));
readonly "streets"?: (ReadonlyArray<(InputDefinitions["Street"])> | (number extends ((InputDefinitions["Street"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Terrain": ({
readonly "depth"?: ((number) | (null));
readonly "terrain": (InputDefinitions["ContentRef"]);
});
}
```
