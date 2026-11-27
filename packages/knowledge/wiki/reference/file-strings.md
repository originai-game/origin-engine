<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-strings"></a>
### `file:strings`

Checked `file["strings"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
/** Reference locale for key/argument checks; omitted is false. */
readonly "base"?: ((boolean) | (null));
/** Omitted: direction of locale. */
readonly "direction"?: (((InputDefinitions["Direction"])) | ((null)));
/** ICU-subset source messages, not an embedded author configuration. */
readonly "entries": (Readonly<Record<string, (string)>>);
readonly "fallback"?: ((string) | (null));
/** Ordered locale font fallback chain. */
readonly "fonts"?: (ReadonlyArray<(InputDefinitions["ContentRef"])> | (number extends ((InputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Canonicalized as a locale tag by the localization domain. */
readonly "locale": (string);
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
readonly "Direction": ("ltr" | "rtl") & (string);
}
```
