<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-icons"></a>
### `file:icons`

Checked `file["icons"](input)` declaration from `@origingame/engine/author`.

```ts
/** An icon declaration; SVG validation and distance-field baking stay native. */
export type Input = ({
/** (name, SVG file stem), in index order.
Unique names and paths beneath source are checked by the icon validator. */
readonly "icons": (ReadonlyArray<(readonly [(string), (string)])> | (number extends ((readonly [(string), (string)])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** The join of a stroked path that names none (`join`; SVG's miter). */
readonly "join"?: (InputDefinitions["Join"]);
/** MSDF distance range in pixels. */
readonly "range"?: (number);
/** Square MSDF glyph size in pixels. */
readonly "size"?: (number);
/** Canonical directory of SVGs relative to the owning module, for example
assets/icons. No source-file-relative or asset-root fallback. */
readonly "source"?: (string);
/** A toned icon's outline width and shadow offset, in 24ths of the em. */
readonly "sticker"?: (readonly [(number), (number), (number)]);
});
export interface InputDefinitions {
/** How a stroke's segments meet at a corner (SVG's `stroke-linejoin`). */
readonly "Join": ((("miter") & (string)) | (("round") & (string)) | (("bevel") & (string)));
}
```
