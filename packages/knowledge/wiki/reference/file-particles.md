<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-particles"></a>
### `file:particles`

Checked `file["particles"](input)` declaration from `@origingame/engine/author`.

```ts
/** Emitter declaration. Rendering-mode relationships, curve ordering and
child-emitter cycles require the existing domain/graph validators. */
export type Input = ({
readonly "align"?: (((InputDefinitions["Align"])) | ((null)));
readonly "angularVelocity"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "array"?: (((InputDefinitions["ParticleArray"])) | ((null)));
readonly "attractors"?: ((ReadonlyArray<(InputDefinitions["Attractor"])> | (number extends ((InputDefinitions["Attractor"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "blend"?: (((InputDefinitions["Blend"])) | ((null)));
readonly "bursts"?: ((ReadonlyArray<(InputDefinitions["Burst"])> | (number extends ((InputDefinitions["Burst"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "collision"?: (((InputDefinitions["Collision"])) | ((null)));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "colorOverLife"?: (((InputDefinitions["ColorCurve"])) | ((null)));
readonly "distortion"?: ((number) | (null));
readonly "drag"?: ((number) | (null));
readonly "duration"?: ((number) | (null));
readonly "effect"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "events"?: ((ReadonlyArray<(InputDefinitions["ParticleEvent"])> | (number extends ((InputDefinitions["ParticleEvent"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "field"?: (((InputDefinitions["ParticleField"])) | ((null)));
readonly "forceFields"?: ((number) | (null));
readonly "glow"?: ((number) | (null));
readonly "gravity"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "heightOverLife"?: (((InputDefinitions["Curve"])) | ((null)));
readonly "inheritVelocity"?: ((number) | (null));
readonly "lifetime"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "lit"?: ((boolean) | (null));
readonly "looping"?: ((boolean) | (null));
readonly "maxParticles"?: ((number) | (null));
readonly "noise"?: (((InputDefinitions["ParticleNoise"])) | ((null)));
readonly "randomColorOverLife"?: (((InputDefinitions["ColorCurve"])) | ((null)));
readonly "rate"?: ((number) | (null));
readonly "rateOverTime"?: (((InputDefinitions["Curve"])) | ((null)));
readonly "render"?: (((InputDefinitions["ParticleRender"])) | ((null)));
readonly "rotation"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "roughness"?: ((number) | (null));
readonly "round"?: ((boolean) | (null));
readonly "shape"?: (((InputDefinitions["EmitterShape"])) | ((null)));
readonly "sheet"?: (((InputDefinitions["Sheet"])) | ((null)));
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "sizeOverLife"?: (((InputDefinitions["Curve"])) | ((null)));
readonly "softDistance"?: ((number) | (null));
readonly "space"?: (((InputDefinitions["Space"])) | ((null)));
readonly "speed"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "speedOverLife"?: (((InputDefinitions["Curve"])) | ((null)));
readonly "spinOverLife"?: (((InputDefinitions["Curve"])) | ((null)));
readonly "spread"?: ((number) | (null));
readonly "subEmitters"?: ((ReadonlyArray<(InputDefinitions["SubEmitter"])> | (number extends ((InputDefinitions["SubEmitter"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "texture"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "trail"?: (((InputDefinitions["ParticleTrail"])) | ((null)));
readonly "translucency"?: ((number) | (null));
readonly "wind"?: ((number) | (null));
});
export interface InputDefinitions {
readonly "Align": (((({
readonly "kind": ("camera") & (string);
})) & { readonly "axis"?: never; readonly "normal"?: never; readonly "stretch"?: never; }) | ((({
readonly "kind": ("velocity") & (string);
readonly "stretch"?: ((number) | (null));
})) & { readonly "axis"?: never; readonly "normal"?: never; }) | ((({
readonly "axis": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "kind": ("upright") & (string);
})) & { readonly "normal"?: never; readonly "stretch"?: never; }) | ((({
readonly "kind": ("facing") & (string);
readonly "normal": (ReadonlyArray<(number)> & { readonly length: 3 });
})) & { readonly "axis"?: never; readonly "stretch"?: never; }));
readonly "Attractor": ({
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "radius"?: ((number) | (null));
readonly "strength": (number);
});
readonly "Blend": ("alpha" | "additive" | "multiply" | "subtractive") & (string);
readonly "Burst": ({
readonly "count": (number);
readonly "cycles"?: ((number) | (null));
readonly "interval"?: ((number) | (null));
readonly "time": (number);
});
readonly "Collision": ({
readonly "against"?: (((InputDefinitions["CollisionTarget"])) | ((null)));
readonly "bounce"?: ((number) | (null));
readonly "die"?: ((boolean) | (null));
readonly "friction"?: ((number) | (null));
});
readonly "CollisionTarget": ("depth" | "colliders") & (string);
readonly "ColorCurve": (ReadonlyArray<(readonly [(number), (ReadonlyArray<(number)> & { readonly length: 4 })])> | (number extends ((readonly [(number), (ReadonlyArray<(number)> & { readonly length: 4 })])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Curve": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "EmitterShape": (((({
readonly "kind": ("point") & (string);
})) & { readonly "angleDegrees"?: never; readonly "closed"?: never; readonly "halfExtents"?: never; readonly "mesh"?: never; readonly "points"?: never; readonly "radius"?: never; readonly "surface"?: never; }) | ((({
readonly "kind": ("sphere") & (string);
readonly "radius": (number);
readonly "surface"?: ((boolean) | (null));
})) & { readonly "angleDegrees"?: never; readonly "closed"?: never; readonly "halfExtents"?: never; readonly "mesh"?: never; readonly "points"?: never; }) | ((({
readonly "angleDegrees": (number);
readonly "kind": ("cone") & (string);
readonly "radius": (number);
})) & { readonly "closed"?: never; readonly "halfExtents"?: never; readonly "mesh"?: never; readonly "points"?: never; readonly "surface"?: never; }) | ((({
readonly "halfExtents": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "kind": ("box") & (string);
})) & { readonly "angleDegrees"?: never; readonly "closed"?: never; readonly "mesh"?: never; readonly "points"?: never; readonly "radius"?: never; readonly "surface"?: never; }) | ((({
readonly "closed"?: ((boolean) | (null));
readonly "kind": ("spline") & (string);
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "angleDegrees"?: never; readonly "halfExtents"?: never; readonly "mesh"?: never; readonly "radius"?: never; readonly "surface"?: never; }) | ((({
readonly "kind": ("mesh") & (string);
readonly "mesh": (InputDefinitions["MeshInput"]);
})) & { readonly "angleDegrees"?: never; readonly "closed"?: never; readonly "halfExtents"?: never; readonly "points"?: never; readonly "radius"?: never; readonly "surface"?: never; }));
readonly "MeshInput": (((InputDefinitions["ParticleMesh"])) | ((InputDefinitions["ContentRef"])));
readonly "ParticleArray": ({
readonly "asset": (InputDefinitions["ContentRef"]);
readonly "layers"?: ((number) | (null));
readonly "mode"?: (((InputDefinitions["SheetMode"])) | ((null)));
});
readonly "ParticleEvent": ("birth" | "death" | "collision") & (string);
readonly "ParticleField": (((({
readonly "frequency"?: ((number) | (null));
readonly "kind": ("curl") & (string);
readonly "scroll"?: ((number) | (null));
readonly "strength": (number);
})) & { readonly "asset"?: never; readonly "cells"?: never; readonly "size"?: never; readonly "vectors"?: never; }) | ((({
readonly "asset": (InputDefinitions["ContentRef"]);
readonly "kind": ("asset") & (string);
readonly "size": (number);
readonly "strength": (number);
})) & { readonly "cells"?: never; readonly "frequency"?: never; readonly "scroll"?: never; readonly "vectors"?: never; }) | ((({
readonly "cells": (number);
readonly "kind": ("grid") & (string);
readonly "size": (number);
readonly "strength": (number);
readonly "vectors": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "asset"?: never; readonly "frequency"?: never; readonly "scroll"?: never; }));
readonly "ParticleMesh": ("box" | "sphere" | "tetrahedron") & (string);
readonly "ParticleNoise": ({
readonly "frequency"?: ((number) | (null));
readonly "scroll"?: ((number) | (null));
readonly "strength": (number);
});
readonly "ParticleRender": (((({
readonly "kind": ("billboard") & (string);
})) & { readonly "layer"?: never; readonly "mesh"?: never; }) | ((({
readonly "kind": ("sprite") & (string);
readonly "layer"?: ((number) | (null));
})) & { readonly "mesh"?: never; }) | ((({
readonly "kind": ("mesh") & (string);
readonly "mesh": (InputDefinitions["MeshInput"]);
})) & { readonly "layer"?: never; }));
readonly "ParticleTrail": ({
readonly "head"?: ((boolean) | (null));
readonly "lifetime": (number);
readonly "width"?: ((number) | (null));
});
readonly "Sheet": ({
readonly "blend"?: ((boolean) | (null));
readonly "columns": (number);
readonly "frames"?: ((number) | (null));
readonly "mode"?: (((InputDefinitions["SheetMode"])) | ((null)));
readonly "rows": (number);
});
readonly "SheetMode": (((({
readonly "kind": ("off") & (string);
})) & { readonly "cycles"?: never; readonly "fps"?: never; }) | ((({
readonly "kind": ("random") & (string);
})) & { readonly "cycles"?: never; readonly "fps"?: never; }) | ((({
readonly "cycles"?: ((number) | (null));
readonly "kind": ("life") & (string);
})) & { readonly "fps"?: never; }) | ((({
readonly "fps": (number);
readonly "kind": ("fps") & (string);
})) & { readonly "cycles"?: never; }));
readonly "Space": ("world" | "local") & (string);
readonly "SubEmitter": ({
/** Required for event triggers, omitted for life emission. */
readonly "count"?: ((number) | (null));
readonly "emitter": (InputDefinitions["ContentRef"]);
readonly "inheritVelocity"?: ((number) | (null));
readonly "trigger": (InputDefinitions["Trigger"]);
});
readonly "Trigger": ("birth" | "death" | "collision" | "life") & (string);
}
```
