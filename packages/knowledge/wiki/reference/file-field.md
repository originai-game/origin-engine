<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-field"></a>
### `file:field`

Checked `file["field"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
/** Interleaved scalar channels per cell; their meaning belongs to the author. */
readonly "channels": (number);
/** Positive sample counts; x varies fastest, then y, then z. */
readonly "dimensions": (ReadonlyArray<(number)> & { readonly length: 3 });
/** First sample position in authored coordinate units. */
readonly "origin"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Positive sample spacing in authored coordinate units. */
readonly "spacing"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Finite f32 samples; dimensions product times channels, without normalization. */
readonly "values": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
```
