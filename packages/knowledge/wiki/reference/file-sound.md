<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-sound"></a>
### `file:sound`

Checked `file["sound"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
/** One mono channel or interleaved stereo. */
readonly "channels": (number);
/** Sample rate in hertz; no resampling before the codec import. */
readonly "rate": (number);
/** Finite PCM samples, at least one complete interleaved frame. */
readonly "samples": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
```
