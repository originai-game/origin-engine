<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-vfx2d"></a>
### `file:vfx2d`

Checked `file["vfx2d"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "duration"?: ((number) | (null));
readonly "events"?: (ReadonlyArray<(InputDefinitions["Event"])> | (number extends ((InputDefinitions["Event"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "impact"?: ((number) | (null));
readonly "loop"?: ((boolean) | (null));
readonly "recipe": (InputDefinitions["Recipe"]);
});
export interface InputDefinitions {
readonly "Blend": ("over" | "under") & (string);
readonly "BoltMode": ("radial" | "strike" | "line") & (string);
readonly "Core": ("dark" | "light") & (string);
readonly "Event": ({
readonly "name": (string);
readonly "time": (number);
});
readonly "Layer": (((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cool"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "drift"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "erode"?: ((number) | (null));
readonly "erodeStart"?: ((number) | (null));
readonly "evolve"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hollow"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "r0"?: ((number) | (null));
readonly "r1"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("burst") & (string);
readonly "wobble"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "breakup"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fade"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "inner"?: ((number) | (null));
readonly "r0"?: ((number) | (null));
readonly "r1"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "thEnd"?: ((number) | (null));
readonly "type": ("ring") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "flick"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hgt"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hollow"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "taper"?: ((number) | (null));
readonly "type": ("flame") & (string);
readonly "w"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hold"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cap"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "type": ("beam") & (string);
readonly "wob"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "a0"?: ((number) | (null));
readonly "a1"?: ((number) | (null));
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "erode"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "sweep"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "trail"?: ((number) | (null));
readonly "type": ("slash") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "pulse"?: ((number) | (null));
readonly "pulseRate"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "rim"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("orb") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "from"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("comet") & (string);
readonly "until"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "arms"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "core"?: (((InputDefinitions["Core"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dir"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "rIn"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "twist"?: ((number) | (null));
readonly "type": ("vortex") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hgt"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("pillar") & (string);
readonly "w"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "inner"?: ((number) | (null));
readonly "jitter"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "twinkle"?: ((number) | (null));
readonly "type": ("rays") & (string);
readonly "width"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "jag"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "draw"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fill"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "rings"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "runes"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "star"?: ((number) | (null));
readonly "step"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "ticks"?: ((number) | (null));
readonly "type": ("glyph") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cone"?: ((number) | (null));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "decay"?: ((number) | (null));
readonly "dir"?: ((number) | (null));
readonly "dissolve"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "drift"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "emit"?: ((number) | (null));
readonly "flat"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "life"?: (((InputDefinitions["Pair"])) | ((null)));
readonly "r0"?: ((number) | (null));
readonly "r1"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "rise"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "speed"?: (((InputDefinitions["Pair"])) | ((null)));
readonly "spread"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("puffs") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "depth"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "spin"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "area"?: ((number) | (null));
readonly "areaX"?: ((number) | (null));
readonly "areaY"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "both"?: ((boolean) | (null));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dir"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "drag"?: ((number) | (null));
readonly "driftX"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "emit"?: ((number) | (null));
readonly "fadeIn"?: ((number) | (null));
readonly "fallScale"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "gravity"?: ((number) | (null));
readonly "ground"?: ((number) | (null));
readonly "heat"?: (((InputDefinitions["Pair"])) | ((null)));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "implodeR"?: ((number) | (null));
readonly "jitter"?: ((number) | (null));
readonly "life"?: (((InputDefinitions["Pair"])) | ((null)));
readonly "mode"?: (((InputDefinitions["ParticleMode"])) | ((null)));
readonly "orbitR"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "shape"?: (((InputDefinitions["ParticleShape"])) | ((null)));
readonly "size"?: (((InputDefinitions["Pair"])) | ((null)));
readonly "spanX"?: ((number) | (null));
readonly "spanY"?: ((number) | (null));
readonly "speed"?: (((InputDefinitions["Pair"])) | ((null)));
readonly "spread"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "swirl"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "turns"?: ((number) | (null));
readonly "twinkle"?: ((number) | (null));
readonly "type": ("particles") & (string);
readonly "wiggle"?: ((number) | (null));
readonly "wiggleRate"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "arms"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dissolve"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "dur"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeStart"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "grow"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "spin"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "branch"?: ((number) | (null));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "depth"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fadeStart"?: ((number) | (null));
readonly "flicker"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hold"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "jag"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "mode"?: (((InputDefinitions["BoltMode"])) | ((null)));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spread"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "thick"?: ((number) | (null));
readonly "type": ("bolt") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "x0"?: ((number) | (null));
readonly "x1"?: ((number) | (null));
readonly "y"?: ((number) | (null));
readonly "y0"?: ((number) | (null));
readonly "y1"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fadeStart"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("cracks") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "count"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fan"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "grow"?: ((number) | (null));
readonly "hgt"?: (((InputDefinitions["Pair"])) | ((null)));
readonly "hidden"?: ((boolean) | (null));
readonly "hold"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "lean"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spread"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("spikes") & (string);
readonly "width"?: (((InputDefinitions["Pair"])) | ((null)));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hollow"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "inner"?: ((number) | (null));
readonly "jitter"?: ((number) | (null));
readonly "points"?: ((number) | (null));
readonly "pop"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "sharp"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("star") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "implodeR"?: never; readonly "jag"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "count"?: ((number) | (null));
readonly "curve"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dir"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "gap"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "len"?: ((number) | (null));
readonly "mode"?: (((InputDefinitions["StreakMode"])) | ((null)));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "spanY"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "stagger"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "turns"?: ((number) | (null));
readonly "type": ("streaks") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "life"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cell"?: ((number) | (null));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fill"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hex"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "shimmer"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "th"?: ((number) | (null));
readonly "type": ("bubble") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "dur"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "r"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "type": ("flash") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "fadeStart"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "pop"?: ((boolean) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "rise"?: ((number) | (null));
readonly "scale"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "text"?: ((string) | (null));
readonly "type": ("text") & (string);
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "freq"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hgt"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rough"?: never; readonly "runes"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spin"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sway"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "twist"?: never; readonly "until"?: never; readonly "w"?: never; readonly "w0"?: never; readonly "w1"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }) | ((({
readonly "angle"?: ((number) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "cut"?: ((number) | (null));
readonly "dither"?: ((number) | (null));
readonly "ease"?: ((string) | (null));
readonly "freq"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "gamma"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "glowFalloff"?: ((number) | (null));
readonly "hgt"?: ((number) | (null));
readonly "hidden"?: ((boolean) | (null));
readonly "hue"?: ((number) | (null));
readonly "ramp"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "rough"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "spin"?: ((number) | (null));
readonly "squash"?: ((number) | (null));
readonly "sway"?: ((number) | (null));
/** Normalized effect time; a layer may extend past the clip. */
readonly "t0"?: ((number) | (null));
readonly "t1"?: ((number) | (null));
readonly "twist"?: ((number) | (null));
readonly "type": ("tornado") & (string);
readonly "w0"?: ((number) | (null));
readonly "w1"?: ((number) | (null));
/** Half the shorter canvas dimension, y down. */
readonly "x"?: ((number) | (null));
readonly "y"?: ((number) | (null));
})) & { readonly "a0"?: never; readonly "a1"?: never; readonly "area"?: never; readonly "areaX"?: never; readonly "areaY"?: never; readonly "arms"?: never; readonly "both"?: never; readonly "branch"?: never; readonly "breakup"?: never; readonly "cap"?: never; readonly "cell"?: never; readonly "cone"?: never; readonly "cool"?: never; readonly "core"?: never; readonly "count"?: never; readonly "curve"?: never; readonly "decay"?: never; readonly "depth"?: never; readonly "dir"?: never; readonly "dissolve"?: never; readonly "drag"?: never; readonly "draw"?: never; readonly "drift"?: never; readonly "driftX"?: never; readonly "dur"?: never; readonly "emit"?: never; readonly "erode"?: never; readonly "erodeStart"?: never; readonly "evolve"?: never; readonly "fade"?: never; readonly "fadeIn"?: never; readonly "fadeStart"?: never; readonly "fallScale"?: never; readonly "fan"?: never; readonly "fill"?: never; readonly "flat"?: never; readonly "flick"?: never; readonly "flicker"?: never; readonly "from"?: never; readonly "gap"?: never; readonly "gravity"?: never; readonly "ground"?: never; readonly "grow"?: never; readonly "heat"?: never; readonly "hex"?: never; readonly "hold"?: never; readonly "hollow"?: never; readonly "implodeR"?: never; readonly "inner"?: never; readonly "jag"?: never; readonly "jitter"?: never; readonly "lean"?: never; readonly "len"?: never; readonly "life"?: never; readonly "mode"?: never; readonly "orbitR"?: never; readonly "points"?: never; readonly "pop"?: never; readonly "pulse"?: never; readonly "pulseRate"?: never; readonly "r"?: never; readonly "r0"?: never; readonly "r1"?: never; readonly "rIn"?: never; readonly "rim"?: never; readonly "rings"?: never; readonly "rise"?: never; readonly "runes"?: never; readonly "scale"?: never; readonly "shape"?: never; readonly "sharp"?: never; readonly "shimmer"?: never; readonly "size"?: never; readonly "spanX"?: never; readonly "spanY"?: never; readonly "speed"?: never; readonly "spread"?: never; readonly "stagger"?: never; readonly "star"?: never; readonly "step"?: never; readonly "sweep"?: never; readonly "swirl"?: never; readonly "taper"?: never; readonly "text"?: never; readonly "th"?: never; readonly "thEnd"?: never; readonly "thick"?: never; readonly "ticks"?: never; readonly "trail"?: never; readonly "turns"?: never; readonly "twinkle"?: never; readonly "until"?: never; readonly "w"?: never; readonly "width"?: never; readonly "wiggle"?: never; readonly "wiggleRate"?: never; readonly "wob"?: never; readonly "wobble"?: never; readonly "x0"?: never; readonly "x1"?: never; readonly "y0"?: never; readonly "y1"?: never; }));
readonly "Outline": ("none" | "outside" | "outside8" | "inside") & (string);
readonly "Pair": (((number)) | ((ReadonlyArray<(number)> & { readonly length: 2 })));
readonly "ParticleMode": ("burst" | "spray" | "rise" | "fall" | "ambient" | "orbit" | "implode") & (string);
readonly "ParticleShape": ("dot" | "square" | "plus" | "cross" | "star" | "diamond" | "ring" | "bubble" | "streak" | "drop" | "shard" | "leaf" | "petal" | "chevron" | "coin" | "chunk" | "circle" | "heart") & (string);
readonly "Ramp": (((string)) | ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))));
readonly "Recipe": ({
readonly "cleanup"?: ((boolean) | (null));
readonly "dither"?: ((number) | (null));
readonly "fps"?: ((number) | (null));
readonly "frames"?: ((number) | (null));
readonly "h"?: ((number) | (null));
readonly "hue"?: ((number) | (null));
readonly "impact"?: ((number) | (null));
readonly "innerLine"?: ((number) | (null));
readonly "layers": (ReadonlyArray<(InputDefinitions["Layer"])> | (number extends ((InputDefinitions["Layer"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "line"?: ((number) | (null));
readonly "loop"?: ((boolean) | (null));
readonly "name"?: ((string) | (null));
readonly "outline"?: (((InputDefinitions["Outline"])) | ((null)));
readonly "outlineColor"?: ((string) | (null));
readonly "palette"?: (((InputDefinitions["Ramp"])) | ((null)));
readonly "pivot"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "ramps"?: (Readonly<Record<string, (InputDefinitions["Ramp"])>>);
readonly "sat"?: ((number) | (null));
readonly "scale"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
readonly "supersample"?: ((number) | (null));
readonly "w"?: ((number) | (null));
});
readonly "StreakMode": ("claw" | "speed" | "flow") & (string);
}
```
