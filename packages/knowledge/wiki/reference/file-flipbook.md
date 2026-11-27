<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-flipbook"></a>
### `file:flipbook`

Checked `file["flipbook"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ((((InputDefinitions["TextureSheet"])) & { readonly "align"?: never; readonly "crop"?: never; readonly "dedupe"?: never; readonly "key"?: never; readonly "padding"?: never; readonly "scale"?: never; readonly "sources"?: never; }) | (((InputDefinitions["AnimatedSheet"])) & { readonly "atlas"?: never; readonly "frames"?: never; readonly "grid"?: never; readonly "margin"?: never; readonly "spacing"?: never; readonly "texture"?: never; }));
export interface InputDefinitions {
readonly "Alignment": ("feet" | "bbox" | "centroid" | "none") & (string);
readonly "AnimatedSheet": ({
/** Omitted: no alignment. */
readonly "align"?: (((InputDefinitions["Alignment"])) | ((null)));
readonly "array"?: (boolean);
/** Explicit clips override an imported clip of the same name. */
readonly "clips"?: (ReadonlyArray<(InputDefinitions["Clip"])> | (number extends ((InputDefinitions["Clip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "conform"?: (((InputDefinitions["TilesetConform"])) | ((null)));
/** Omitted all: common frame size; each packs sources in separate sizes. */
readonly "crop"?: (((InputDefinitions["Crop"])) | ((null)));
/** Omitted true: identical frames share a cell. */
readonly "dedupe"?: ((boolean) | (null));
readonly "filter"?: (((InputDefinitions["Filter"])) | ((null)));
/** Omitted: no background keying. */
readonly "key"?: (((InputDefinitions["ChromaKey"])) | ((null)));
/** Transparent texels around the common crop; omitted 1. */
readonly "padding"?: ((number) | (null));
readonly "pivot"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Omitted 1; resample before measuring. */
readonly "scale"?: ((number) | (null));
/** Explicit animated GIF, WebP, APNG or still paths under this module's
assets/ or sources/. No declaration-relative or sibling lookup. */
readonly "sources": (ReadonlyArray<(InputDefinitions["AnimatedSource"])> | (number extends ((InputDefinitions["AnimatedSource"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "AnimatedSource": ({
readonly "align"?: (((InputDefinitions["Alignment"])) | ((null)));
/** Omitted: the source's file stem. */
readonly "clip"?: ((string) | (null));
readonly "events"?: (ReadonlyArray<(readonly [(number), (string)])> | (number extends ((readonly [(number), (string)])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "fps"?: ((number) | (null));
readonly "mode"?: (((InputDefinitions["FlipMode"])) | ((null)));
readonly "scale"?: ((number) | (null));
readonly "source": (string);
});
readonly "ChromaKey": ({
/** Omitted: auto-detect the flat border colour; explicit RGB8 otherwise. */
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Also key enclosed pockets of background; omitted false. */
readonly "holes"?: (boolean);
/** Omitted 0.1. */
readonly "tolerance"?: ((number) | (null));
});
readonly "Clip": ({
/** One strictly positive duration per frame; otherwise use fps. */
readonly "durations"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "events"?: (ReadonlyArray<(readonly [(number), (string)])> | (number extends ((readonly [(number), (string)])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "fps"?: ((number) | (null));
/** Frame names, resolved only by the domain validator. */
readonly "frames": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "mode"?: (((InputDefinitions["FlipMode"])) | ((null)));
readonly "name": (string);
});
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Crop": ("all" | "each") & (string);
readonly "Filter": ("nearest" | "linear") & (string);
readonly "FlipMode": ("loop" | "once" | "pingPong") & (string);
readonly "Frame": ({
readonly "name": (string);
/** Bottom-left fractions; a pivot may lie outside the frame. */
readonly "pivot"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Top-left x, y, width, height. Extents checked against the texture. */
readonly "rect": (ReadonlyArray<(number)> & { readonly length: 4 });
});
readonly "Grid": ({
readonly "columns"?: ((number) | (null));
readonly "rows"?: ((number) | (null));
readonly "size": (ReadonlyArray<(number)> & { readonly length: 2 });
});
readonly "TextureSheet": ({
readonly "array"?: (boolean);
/** Explicit Aseprite or TexturePacker exchange text from input.text. */
readonly "atlas"?: ((string) | (null));
readonly "clips"?: (ReadonlyArray<(InputDefinitions["Clip"])> | (number extends ((InputDefinitions["Clip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "conform"?: (((InputDefinitions["TilesetConform"])) | ((null)));
readonly "filter"?: (((InputDefinitions["Filter"])) | ((null)));
readonly "frames"?: (ReadonlyArray<(InputDefinitions["Frame"])> | (number extends ((InputDefinitions["Frame"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Grid and explicit/imported frames are composed by the domain validator. */
readonly "grid"?: (((InputDefinitions["Grid"])) | ((null)));
readonly "margin"?: (number);
/** Bottom-left fractions; a pivot may lie outside the frame. */
readonly "pivot"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "spacing"?: (number);
readonly "texture": (InputDefinitions["ContentRef"]);
});
readonly "TilesetConform": (((({
readonly "kind": ("off") & (string);
})) & { readonly "size"?: never; }) | (({
readonly "kind": ("size") & (string);
readonly "size": (number);
})));
}
```
