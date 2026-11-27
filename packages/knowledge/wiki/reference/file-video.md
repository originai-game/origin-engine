<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-video"></a>
### `file:video`

Checked `file["video"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "aiGenerated"?: ((boolean) | (null));
readonly "alpha"?: ((boolean) | (null));
/** Keep the source audio when present; omitted true. */
readonly "audio"?: ((boolean) | (null));
/** Alternate audio paths under this module's assets/ or sources/.
Locale tags are unique ignoring case; ASCII letters, digits, '-' and '_'. */
readonly "audioLocale"?: (Readonly<Record<string, (string)>>);
readonly "chunk"?: ((number) | (null));
readonly "crf"?: ((number) | (null));
readonly "crf720"?: ((number) | (null));
readonly "fps"?: ((number) | (null));
readonly "keyframe"?: ((number) | (null));
/** Frame processing; omitted color, pixel keeps nearest/lossless encoding. */
readonly "kind"?: (((InputDefinitions["FrameKind"])) | ((null)));
readonly "loudness"?: ((number) | (null));
readonly "lowRendition"?: ((boolean) | (null));
/** Seconds retained from the beginning; omitted: 8 for a clip, whole source for a stream. */
readonly "maxSeconds"?: ((number) | (null));
readonly "placeholder"?: ((boolean) | (null));
/** Zero keeps the full resolution; otherwise 64..4096 pixels. */
readonly "posterSize"?: (((0)) | ((number)) | ((null))) & ((number) | (null));
/** Browser codecs beside AV1; omitted HEVC when available, [] disables them.
All rendition controls require stream: true. */
readonly "renditions"?: ((ReadonlyArray<(InputDefinitions["Rendition"])> | (number extends ((InputDefinitions["Rendition"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Width, height in pixels. */
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Canonical .mp4, .webm or .mov path under the owning module's assets/
or sources/, without declaration-relative or sibling fallback. */
readonly "source": (string);
readonly "stream"?: ((boolean) | (null));
});
export interface InputDefinitions {
readonly "FrameKind": ("color" | "pixel") & (string);
readonly "Rendition": ("hevc" | "h264") & (string);
}
```
