<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-tilemap"></a>
### `file:tilemap`

Checked `file["tilemap"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "height": (number);
readonly "layers": (ReadonlyArray<(InputDefinitions["Layer"])> | (number extends ((InputDefinitions["Layer"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "objects"?: (ReadonlyArray<(InputDefinitions["Object"])> | (number extends ((InputDefinitions["Object"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "orientation"?: (((InputDefinitions["Orientation"])) | ((null)));
readonly "tileHeight": (number);
readonly "tileWidth": (number);
readonly "tilesets": (ReadonlyArray<(InputDefinitions["ContentRef"])> | (number extends ((InputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** World units per tile width; height retains the authored pixel aspect ratio. */
readonly "unit"?: ((number) | (null));
readonly "width": (number);
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
readonly "Layer": ({
readonly "cellColliders"?: ((boolean) | (null));
/** Row-major, top row first: 0 empty; otherwise tile+1 with flip bits. */
readonly "cells": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "colliderLayers"?: ((number) | (null));
readonly "colliderMask"?: ((number) | (null));
readonly "collision"?: ((boolean) | (null));
readonly "lit"?: ((boolean) | (null));
readonly "name": (string);
readonly "offset"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "opacity"?: ((number) | (null));
readonly "parallax"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "repeatX"?: ((boolean) | (null));
readonly "repeatY"?: ((boolean) | (null));
readonly "terrain"?: ((string) | (null));
/** Index into the map's declared tilesets, checked by the domain. */
readonly "tileset": (number);
readonly "visible"?: ((boolean) | (null));
readonly "z"?: ((number) | (null));
});
readonly "Object": ({
readonly "height"?: ((number) | (null));
readonly "kind": (string);
readonly "name": (string);
readonly "properties"?: (Readonly<Record<string, (string)>>);
readonly "width"?: ((number) | (null));
/** Map-local world units, y up from the map's top-left. */
readonly "x": (number);
readonly "y": (number);
});
readonly "Orientation": ("orthogonal" | "isometric" | "staggered" | "hex" | "hexFlat") & (string);
}
```
