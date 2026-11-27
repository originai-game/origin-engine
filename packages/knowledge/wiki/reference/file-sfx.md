<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-sfx"></a>
### `file:sfx`

Checked `file["sfx"](input)` declaration from `@origingame/engine/author`.

```ts
/** Procedural sound: optional preset plus explicit oscillator/sample layers.
At least one preset/layer and maximum rendered duration are domain checks. */
export type Input = ({
readonly "layers"?: ((ReadonlyArray<(InputDefinitions["SynthLayer"])> | (number extends ((InputDefinitions["SynthLayer"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "length"?: ((number) | (null));
readonly "pitch"?: ((number) | (null));
readonly "preset"?: (((InputDefinitions["SfxPreset"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "volume"?: ((number) | (null));
});
export interface InputDefinitions {
readonly "Pcm": (number);
readonly "SampleInput": (((({
readonly "kind": ("file") & (string);
readonly "looping"?: ((boolean) | (null));
readonly "reference"?: ((number) | (null));
/** Explicit original audio input, relative to the owning module. */
readonly "source": (string);
})) & { readonly "rate"?: never; readonly "samples"?: never; }) | ((({
readonly "kind": ("pcm") & (string);
readonly "looping"?: ((boolean) | (null));
readonly "rate": (number);
readonly "reference"?: ((number) | (null));
readonly "samples": (ReadonlyArray<(InputDefinitions["Pcm"])> | (number extends ((InputDefinitions["Pcm"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "source"?: never; }));
readonly "SfxPreset": ("coin" | "pickup" | "powerup" | "jump" | "land" | "footstep" | "hit" | "hurt" | "explosion" | "laser" | "whoosh" | "door" | "death" | "click" | "select" | "back" | "error" | "levelup") & (string);
/** A mono sample replaces the oscillator; sample layers reject FM. */
readonly "SynthLayer": ({
readonly "arp"?: ((number) | (null));
readonly "arpAt"?: ((number) | (null));
readonly "attack"?: ((number) | (null));
readonly "decay"?: ((number) | (null));
readonly "delay"?: ((number) | (null));
readonly "drive"?: ((number) | (null));
readonly "duty"?: ((number) | (null));
readonly "fmFreq"?: ((number) | (null));
readonly "fmIndex"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "freqEnd"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "highpass"?: ((number) | (null));
readonly "lowpass"?: ((number) | (null));
readonly "lpSlide"?: ((number) | (null));
readonly "punch"?: ((number) | (null));
readonly "q"?: ((number) | (null));
readonly "release"?: ((number) | (null));
readonly "sample"?: (((InputDefinitions["SampleInput"])) | ((null)));
readonly "slide"?: ((number) | (null));
readonly "sustain"?: ((number) | (null));
readonly "sustainLevel"?: ((number) | (null));
readonly "vibDepth"?: ((number) | (null));
readonly "vibRate"?: ((number) | (null));
readonly "wave"?: (((InputDefinitions["Wave"])) | ((null)));
});
readonly "Wave": ("sine" | "square" | "saw" | "triangle" | "noise") & (string);
}
```
