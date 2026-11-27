<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-timeline"></a>
### `file:timeline`

Checked `file["timeline"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "duration"?: ((number) | (null));
readonly "name"?: ((string) | (null));
readonly "paths"?: (Readonly<Record<string, (InputDefinitions["Path"])>>);
readonly "tracks": (ReadonlyArray<(InputDefinitions["Track"])> | (number extends ((InputDefinitions["Track"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface InputDefinitions {
readonly "Animation": ({
readonly "blendIn"?: ((number) | (null));
readonly "clip": (string);
readonly "loop"?: ((boolean) | (null));
readonly "speed"?: ((number) | (null));
});
readonly "Audio": ({
readonly "bus"?: (((InputDefinitions["Bus"])) | ((null)));
readonly "playbackRate"?: ((number) | (null));
readonly "sound": (InputDefinitions["ContentRef"]);
readonly "volume"?: ((number) | (null));
});
readonly "Bus": ("master" | "music" | "sfx" | "ui") & (string);
readonly "Camera": ({
readonly "at"?: ((number) | (null));
readonly "focus"?: ((number) | (null));
readonly "focusOn"?: ((string) | (null));
readonly "focusRange"?: ((number) | (null));
readonly "fov"?: ((number) | (null));
readonly "position"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "roll"?: ((number) | (null));
readonly "target"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
});
readonly "Caption": ({
readonly "end"?: ((number) | (null));
readonly "position"?: (((InputDefinitions["CaptionPosition"])) | ((null)));
readonly "spans"?: (ReadonlyArray<(InputDefinitions["Span"])> | (number extends ((InputDefinitions["Span"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "speaker"?: ((string) | (null));
readonly "text": (string);
});
readonly "CaptionCues": ({
readonly "imports"?: (ReadonlyArray<(InputDefinitions["CaptionImport"])> | (number extends ((InputDefinitions["CaptionImport"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "keys"?: (ReadonlyArray<(InputDefinitions["Key6"])> | (number extends ((InputDefinitions["Key6"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "CaptionImport": ({
readonly "offset"?: ((number) | (null));
/** Explicit SRT, VTT or JSON cue file; cook captures it as a dependency. */
readonly "source": (string);
});
readonly "CaptionPosition": ("top" | "bottom") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Direction": ("left" | "right" | "top" | "bottom") & (string);
readonly "Event": ({
readonly "action": (string);
readonly "retroactive"?: ((boolean) | (null));
});
readonly "FieldValue": (((number)) | ((boolean)) | ((string)) | ((ReadonlyArray<(number)> & { readonly length: 2 })) | ((ReadonlyArray<(number)> & { readonly length: 3 })) | ((ReadonlyArray<(number)> & { readonly length: 4 })));
readonly "Hotspot": ({
readonly "id": (string);
readonly "kind"?: (((InputDefinitions["HotspotKind"])) | ((null)));
readonly "label"?: ((string) | (null));
readonly "off"?: ((boolean) | (null));
readonly "rect"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "shape"?: (((InputDefinitions["HotspotShape"])) | ((null)));
});
readonly "HotspotKind": ("choice" | "inspect") & (string);
readonly "HotspotShape": ("rect" | "circle") & (string);
readonly "Key": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Camera"]);
});
readonly "Key10": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["FieldValue"]);
});
readonly "Key11": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Light"]);
});
readonly "Key12": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::PostEffects"]);
});
readonly "Key13": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (boolean);
});
readonly "Key14": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::Blendshapes"]);
});
readonly "Key15": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Material"]);
});
readonly "Key16": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Nested"]);
});
readonly "Key17": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Shot"]);
});
readonly "Key18": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Vfx"]);
});
readonly "Key19": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Video"]);
});
readonly "Key2": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Transform"]);
});
readonly "Key20": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Hotspot"]);
});
readonly "Key21": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Scene"]);
});
readonly "Key3": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Animation"]);
});
readonly "Key4": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Audio"]);
});
readonly "Key5": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Music"]);
});
readonly "Key6": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Caption"]);
});
readonly "Key7": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Event"]);
});
readonly "Key8": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Transition"]);
});
readonly "Key9": ({
/** Shared easing syntax, including cubicBezier, steps and points. */
readonly "ease"?: ((string) | (null));
readonly "time": (number);
readonly "value": (InputDefinitions["Marker"]);
});
readonly "Light": ((((InputDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::DirectionalLight"])) & { readonly "angle"?: never; readonly "castShadows"?: never; readonly "cookie"?: never; readonly "distance"?: never; readonly "falloff"?: never; readonly "height"?: never; readonly "innerAngle"?: never; readonly "kind"?: never; readonly "layers"?: never; readonly "outerAngle"?: never; readonly "penumbra"?: never; readonly "radius"?: never; readonly "sourceRadius"?: never; readonly "twoSided"?: never; readonly "width"?: never; }) | (((InputDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::PointLight"])) & { readonly "angle"?: never; readonly "cascadeBlend"?: never; readonly "castShadows"?: never; readonly "falloff"?: never; readonly "height"?: never; readonly "innerAngle"?: never; readonly "kind"?: never; readonly "layers"?: never; readonly "outerAngle"?: never; readonly "penumbra"?: never; readonly "shadowBias"?: never; readonly "shadowBox"?: never; readonly "shadowDistance"?: never; readonly "shadowFocus"?: never; readonly "shadowFocusDrop"?: never; readonly "shadowNormalBias"?: never; readonly "shadowSoftness"?: never; readonly "sourceRadius"?: never; readonly "twoSided"?: never; readonly "width"?: never; }) | (((InputDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::SpotLight"])) & { readonly "cascadeBlend"?: never; readonly "castShadows"?: never; readonly "falloff"?: never; readonly "height"?: never; readonly "innerAngle"?: never; readonly "kind"?: never; readonly "layers"?: never; readonly "outerAngle"?: never; readonly "shadowBias"?: never; readonly "shadowBox"?: never; readonly "shadowDistance"?: never; readonly "shadowFocus"?: never; readonly "shadowFocusDrop"?: never; readonly "shadowNormalBias"?: never; readonly "shadowSoftness"?: never; readonly "sourceRadius"?: never; readonly "twoSided"?: never; readonly "width"?: never; }) | (((InputDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::RectLight"])) & { readonly "angle"?: never; readonly "cascadeBlend"?: never; readonly "castShadows"?: never; readonly "cookie"?: never; readonly "distance"?: never; readonly "falloff"?: never; readonly "innerAngle"?: never; readonly "kind"?: never; readonly "layers"?: never; readonly "outerAngle"?: never; readonly "penumbra"?: never; readonly "radius"?: never; readonly "shadowBias"?: never; readonly "shadowBox"?: never; readonly "shadowDistance"?: never; readonly "shadowFocus"?: never; readonly "shadowFocusDrop"?: never; readonly "shadowNormalBias"?: never; readonly "shadowSoftness"?: never; readonly "sourceRadius"?: never; }) | (((InputDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::Light2D"])) & { readonly "angle"?: never; readonly "cascadeBlend"?: never; readonly "castShadow"?: never; readonly "cookie"?: never; readonly "mobility"?: never; readonly "penumbra"?: never; readonly "radius"?: never; readonly "shadowBias"?: never; readonly "shadowBox"?: never; readonly "shadowDistance"?: never; readonly "shadowFocus"?: never; readonly "shadowFocusDrop"?: never; readonly "shadowNormalBias"?: never; readonly "shadowSoftness"?: never; readonly "twoSided"?: never; readonly "width"?: never; }));
readonly "Marker": ({
readonly "name": (string);
});
readonly "Material": ({
readonly "mapOffset"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "mapRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "mapRotation"?: ((number) | (null));
/** Explicit shader uniforms, not an escape hatch for misspelled standard
fields. Matching the target shader's names/dimensions is a graph check. */
readonly "params"?: (({
readonly "alphaTest"?: never;
readonly "anisotropy"?: never;
readonly "anisotropyRotation"?: never;
readonly "aoMapIntensity"?: never;
readonly "attenuationColor"?: never;
readonly "attenuationDistance"?: never;
readonly "clearcoat"?: never;
readonly "clearcoatRoughness"?: never;
readonly "color"?: never;
readonly "dissolve"?: never;
readonly "dissolveEdgeColor"?: never;
readonly "dissolveEdgeWidth"?: never;
readonly "dissolveScale"?: never;
readonly "emissive"?: never;
readonly "emissiveIntensity"?: never;
readonly "envMapIntensity"?: never;
readonly "flash"?: never;
readonly "flashColor"?: never;
readonly "ior"?: never;
readonly "iridescence"?: never;
readonly "mapOffset"?: never;
readonly "mapRepeat"?: never;
readonly "mapRotation"?: never;
readonly "metalness"?: never;
readonly "normalScale"?: never;
readonly "opacity"?: never;
readonly "outlineColor"?: never;
readonly "outlineWidth"?: never;
readonly "rimColor"?: never;
readonly "rimIntensity"?: never;
readonly "rimPower"?: never;
readonly "roughness"?: never;
readonly "sheen"?: never;
readonly "sheenColor"?: never;
readonly "sheenRoughness"?: never;
readonly "thickness"?: never;
readonly "transmission"?: never;
readonly "transparent"?: never;
}) & Readonly<Record<string, (InputDefinitions["Uniform"])>>);
/** Only reflected fields actually driven by timeline presentation. */
readonly "standard"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::media::timeline::StandardMaterial"])) | ((null)));
});
readonly "Music": ({
readonly "bus"?: (((InputDefinitions["Bus"])) | ((null)));
readonly "end"?: ((number) | (null));
readonly "fadeIn"?: ((number) | (null));
readonly "fadeOut"?: ((number) | (null));
readonly "loop"?: ((boolean) | (null));
readonly "sound": (InputDefinitions["ContentRef"]);
readonly "volume"?: ((number) | (null));
});
readonly "Nested": ({
readonly "loop"?: ((boolean) | (null));
readonly "play": (InputDefinitions["ContentRef"]);
readonly "speed"?: ((number) | (null));
});
readonly "Path": ({
readonly "closed"?: ((boolean) | (null));
readonly "kind"?: (((InputDefinitions["PathKind"])) | ((null)));
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "tension"?: ((number) | (null));
});
readonly "PathKind": ("catmullRom" | "bezier" | "linear") & (string);
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::Blendshapes": ({
/** Weights of targets 8 to 63, as three.js morphTargetInfluences past the first eight (w0...w7): influences[k] drives target 8 + k; past the list's length 0. A face rig's 52 ARKit shapes fit. Clips write it with `source` clips, as they write w0...w7. */
readonly "influences"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Weight of target 0. */
readonly "w0"?: (number);
/** Weight of target 1. */
readonly "w1"?: (number);
/** Weight of target 2. */
readonly "w2"?: (number);
/** Weight of target 3. */
readonly "w3"?: (number);
/** Weight of target 4. */
readonly "w4"?: (number);
/** Weight of target 5. */
readonly "w5"?: (number);
/** Weight of target 6. */
readonly "w6"?: (number);
/** Weight of target 7. */
readonly "w7"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::DirectionalLight": ({
/** The fraction of each shadow cascade's depth over which it fades into the next, hiding the seam where the shadow's resolution changes. 0 is none; 0.1 to 0.2 is typical (those pixels read two cascades). */
readonly "cascadeBlend"?: (number);
/** Cast shadows through the cascades (RenderSettings.shadowCascades), as three.js Light.castShadow. */
readonly "castShadow"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The radiance multiplier, as three.js DirectionalLight.intensity. */
readonly "intensity"?: (number);
/** How the light may change while the game runs: what og bake lightmaps bakes of it and what is drawn every frame (stationary, the default; movable; static). */
readonly "mobility"?: ("movable" | "stationary" | "static") & (string);
/** With shadowBox: the depth bias toward the light (three.js shadow.bias times its camera's depth range). 0 is one texel. Leaf cards need a few centimetres or they shadow each other dark. */
readonly "shadowBias"?: (number);
/** One shadow map over a fixed box instead of the cascades: its half size across the light. 0 keeps the cascades. Everything outside the box is lit; the casters are drawn once, not once per cascade (three.js games that fit the light's orthographic shadow camera around the player draw this way). */
readonly "shadowBox"?: (number);
/** How far from the camera shadows reach; they fade over the last tenth. Smaller is sharper. */
readonly "shadowDistance"?: (number);
/** With shadowBox: how far ahead of the camera (along its level forward) the box is centred. */
readonly "shadowFocus"?: (number);
/** With shadowBox: how far below the camera the box's centre sits (the ground under a first-person eye). */
readonly "shadowFocusDrop"?: (number);
/** With shadowBox: how far a receiver is pushed along its normal before it reads the map (three.js shadow.normalBias). 0 is one texel. */
readonly "shadowNormalBias"?: (number);
/** Soft shadows (PCSS): the penumbra's width per metre between caster and receiver, the tangent of the sun's apparent radius (the real sun's is 0.005). 0 keeps the hard one-texel edge; 0.01 to 0.05 is a soft sun, sharp at contact and blurring with distance. */
readonly "shadowSoftness"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::Light2D": ({
/** Occluders between the light and a pixel shade it (point and spot lights). */
readonly "castShadows"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Where the light reaches zero, in world units. */
readonly "distance"?: (number);
/** The exponent of the fade. */
readonly "falloff"?: (number);
/** How far above the 2D plane the light is, in world units: the slant normal-mapped sprites see it at (0 grazes the surface). */
readonly "height"?: (number);
/** A spot's half-angle at full intensity. */
readonly "innerAngle"?: (number);
/** The brightness at the centre. */
readonly "intensity"?: (number);
/** Point, spot or global. */
readonly "kind"?: ("point" | "spot" | "global") & (string);
/** The light layers it lights: a bit per layer, matched against each sprite's, rig's or tilemap's og.LightMask2D (layer 0, bit 1, without one). */
readonly "layers"?: (number);
/** A spot's half-angle where it reaches zero (at least innerAngle). */
readonly "outerAngle"?: (number);
/** The light's own size: larger gives softer shadows. */
readonly "sourceRadius"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::PointLight": ({
/** Cast shadows: six cube faces or two paraboloid hemispheres (RenderOptions.pointShadows). */
readonly "castShadow"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** A light cookie projected all around the light: six square faces stacked top to bottom as +X, -X, +Y, -Y, +Z, -Z, six times as tall as wide (a KTX2 cube's face order, a cube map's orientation, world axes). The light's colour is multiplied by the texel its direction meets. 0 is none. */
readonly "cookie"?: ({
readonly "$ogRef": (string);
});
/** Where the light reaches zero, as three.js PointLight.distance (never infinite here). */
readonly "distance"?: (number);
/** Luminous intensity: the radiance at one metre for a zero-radius source, before the distance fade. */
readonly "intensity"?: (number);
/** How the light may change while the game runs: what og bake lightmaps bakes of it and what is drawn every frame (stationary, the default; movable; static). */
readonly "mobility"?: ("movable" | "stationary" | "static") & (string);
/** The spherical source radius: softens near-source falloff and widens specular highlights. Zero keeps a point source; independent of distance and shadow softness. */
readonly "radius"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::PostEffects": ({
/** Screen-space ambient occlusion: darkens the ambient light in creases and contact areas. */
readonly "ao"?: (boolean);
/** How much the occlusion also darkens direct and local light (0 only the ambient light, 1 a fully occluded pixel keeps 35 % of it). Above 0 the AO sees and darkens water surfaces too; at 0 water is outside it. */
readonly "aoDirect"?: (number);
/** 0 none, 1 full; above 1 exaggerates. */
readonly "aoIntensity"?: (number);
/** The algorithm. */
readonly "aoMethod"?: ("auto" | "gtao" | "ssao" | "screen") & (string);
/** The world-space radius searched for occluders. */
readonly "aoRadius"?: (number);
/** Above 0 the radius grows with the distance, min(aoRadius, aoRadiusScale × distance), a fixed share of the screen up close as three.js SSAO passes size theirs; 0 keeps aoRadius everywhere. */
readonly "aoRadiusScale"?: (number);
/** Half (the default) searches and blurs at half resolution with a normal- and depth-aware upsample; full searches every pixel. QualityPreset.high sets RenderSettings.aoResolution to full. */
readonly "aoResolution"?: ("full" | "half") & (string);
/** The colour balance of the highlights (luminance above 0.5, fading in from it), as balanceShadows. */
readonly "balanceHighlights"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The colour balance of the mid-tones (what the shadows and highlights leave), as balanceShadows. */
readonly "balanceMidtones"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The colour balance of the shadows (luminance below 0.5, fading out towards it): x shifts from cyan to red, y from magenta to green, z from yellow to blue, each -1 to 1 added to its channel; zero is off. */
readonly "balanceShadows"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Added to every channel of the display colour; 0 is off. */
readonly "brightness"?: (number);
/** Red and blue split towards the edges, in pixels at the corners at 720p; 0 is off. */
readonly "chromaticAberration"?: (number);
/** Contrast around mid grey: positive spreads the tones by 1 / (1 - contrast), negative pulls them in by 1 + contrast (-1 is flat grey); 0 is off. */
readonly "contrast"?: (number);
/** Custom post passes in order (og.load.post), at most RenderCapabilities.maxCustomPosts (8), each over the one before, after depth of field and motion blur, before exposure and bloom, reading depth and normals (the prepass). A 0 is skipped, as is one failing to compile (og.render.postErrors). */
readonly "custom"?: (ReadonlyArray<({
readonly "$ogRef": (string);
})> | (number extends (({
readonly "$ogRef": (string);
})) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Bokeh depth of field: pixels away from the focus distance blur into discs. */
readonly "depthOfField"?: (boolean);
/** How quickly blur grows outside the band: full blur focusRange times (1 + falloff) away. */
readonly "dofFalloff"?: (number);
/** The distance from the camera that is sharp. */
readonly "dofFocusDistance"?: (number);
/** The half-width of the sharp band around it. */
readonly "dofFocusRange"?: (number);
/** The largest blur radius, as a fraction of the image height (0.012 is about 9 px at 720p). */
readonly "dofMaxBlur"?: (number);
/** Taps per pixel. */
readonly "dofQuality"?: ("low" | "high") & (string);
/** Tilt-shift: above 0 the sharp band is a strip across the image's middle, this fraction of its height either side of the centre line, at any camera distance; blur grows above it as far and below it as near, full at dofTiltShift times (1 + dofFalloff) past it. 0 focuses by depth. */
readonly "dofTiltShift"?: (number);
/** Luminance-weighted noise amplitude in display units, a new pattern every tick (0.04 is subtle); 0 is off. */
readonly "filmGrain"?: (number);
/** The grain size in pixels. */
readonly "filmGrainSize"?: (number);
/** A full-screen flash: the display colour mixed towards flashColor by this much (animate it per tick); 0 is off. */
readonly "flash"?: (number);
/** The flash's linear display colour. */
readonly "flashColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Order-independent 2D glow: sprites with SpriteDesc.glow (and 2D particles with a glow) are blurred and added back, with or without bloom. */
readonly "glow"?: (boolean);
/** How much of the blurred glow is added back. */
readonly "glowIntensity"?: (number);
/** Screen-space light shafts from the DirectionalLight (the sun must be in front of the camera). The sun casts them with its own light, as a source of a fixed angular size whatever size its disc is drawn, dimmed by the clouds over it; the bright sky around it adds its own. */
readonly "godRays"?: (boolean);
/** How fast the rays fade along their length. */
readonly "godRaysDecay"?: (number);
/** How far towards the sun each ray samples (a fraction of the distance). */
readonly "godRaysDensity"?: (number);
/** How bright the shafts are. */
readonly "godRaysIntensity"?: (number);
/** Taps per ray. */
readonly "godRaysSamples"?: (number);
/** Brightness (linear, before exposure) below which the sky and the sun carry no rays: only what is brighter casts shafts. */
readonly "godRaysThreshold"?: (number);
/** A linear tint on the light the rays carry. */
readonly "godRaysTint"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** How far each colour goes towards its luminance (grey); 0 is off. */
readonly "grayscale"?: (number);
/** Turns every hue about the grey axis (red towards green at 120); 0 is off. */
readonly "hue"?: (number);
/** How far each colour goes towards its complement; 0 is off. */
readonly "invert"?: (number);
/** Barrel (positive: lines bow out) or pincushion (negative: in) distortion; 0 is off. Radius in pixels at any aspect; a barrel keeps its farthest corner on the image. After the depth-reading Stylise passes, before palette quantisation; the UI is not bent, pointer raycasts see the undistorted frame. */
readonly "lensDistortion"?: (number);
/** The lens's centre, in fractions of the frame from the top left. */
readonly "lensDistortionCenter"?: (ReadonlyArray<(number)> & { readonly length: 2 });
/** Colour grading by a lookup table from og.load.lut; 0 is none (and so is a LUT whose streaming group is not resident). */
readonly "lut"?: ({
readonly "$ogRef": (string);
});
/** 0 leaves the image as tone mapped, 1 applies the table fully. */
readonly "lutIntensity"?: (number);
/** Per-object and camera motion blur along the screen-space velocity. */
readonly "motionBlur"?: (boolean);
/** The longest blur in pixels. */
readonly "motionBlurMaxPixels"?: (number);
/** Taps along the velocity. */
readonly "motionBlurSamples"?: (number);
/** The shutter as a fraction of the frame (0.5 is a 180 degree shutter). */
readonly "motionBlurShutter"?: (number);
/** Outlines every opaque surface from depth and normal edges, creases between the faces of one object included (materials with outlineWidth get their own hull outline instead). */
readonly "outline"?: (boolean);
/** The line colour, linear. */
readonly "outlineColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The relative depth step that counts as a silhouette (0.04 is 4 %). */
readonly "outlineDepthThreshold"?: (number);
/** The normal change that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees). */
readonly "outlineNormalThreshold"?: (number);
/** Line width in pixels, drawn inside the silhouette. */
readonly "outlineWidth"?: (number);
/** The colour balance keeps each pixel's luminance, so it moves only the hue (Photoshop's Preserve Luminosity). */
readonly "preserveLuminosity"?: (boolean);
/** -1 grey, 0 unchanged, 1 twice the colour (the chroma around the luminance scaled by 1 + saturation). */
readonly "saturation"?: (number);
/** A zoom, spin or directional blur of the whole frame (RedGPU's zoom, radial and directional blurs), weighted to the middle of its reach: one pass after motion blur and before bloom and the tone curve, so highlights streak and glow; the UI is not blurred. Off at none or at screenBlurStrength 0. */
readonly "screenBlur"?: ("none" | "zoom" | "spin" | "directional") & (string);
/** Directional's direction, counter-clockwise from the right (90 is vertical). */
readonly "screenBlurAngle"?: (number);
/** Zoom's and spin's centre, in fractions of the frame from the top left. */
readonly "screenBlurCenter"?: (ReadonlyArray<(number)> & { readonly length: 2 });
/** Taps per pixel for spin and directional. Zoom exceeds this quality floor with three eight-tap stages (512 effective positions), or direct adaptive sampling when strength >= 1 crosses the centre. Both use stable per-pixel jitter. */
readonly "screenBlurSamples"?: (number);
/** How far it reaches each way: for zoom the fraction of the way to the centre, for spin radians, for directional pixels at 720 lines; 0 is off. */
readonly "screenBlurStrength"?: (number);
/** How fast the shake moves. */
readonly "shakeFrequency"?: (number);
/** The largest offset at trauma 1, as a fraction of the view height. */
readonly "shakeMaxOffset"?: (number);
/** The largest roll at trauma 1. */
readonly "shakeMaxRoll"?: (number);
/** The shake's noise seed. */
readonly "shakeSeed"?: (number);
/** Camera shake held at a fixed level (decay it yourself): 0 none, 1 the most (the amplitude is trauma squared). Prefer og.camera.shake, which decays by itself; the frame shakes by the larger of the two. Smooth value noise by the tick, so a replay shakes the same. Visual only. */
readonly "shakeTrauma"?: (number);
/** Contrast-adaptive sharpening (CAS) of the final image, as a three.js CAS pass after the output pass: each pixel pushed away from its four neighbours by a weight that falls where the neighbourhood is already near black or white. 0 is the mildest, 1 the strongest; below 0 is off. */
readonly "sharpen"?: (number);
/** Stochastic, denoised screen-space reflections (after FidelityFX SSSR and Frostbite): a GGX-lobe ray a pixel up to ssrMaxRoughness through a depth pyramid, reflecting the previous frame's lit image; on a miss, the probes', probe GI's and environment's. Opaque, perspective 3D; turns on the prepass. */
readonly "ssr"?: (boolean);
/** The fraction of the screen's edge over which a reflection fades into the probes' (the last fifth of ssrMaxDistance fades too). */
readonly "ssrFade"?: (number);
/** How much of the reflection replaces the probes' and the environment's where it resolved; 1 is physical (weighted by the surface's Fresnel and lobe as the probes' are). */
readonly "ssrIntensity"?: (number);
/** How far a reflected ray marches, in world units; the last fifth fades into the probes'. The march's cost is set by ssrSteps, not by the distance. */
readonly "ssrMaxDistance"?: (number);
/** The roughest surface traced: rougher ones keep the probes' reflection, and the last quarter below it fades into them. */
readonly "ssrMaxRoughness"?: (number);
/** Half (the default): one ray per 2x2 block, a different pixel of the block each frame, denoised on the blocks and upsampled to the pixels by their depth, normal and roughness. Full: one ray per pixel, denoised at full resolution. */
readonly "ssrResolution"?: ("full" | "half") & (string);
/** The hierarchical march's iterations along each ray, each crossing a cell of the min-max depth pyramid's current level; a ray out of steps keeps the probes' reflection. Only the rays that need the steps pay for them. */
readonly "ssrSteps"?: (number);
/** How far behind the depth buffer a surface is taken to extend: a ray passing behind a surface by less than this hits it. */
readonly "ssrThickness"?: (number);
/** White balance: towards warm orange (positive) or cool blue (negative), Unity's Temperature divided by 100 (a von Kries scaling in LMS from a white point moved along the daylight locus); 0 is off. */
readonly "temperature"?: (number);
/** Above 0, each pixel white where its luminance reaches this level and black below it (the table's 32 steps soften the edge over about 1/31 of the range); 0 is off. */
readonly "threshold"?: (number);
/** White balance: towards magenta (positive) or green (negative), as Lightroom's and Unity's Tint divided by 100; 0 is off. */
readonly "tint"?: (number);
/** Saturation that spares colours already saturated and skin tones: positive livens dull colours, negative mutes them; 0 is off. */
readonly "vibrance"?: (number);
/** Darkened (or tinted) edges of the frame. */
readonly "vignette"?: (boolean);
/** The linear colour the edges move towards. */
readonly "vignetteColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** 0 none, 1 black corners. */
readonly "vignetteIntensity"?: (number);
/** Where darkening starts, as a fraction of the half-diagonal. */
readonly "vignetteRadius"?: (number);
/** The width of the fade. */
readonly "vignetteSmoothness"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::RectLight": ({
/** Cast a soft shadow: a shadow map from the rectangle's centre (two when twoSided, counted in the shadowed lights' budget) read with percentage-closer soft shadows, sharp at contact and widening with the rectangle's size away from it. */
readonly "castShadow"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Its size along the entity's local y, as RectAreaLight.height. */
readonly "height"?: (number);
/** The emitted radiance (luminance), as three.js RectAreaLight.intensity. */
readonly "intensity"?: (number);
/** How the light may change while the game runs: what og bake lightmaps bakes of it and what is drawn every frame (stationary, the default; movable; static). */
readonly "mobility"?: ("movable" | "stationary" | "static") & (string);
/** Emit from the back face as well (a hanging panel lit both ways). */
readonly "twoSided"?: (boolean);
/** Its size along the entity's local x, as RectAreaLight.width. */
readonly "width"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::SpotLight": ({
/** The half-angle of the outer cone, as three.js SpotLight.angle. */
readonly "angle"?: (number);
/** Cast a shadow map. */
readonly "castShadow"?: (boolean);
/** The light's linear colour. */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** A light cookie, as three.js SpotLight.map: the texture projected along the cone (its centre on the axis, its edges at the outer angle, its top towards world up), multiplying the light's colour. 0 is none. A shadowed spot keeps its shadow over the pattern. */
readonly "cookie"?: ({
readonly "$ogRef": (string);
});
/** Where the light reaches zero. */
readonly "distance"?: (number);
/** Luminous intensity on the axis: the radiance at one metre for a zero-radius source, before the distance fade. */
readonly "intensity"?: (number);
/** How the light may change while the game runs: what og bake lightmaps bakes of it and what is drawn every frame (stationary, the default; movable; static). */
readonly "mobility"?: ("movable" | "stationary" | "static") & (string);
/** The fraction of the cone over which the light fades out, as three.js SpotLight.penumbra. */
readonly "penumbra"?: (number);
/** The spherical source radius: softens near-source falloff and widens specular highlights. Zero keeps a point source; independent of distance and shadow softness. */
readonly "radius"?: (number);
});
/** A physically based metallic-roughness material with three.js
`MeshStandardMaterial` parameter names, reflected: its schema
is what the inspector, Tune and the probe edit,
and its fields match `render.StandardMaterialDesc` in the IDL.

The `MeshPhysicalMaterial` fields (clearcoat maps, sheen, transmission,
iridescence, anisotropy, `ior`) draw through the physical lit pipelines
([`StandardMaterial::is_physical`]); a material without them draws
exactly as a `MeshStandardMaterial`.

Texture fields are slots: `0` is none, otherwise
[`TextureHandle::slot`] (the texture's index plus one). Boolean fields
are `0` or `1`. Colours are linear. The default is three.js's: white,
metalness 0, roughness 1, opaque, front side. */
readonly "Reflected_og_cook::cook::source_schemas::media::timeline::StandardMaterial": ({
/** Pixels whose alpha is below this are discarded (0: none). */
readonly "alphaTest"?: (number);
/** Stretches the specular highlight along a direction of the surface
(three.js `anisotropy`, glTF `KHR_materials_anisotropy`): brushed
metal, hair, satin. 0: none, 1: fully stretched. */
readonly "anisotropy"?: (number);
/** The stretch's direction: radians from the tangent (+u) towards the bitangent. */
readonly "anisotropyRotation"?: (number);
/**  */
readonly "aoMapIntensity"?: (number);
/** The colour white light becomes after `attenuationDistance` in the volume (linear RGB). */
readonly "attenuationColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The distance light travels in the volume before it is tinted to
`attenuationColor` (world units). 0: no absorption (three.js's Infinity). */
readonly "attenuationDistance"?: (number);
/** A clear lacquer over the surface (three.js `MeshPhysicalMaterial`
`clearcoat`): a second specular layer, 4 % reflective, on the
surface's normal before the normal map. 0: none. */
readonly "clearcoat"?: (number);
/** How rough the coat is. */
readonly "clearcoatRoughness"?: (number);
/** The base colour (linear RGB), multiplied with `map` and vertex colours. */
readonly "color"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The dissolved fraction: 0 whole, 1 gone. Pixels whose noise is below
it are discarded (also from shadows and the prepass); the band just
above glows with `dissolveEdgeColor`. */
readonly "dissolve"?: (number);
/** The glowing edge (linear RGB, usually above 1 so it blooms). */
readonly "dissolveEdgeColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Width of the glowing band in noise units. */
readonly "dissolveEdgeWidth"?: (number);
/** Frequency of the built-in noise (cells per object-space unit). */
readonly "dissolveScale"?: (number);
/** Emitted light (linear RGB), multiplied with `emissiveMap`. */
readonly "emissive"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** A multiplier on the emitted light. */
readonly "emissiveIntensity"?: (number);
/** A multiplier on the image-based (environment) light. */
readonly "envMapIntensity"?: (number);
/** How far the final colour is mixed towards `flashColor` (0 none, 1 flat). */
readonly "flash"?: (number);
/** The colour a hit flash mixes towards (linear RGB, may exceed 1). */
readonly "flashColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The index of refraction (glTF `KHR_materials_ior`): the dielectric
reflectance ((ior - 1) / (ior + 1))², 4 % at the default 1.5, and
how far transmission refracts. */
readonly "ior"?: (number);
/** A thin-film interference layer (three.js `iridescence`, glTF
`KHR_materials_iridescence`): soap bubbles, oil, beetle shells. 0: none. */
readonly "iridescence"?: (number);
/** How metallic the surface is. */
readonly "metalness"?: (number);
/** How strongly the normal map bends the normal, per tangent axis. */
readonly "normalScale"?: (ReadonlyArray<(number)> & { readonly length: 2 });
/** The opacity; below 1 only shows with `transparent`. */
readonly "opacity"?: (number);
/** This object's outline colour (linear RGB) in the outline pass. */
readonly "outlineColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** This object's outline width in pixels (0: the scene's
`PostEffects::outline`, if any). Turns the outline pass on. */
readonly "outlineWidth"?: (number);
/** Rim (fresnel) light added at grazing angles, unshadowed (linear RGB). */
readonly "rimColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** 0: no rim light. */
readonly "rimIntensity"?: (number);
/** How tight the rim is (higher: thinner). */
readonly "rimPower"?: (number);
/** How rough the surface is. */
readonly "roughness"?: (number);
/** A velvet-like retroreflective layer (three.js `sheen`, glTF
`KHR_materials_sheen`): its strength times `sheenColor`. 0: none. */
readonly "sheen"?: (number);
/** The sheen's colour (linear RGB), multiplied with `sheenColorMap`. */
readonly "sheenColor"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** How rough the sheen is: low is a thin rim, high spreads it over the surface. */
readonly "sheenRoughness"?: (number);
/** The volume's thickness under the surface in object-space units
(glTF `KHR_materials_volume`): how far refraction bends the view
and how much `attenuationColor` absorbs. 0: a thin wall. */
readonly "thickness"?: (number);
/** How much light passes through the surface (three.js `transmission`,
glTF `KHR_materials_transmission`): glass, water, thin plastic. The
opaque scene behind it is refracted, blurred by `roughness`. 0: none. */
readonly "transmission"?: (number);
/** 1: blended over the opaque scene, sorted back to front, no depth write. */
readonly "transparent"?: (number);
});
readonly "Scene": ({
/** Null clears the scene at this key. */
readonly "scene"?: (((InputDefinitions["ContentRef"])) | ((null)));
});
readonly "Shot": ({
readonly "blend"?: ((number) | (null));
readonly "camera"?: ((string) | (null));
});
readonly "Span": ({
readonly "color"?: ((string) | (null));
readonly "length": (number);
readonly "name": (string);
readonly "position": (number);
});
readonly "Track": (((({
readonly "blendIn"?: ((number) | (null));
readonly "blendOut"?: ((number) | (null));
readonly "follow"?: ((string) | (null));
readonly "keys": (ReadonlyArray<(InputDefinitions["Key"])> | (number extends ((InputDefinitions["Key"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("camera") & (string);
readonly "rail"?: ((string) | (null));
})) & { readonly "entity"?: never; readonly "field"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(InputDefinitions["Key2"])> | (number extends ((InputDefinitions["Key2"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("transform") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(InputDefinitions["Key3"])> | (number extends ((InputDefinitions["Key3"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("animation") & (string);
readonly "model"?: (((InputDefinitions["ContentRef"])) | ((null)));
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key4"])> | (number extends ((InputDefinitions["Key4"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("audio") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key5"])> | (number extends ((InputDefinitions["Key5"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("music") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "imports"?: (ReadonlyArray<(InputDefinitions["CaptionImport"])> | (number extends ((InputDefinitions["CaptionImport"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "keys"?: (ReadonlyArray<(InputDefinitions["Key6"])> | (number extends ((InputDefinitions["Key6"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("caption") & (string);
readonly "locales"?: ((Readonly<Record<string, (InputDefinitions["CaptionCues"])>>) | (null));
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key7"])> | (number extends ((InputDefinitions["Key7"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("event") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key8"])> | (number extends ((InputDefinitions["Key8"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("transition") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key9"])> | (number extends ((InputDefinitions["Key9"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("marker") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity"?: ((string) | (null));
readonly "field": (string);
readonly "keys": (ReadonlyArray<(InputDefinitions["Key10"])> | (number extends ((InputDefinitions["Key10"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("property") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(InputDefinitions["Key11"])> | (number extends ((InputDefinitions["Key11"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("light") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key12"])> | (number extends ((InputDefinitions["Key12"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("post") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(InputDefinitions["Key13"])> | (number extends ((InputDefinitions["Key13"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("visibility") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "keys": (ReadonlyArray<(InputDefinitions["Key14"])> | (number extends ((InputDefinitions["Key14"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("blendshape") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity": (string);
readonly "index"?: ((number) | (null));
readonly "keys": (ReadonlyArray<(InputDefinitions["Key15"])> | (number extends ((InputDefinitions["Key15"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("material") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key16"])> | (number extends ((InputDefinitions["Key16"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("timeline") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key17"])> | (number extends ((InputDefinitions["Key17"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("shot") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key18"])> | (number extends ((InputDefinitions["Key18"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("vfx") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "entity"?: ((string) | (null));
readonly "keys": (ReadonlyArray<(InputDefinitions["Key19"])> | (number extends ((InputDefinitions["Key19"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("video") & (string);
readonly "layer"?: (((InputDefinitions["VideoLayer"])) | ((null)));
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key20"])> | (number extends ((InputDefinitions["Key20"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("hotspot") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }) | ((({
readonly "keys": (ReadonlyArray<(InputDefinitions["Key21"])> | (number extends ((InputDefinitions["Key21"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("scene") & (string);
})) & { readonly "blendIn"?: never; readonly "blendOut"?: never; readonly "entity"?: never; readonly "field"?: never; readonly "follow"?: never; readonly "imports"?: never; readonly "index"?: never; readonly "layer"?: never; readonly "locales"?: never; readonly "model"?: never; readonly "rail"?: never; }));
readonly "Transform": ({
readonly "position"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "quaternion"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
/** Euler XYZ degrees; exclusive with quaternion (domain check). */
readonly "rotation"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "scale"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
});
readonly "Transition": ({
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "duration"?: ((number) | (null));
readonly "from"?: (((InputDefinitions["Direction"])) | ((null)));
readonly "kind"?: (((InputDefinitions["TransitionKind"])) | ((null)));
readonly "mode"?: (((InputDefinitions["TransitionMode"])) | ((null)));
});
readonly "TransitionKind": ("cut" | "fade" | "wipe" | "dissolve" | "iris" | "slide") & (string);
readonly "TransitionMode": ("through" | "in" | "out") & (string);
readonly "Uniform": (((number)) | ((ReadonlyArray<(number)> & { readonly length: 2 })) | ((ReadonlyArray<(number)> & { readonly length: 3 })) | ((ReadonlyArray<(number)> & { readonly length: 4 })));
readonly "Vfx": ({
readonly "at"?: ((string) | (null));
readonly "effect": (InputDefinitions["ContentRef"]);
readonly "position"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "scale"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
});
readonly "Video": ({
readonly "from"?: ((number) | (null));
readonly "speed"?: ((number) | (null));
readonly "video": (InputDefinitions["ContentRef"]);
});
readonly "VideoLayer": ("over" | "under") & (string);
}
```
