<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-tileset"></a>
### `file:tileset`

Checked `file["tileset"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "columns": (number);
/** Per-tileset art conformance; omission uses the project's art contract. */
readonly "conform"?: (((InputDefinitions["TilesetConform"])) | ((null)));
readonly "filter"?: (((InputDefinitions["Filter"])) | ((null)));
readonly "margin"?: (number);
readonly "rows": (number);
readonly "spacing"?: (number);
readonly "terrains"?: (ReadonlyArray<(InputDefinitions["Terrain"])> | (number extends ((InputDefinitions["Terrain"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "texture"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "tileHeight": (number);
readonly "tileWidth": (number);
readonly "tiles"?: (Readonly<Record<string, ((InputDefinitions["Tile"]))>>);
});
export interface InputDefinitions {
readonly "Animation": ({
/** One strictly positive duration per frame; domain checks matching counts. */
readonly "durations": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "frames": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Collision": (((({
readonly "kind": ("full") & (string);
})) & { readonly "h"?: never; readonly "w"?: never; readonly "x"?: never; readonly "y"?: never; }) | (({
readonly "h": (number);
readonly "kind": ("rect") & (string);
readonly "w": (number);
readonly "x": (number);
readonly "y": (number);
})));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Filter": ("nearest" | "linear") & (string);
readonly "Terrain": ({
readonly "default": (number);
readonly "edgeFilled"?: ((boolean) | (null));
readonly "mode": (InputDefinitions["TerrainMode"]);
readonly "name": (string);
/** Neighbor mask to tile id. Domain checks reachable masks and tile bounds. */
readonly "rules": (Readonly<Record<string, ((number))>>);
});
readonly "TerrainMode": ("blob47" | "bitmask16") & (string);
readonly "Tile": ({
readonly "animation"?: (((InputDefinitions["Animation"])) | ((null)));
readonly "collision"?: (((InputDefinitions["Collision"])) | ((null)));
/** Closed polygons, top-left pixel coordinates, y down. */
readonly "polygons"?: (ReadonlyArray<(ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))> | (number extends ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "TilesetConform": (((({
readonly "kind": ("off") & (string);
})) & { readonly "size"?: never; }) | (({
readonly "kind": ("size") & (string);
readonly "size": (number);
})));
}
```
