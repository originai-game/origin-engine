<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-look"></a>
### `file:look`

Checked `file["look"](input)` declaration from `@origingame/engine/author`.

```ts
/** A look overlays reflected resource patches on explicitly named bases.
Inheritance order/cycles, preset variants and asset identities are domain checks. */
export type Input = ({
readonly "ArtCamera"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtCamera"])) | ((null)));
readonly "ArtLine"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtLine"])) | ((null)));
readonly "ArtMotion"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtMotion"])) | ((null)));
readonly "ArtPalette"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtPalette"])) | ((null)));
readonly "ArtPixels"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtPixels"])) | ((null)));
readonly "ArtShape"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtShape"])) | ((null)));
readonly "ArtTexels"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::ArtTexels"])) | ((null)));
readonly "Environment"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::Environment"])) | ((null)));
readonly "Fog"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::Fog"])) | ((null)));
readonly "MaterialStyle"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::MaterialStyle"])) | ((null)));
readonly "PostEffects"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::PostEffects"])) | ((null)));
readonly "RenderSettings"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::RenderSettings"])) | ((null)));
readonly "Sky"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::Sky"])) | ((null)));
readonly "Stylise"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::Stylise"])) | ((null)));
readonly "antialias"?: (((InputDefinitions["Antialias"])) | ((null)));
readonly "extends"?: ((ReadonlyArray<(InputDefinitions["LookBase"])> | (number extends ((InputDefinitions["LookBase"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "mood"?: (((InputDefinitions["Mood"])) | ((null)));
readonly "name"?: ((string) | (null));
readonly "ui"?: (((InputDefinitions["UiStyle"])) | ((null)));
readonly "uiSeeds"?: (((InputDefinitions["UiSeeds"])) | ((null)));
readonly "variant"?: ((string) | (null));
});
export interface InputDefinitions {
readonly "Antialias": ("tier" | "off" | "msaa4" | "taa") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "HexColor": (string);
readonly "LookBase": (((string)) | ((InputDefinitions["ContentRef"])));
readonly "Mood": ("bright" | "dark" | "warm" | "cold" | "soft" | "high-contrast") & (string);
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtCamera": ({
/** The camera keeps one angle, pitch and yaw: false is no constraint. */
readonly "fixedAngle"?: (boolean);
/** The widest vertical field of view a perspective camera of the style takes; 0 is no limit. */
readonly "fovMax"?: (number);
/** The narrowest vertical field of view a perspective camera of the style takes; 0 is no limit. */
readonly "fovMin"?: (number);
/** With fixedAngle: how far below the horizon the camera looks (0 side-on, 90 straight down, 30 the 2:1 pixel isometric, 35.264 true isometric). */
readonly "pitch"?: (number);
/** A 2D camera moves and zooms by whole logical pixels only (ArtPixels.unit), as Stylise.pixelSnap does the frame's: pixel art never shimmers as it scrolls. */
readonly "pixelPerfect"?: (boolean);
/** The projection: any (no constraint), perspective or orthographic. */
readonly "projection"?: ("any" | "perspective" | "orthographic") & (string);
/** With fixedAngle: its turn about the vertical axis (45 for isometric). */
readonly "yaw"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtLine": ({
/** Round a sprite: the brush the cook draws the line into its cells with: diamond (the zero value), circle or square (docs/art.md §2.3). */
readonly "brush"?: ("diamond" | "circle" | "square") & (string);
/** The line's colour (linear; a look may write an sRGB "#rrggbb"). */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Round a mesh: the distance at which the line is width wide, nearer wider and farther thinner (as MaterialStyle.outlineDistance); 0 keeps one width. */
readonly "distanceFalloff"?: (number);
/** Where the line lies against the shape: outside (the zero value), inside, or both. */
readonly "place"?: ("outside" | "inside" | "both") & (string);
/** The line's width: in logical pixels round a sprite (1 is one dark pixel round the shape), in pixels at distanceFalloff round a mesh (as MaterialStyle.outlineWidth); 0 is no line. */
readonly "width"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtMotion": ({
/** The most frames one clip or flipbook may hold; 0 is no limit. */
readonly "maxFrames"?: (number);
/** The default squash and stretch of the style's movement: og.tween.squash's impact where a call leaves it out, 0.5 the preset's own (1.25, 0.8), 1 as rubber hose (1.5, 0.6); 0 is no constraint (the preset's own). */
readonly "squash"?: (number);
/** Clips and flipbooks advance this many times a second whatever the tick, holding each pose between (12 is on twos at 24 fps, 8 on threes); 0 is smooth, no constraint. */
readonly "stepFps"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtPalette": ({
/** The colours of colors kept for signals (the player, danger, the goal, a pickup), by their index in the palette (0 its first colour): scenery and backgrounds stay off them. A look gives them on one line (`accents 3 7`); empty keeps none. */
readonly "accents"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** The palette (og.load.palette of a `.hex` or GIMP `.gpl` file; a look names one under assets/ or a built-in `@palettes/<name>`); 0 is none, any colour. */
readonly "colors"?: ({
readonly "$ogRef": (string);
});
/** How an asset mixes the two nearest colours where a tone falls between them: none in flat bands (the zero value), or an ordered or noise pattern as Stylise.dither. */
readonly "dither"?: ("none" | "bayer2" | "bayer4" | "bayer8" | "blueNoise" | "interleavedGradient") & (string);
/** The most distinct colours one asset (a sprite sheet, a texture, a model's colours) may use; 0 is no limit. */
readonly "maxColors"?: (number);
/** The most shades one hue ramps through from its shadow to its highlight (3 or 4 for 8-bit art, 4 to 6 for 16-bit); 0 is no limit. */
readonly "rampSteps"?: (number);
/** Every opaque texel is exactly a colour of colors (indexed colour, as hand-placed pixel art); false lets texels between them (shading, filtering) and holds only the hues to the palette. */
readonly "strict"?: (boolean);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtPixels": ({
/** The rows of the logical resolution the art is drawn for, as Stylise.pixelHeight counts them (180, 224, 240); 0 is no constraint. */
readonly "logicalHeight"?: (number);
/** How pixel art turns: free (no constraint), nearest, cleanEdge, or prerotated in rotationSteps. */
readonly "rotation"?: ("free" | "nearest" | "cleanEdge" | "prerotated") & (string);
/** With rotation prerotated: the rotations baked per full turn (8 is one every 45 degrees); 0 is none. */
readonly "rotationSteps"?: (number);
/** Sprites and tiles stand on whole logical pixels: their positions are multiples of 1/unit metres, so no pixel of the art lies between two of the grid. */
readonly "snap"?: (boolean);
/** The cell sizes, in logical pixels, a sprite, a flipbook frame or a tile may have: each side is one of them (`spriteSizes 16 32` takes 16 x 16, 16 x 32 and 32 x 32). A look gives them on one line; empty is any size. */
readonly "spriteSizes"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Logical pixels per metre (per world unit in 2D): one pixel of every sprite, tile and backdrop covers 1/unit metres, so no two assets show pixels of different sizes (16 with 16-pixel tiles one metre wide); 0 is no constraint. */
readonly "unit"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtShape": ({
/** Hard edges rounded by a chamfer of this radius (clay, toys, diorama pieces); 0 is none. */
readonly "bevel"?: (number);
/** The most triangles one character may have; 0 is no budget. */
readonly "characterFaces"?: (number);
/** With normals flat: neighbouring faces within this angle of their facet's normal shade as one facet, one normal and one baked colour, so a gently curved ground is a few broad planes, not a pattern of triangles. 0 is every triangle its own facet. */
readonly "facetAngle"?: (number);
/** With normals angle: the angle between two faces past which their edge is shaded hard. */
readonly "normalAngle"?: (number);
/** How models are shaded across their faces: any (no constraint), smooth, flat, or angle (split past normalAngle). */
readonly "normals"?: ("any" | "smooth" | "flat" | "angle") & (string);
/** The most triangles one prop may have (50 to 500 for faceted low poly); 0 is no budget. An effect's mesh (`conform effect` in its import settings, as og vfx import writes) has none. */
readonly "propFaces"?: (number);
/** The thinnest feature a model's silhouette keeps at the style's scale: thinner detail does not read and is left out; 0 is no constraint. */
readonly "silhouette"?: (number);
/** Models built of cubes of this edge (voxel worlds); 0 is no voxel grid. */
readonly "voxel"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::ArtTexels": ({
/** Texels per metre on the surfaces of 3D models (a 2 m character at 64 texels per metre takes a 128-texel-tall texture); 0 is no constraint. */
readonly "density"?: (number);
/** How the style's textures are sampled: any (no constraint), nearest or linear. */
readonly "filter"?: ("any" | "nearest" | "linear") & (string);
/** The longest side of any texture of the style, in texels (256 for low-resolution 3D); 0 is no limit. */
readonly "maxSize"?: (number);
/** Their mip maps: any (no constraint), full or none. */
readonly "mips"?: ("any" | "full" | "none") & (string);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::Environment": ({
/** Draw the map behind the scene, as three.js scene.background = the environment (Camera.background, og.Sky and RenderSettings.sky give way); false leaves the background as they set it. */
readonly "background"?: (boolean);
/** Blurs only the background, as three.js scene.backgroundBlurriness: 0 sharp to 1 the coarsest mip. HDR and physical skies use GGX levels; authored LDR skies use their own source-resolution box mips. The light is not changed. */
readonly "backgroundBlurriness"?: (number);
/** Multiplies the map where it is drawn behind the scene, as three.js scene.backgroundIntensity; the light is not changed. */
readonly "backgroundIntensity"?: (number);
/** A second map from og.load.environment that the light and the background blend towards (indoor to outdoor); 0 is none. Both maps turn by rotation and the background is scaled by backgroundIntensity. */
readonly "blendTo"?: ({
readonly "$ogRef": (string);
});
/** How far the blend has gone: 0 the map, 1 blendTo, in between a mix of both maps' light (every GGX level and the harmonics) and background. Drive it from the tick (a tween or a system), never from a clock, so a replay blends alike; it is drawn in 128 steps, each regenerating the light once. */
readonly "blendWeight"?: (number);
/** The ground point under where the map was photographed: the dome's centre is groundHeight above it. */
readonly "groundCenter"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** Draws the sky on a dome with a flat ground under it, as three.js GroundedSkybox: how far above the ground the map was photographed (metres). Objects stand on the map's ground, and the environment's reflections show it too. 0 (the default) draws the sky at infinity. */
readonly "groundHeight"?: (number);
/** The grounded dome's radius (metres, larger than groundHeight), as GroundedSkybox's radius: the ground looks flat out to about this distance. */
readonly "groundRadius"?: (number);
/** How dark the grounded ground (groundHeight above 0) takes the shadows of what stands on it, the sun's (a DirectionalLight with castShadow) and the map's light they hide, so props sit on it: 1 as dark as a real floor, 0 none (the ground as photographed). */
readonly "groundShadows"?: (number);
/** The hemisphere light's ground colour (linear), as three.js HemisphereLight.groundColor. */
readonly "hemiGround"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The hemisphere light's intensity, as three.js HemisphereLight.intensity; 0 (the default) is off. */
readonly "hemiIntensity"?: (number);
/** A hemisphere light's sky colour (linear), as three.js HemisphereLight.color: diffuse-only ambient light mix(hemiGround, hemiSky, 0.5 n.y + 0.5) * hemiIntensity, darkened by ambient occlusion like the map's light. Unlike light baked into the map it adds no reflection. */
readonly "hemiSky"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The strength of the map's light on every surface, as three.js scene.environmentIntensity; while a map is set it replaces RenderSettings.environmentIntensity. */
readonly "intensity"?: (number);
/** The map from og.load.environment; 0 lights by the sky. */
readonly "map"?: ({
readonly "$ogRef": (string);
});
/** Turns the map about +Y, as three.js scene.environmentRotation.y: positive turns what was at +X towards -Z. The light and the background turn together. */
readonly "rotation"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::Fog": ({
/** Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light. */
readonly "anisotropy"?: (number);
/** The colour the fog tends to (linear, before exposure). */
readonly "color"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Exponential distance fog per metre past start, with or without end's ramp (0: none). */
readonly "density"?: (number);
/** Draw the fog. */
readonly "enabled"?: (boolean);
/** The distance from the camera where fog is full: with end > start, a linear ramp from start to end applied together with density's exponential fog (0: no ramp). */
readonly "end"?: (number);
/** How fast height fog thins above height, per metre. */
readonly "falloff"?: (number);
/** The world height below which height fog is densest. */
readonly "height"?: (number);
/** Height fog at and below height (0: none). */
readonly "heightDensity"?: (number);
/** The most the global distance and height fog together hide, also with volumetric on. Does not limit independent FogVolume media. */
readonly "maxOpacity"?: (number);
/** How much of the sun's and the lamps' light the fog scatters (its albedo, tinted by color): 0 leaves the plain fog's haze without shafts or glow, 1 sends all of that light on, above 1 brightens the shafts for effect. */
readonly "scattering"?: (number);
/** How much fog covers the sky and clouds (0: clear). Below a cumulus base, or without cumulus, the lower sky stands for distant land and takes land fog. Above a cumulus base it is cloud sea and keeps this share. */
readonly "sky"?: (number);
/** The distance from the camera where distance fog starts, also with volumetric on. Height fog and FogVolume are independent of start. */
readonly "start"?: (number);
/** Plain fog's colour looking into the sun. Ignored by volumetric fog, which scatters the actual lights' colours; anisotropy controls its angular spread. */
readonly "sunColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** How tight plain fog's sun-colour glow is (higher: smaller). Ignored by volumetric fog: use anisotropy for its Henyey-Greenstein angular spread. */
readonly "sunPower"?: (number);
/** Light the fog as a medium: unlit, it tends to color as the plain fog does; over that it (density, thickened below height) scatters the sun through its cascades and the point, spot and rect lights (up to RenderCapabilities.volumetricLights a sample). Blended surfaces and sprites take the plain fog. */
readonly "volumetric"?: (boolean);
/** How far from the camera the fog shows shafts and the local lights; past it the fog goes on in its colour, with the sun's light in it unshadowed. */
readonly "volumetricDistance"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::MaterialStyle": ({
/** Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks. */
readonly "flatShading"?: (boolean);
/** The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands. */
readonly "gradientMap"?: ({
readonly "$ogRef": (string);
});
/** The outline colour for materials without their own (linear). */
readonly "outlineColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Distance in metres at which outlines have their base width. Nearer they grow up to outlineNearCap times that width (default 4); farther they thin proportionally. 0 keeps a constant width. Foliage never grows beyond its base width and fades below a pixel. */
readonly "outlineDistance"?: (number);
/** Near-camera width limit as a multiple of the base width, with outlineDistance > 0. Default 4 preserves existing growth; 1 caps near lines at base width while retaining far falloff. Also applies to material and ArtLine outlines. Foliage and orthographic cameras keep their existing rules. */
readonly "outlineNearCap"?: (number);
/** Where outlines lie against an object's silhouette: inside (the zero value, its own edge pixels), outside (round it, at any width, anti-aliased) or both. Creases between objects stay inside. */
readonly "outlinePlace"?: ("inside" | "outside" | "both") & (string);
/** The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one. A surface seen at a grazing angle is lined only where it ends. */
readonly "outlineWidth"?: (number);
/** Rim (fresnel) light for materials without their own (linear). */
readonly "rimColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The rim light's strength; 0 is none. */
readonly "rimIntensity"?: (number);
/** How tight the rim is (higher: thinner). */
readonly "rimPower"?: (number);
/** How far shadowed surfaces fall towards black: where the sun does not reach, a surface loses this share of the sun's ramp and of the ambient light (1: solid black shadows). Point, spot and rect lights and the emissive still show inside it; shadowTint's light is added after it. 0 is off. */
readonly "shadowDarkness"?: (number);
/** The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration. */
readonly "shadowTint"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** How far shadowed light moves towards shadowTint; 0 is off. */
readonly "shadowTintAmount"?: (number);
/** A made surface on every lit material (clay, paper, plastic): its normal detail, colour grain and light response, in each mesh's own space; none is no change. Unlit materials, materials with StandardMaterialDesc.styled false and those with a MeshPhysicalMaterial lobe keep their own shading. */
readonly "surface"?: ("none" | "clay" | "paper" | "plastic") & (string);
/** The size of the surface's pattern in metres of the mesh's own space (a clay lump, the paper's fibre); 0 takes the surface's own (clay 0.12, paper 0.15, plastic 0.3). */
readonly "surfaceScale"?: (number);
/** How strongly the surface shows: its bumps, grain and wrapped light; 0 is none. */
readonly "surfaceStrength"?: (number);
/** Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway. */
readonly "toon"?: (boolean);
/** How smooth a toon material's ambient light (the Environment and its hemisphere light) and reflection stay: 0 one flat colour, the light from above and below averaged (MToon's equalised ambient), so the bands sit on flat colour; 1 the smooth PBR ambient and reflections. */
readonly "toonAmbient"?: (number);
/** The toon bands' and highlight's edges, anti-aliased over this many pixels (lilToon's screen-space smoothing); 0 hard steps that only TAA softens. */
readonly "toonSoftness"?: (number);
/** Draw every lit material's feature edges over its faces (StandardMaterialDesc.wireframe's; this style's angle and colour where it sets none), in the colour the surface shows along each edge or wireframeColor; the faces hide the edges behind them. A material's own wireframe draws its edges alone. */
readonly "wireframe"?: (boolean);
/** The crease angle wireframe draws past (StandardMaterialDesc.wireframeAngle); 0 every triangle edge. */
readonly "wireframeAngle"?: (number);
/** The look's line colour for wireframe (StandardMaterialDesc.wireframeColor), linear, above 1 blooms; black (the zero value) draws each edge in the colour its surface shows there. */
readonly "wireframeColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::PostEffects": ({
/** Screen-space ambient occlusion: darkens the ambient light in creases and contact areas. Initially false in a 2D project. */
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
readonly "Reflected_og_cook::cook::source_schemas::procedural::RenderSettings": ({
/** How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. */
readonly "adaptSpeedDown"?: (number);
/** How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame, never by the wall clock; 0 is instant. Snapshots and probe sweeps carry the adapted exposure. */
readonly "adaptSpeedUp"?: (number);
/** PostEffects.ao darkens creases; false draws none. PostEffects keeps its settings either way. */
readonly "ambientOcclusion"?: (boolean);
/** The resolution ambient occlusion is searched at: postEffects (the default) draws PostEffects.aoResolution; full or half is drawn in its place, and PostEffects keeps its own for when this is postEffects again. */
readonly "aoResolution"?: ("postEffects" | "full" | "half") & (string);
/** Meter every frame to a key that follows its brightness (middle grey for a daylit view), then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the frame's own; otherwise it adapts from the exposure metered on the first frame after the first tick, so hosts draw alike. */
readonly "autoExposure"?: (boolean);
/** The most exposure auto exposure applies, in stops (EV, compensation included): a dark view is brightened no further, so a night stays as dark as its light makes it. Below autoExposureMin it is taken as autoExposureMin. */
readonly "autoExposureMax"?: (number);
/** The least exposure auto exposure applies, in stops (EV: the image times 2^EV, Camera.exposure's compensation included): a very bright view is darkened no further. */
readonly "autoExposureMin"?: (number);
/** Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map. */
readonly "bakedLighting"?: (boolean);
/** Light above the threshold bleeds into its surroundings. Initially false in a 2D project. */
readonly "bloom"?: (boolean);
/** With bloomRadius 0 or more: the high pass (and the blur) sees each channel as 0.18 x (colour / 0.18) to this power before exposure, as a contrast grade run ahead of UnrealBloomPass (the way a three.js grade pass before the bloom does) would leave it. 1, the default, leaves the colour as lit. */
readonly "bloomContrast"?: (number);
/** Allow RenderSettings.bloom to draw. Low disables the HDR bloom chain; medium and high allow it. The scene's bloom switch and settings are kept, and authored 2D glow is independent. */
readonly "bloomEnabled"?: (boolean);
/** How much of the blurred light is added back. */
readonly "bloomIntensity"?: (number);
/** Softens the threshold over threshold plus or minus knee; 0 is a hard cut. */
readonly "bloomKnee"?: (number);
/** Below 0 (the default) keeps Origin's bloom. 0 to 1 is three.js UnrealBloomPass at this radius: a half-resolution high pass from bloomThreshold to bloomThreshold + bloomKnee, five Gaussian mips mixed by mix(f, 1.2 - f, radius), added at bloomIntensity as EffectComposer's additive copy. */
readonly "bloomRadius"?: (number);
/** Brightness after exposure where bloom starts; 1 is display white. */
readonly "bloomThreshold"?: (number);
/** The strength of the image-based ambient light from the sky on surfaces; og.Environment.intensity takes its place while an environment map is set. The sky and its clouds are drawn as they are whatever it is. */
readonly "environmentIntensity"?: (number);
/** A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are. */
readonly "exposure"?: (number);
/** Draw what moves between the last two ticks: a frame between ticks blends entities, og.Camera and skinned joints from the tick before the last to the last, showing the world up to one tick late. Frames on a tick boundary, screenshots and captures draw the last tick. Presentation only. */
readonly "interpolate"?: (boolean);
/** A multiplier on every model's projected level-of-detail error (og.Model.lodBias multiplies it again): a level is drawn while its error, measured by the cook against the full mesh, covers under a pixel. 1 is the engine's choice, 2 keeps full meshes twice as far; 0 draws every model in full. */
readonly "lodBias"?: (number);
/** At most this many composed effects (og.Vfx) drawn at once: past it the oldest one-shot effects are left out, then the oldest looping ones, and the probe's dump_render counts them (vfx.dropped). 64; the low preset 32, high 128. Drawing only: their events, shakes and sounds still come. */
readonly "maxVfx"?: (number);
/** At most this many particle slots (the emitters' maxParticles, summed) among the og.Vfx drawn: past it the oldest one-shot effects are left out, then the oldest looping ones, as with maxVfx. 65536; the low preset 32768, high 131072. */
readonly "maxVfxParticles"?: (number);
/** Samples per pixel where the antialiasing multisamples (msaa4): 4, or 1, which draws FXAA instead. RenderCapabilities.maxMsaaSamples caps it: 1 where the device resolved no MSAA, which draws one sample whatever this says. Changing it remakes the multisampled targets and pipelines next frame. */
readonly "msaaSamples"?: (number);
/** Every particle system's emission times this (0 to 1); an emitter left with fewer than 8 particles alive at once keeps more, so a flash or a ring stays whole. The low preset draws 0.5. Drawing only: particle events and collisions are the full emitter's. */
readonly "particleEmission"?: (number);
/** The graphics preset og.render.applyQuality last wrote into the quality fields below (rectShadows to upscaler), or none. A record for the settings screen and the probe: the renderer reads the fields, never this, so a field the game sets after the preset is drawn as set. */
readonly "quality"?: ("none" | "low" | "medium" | "high") & (string);
/** Rect lights whose castShadow is on cast their soft shadows; false draws every rect light unshadowed, each keeping its castShadow for when this is on again. The low preset turns it off. */
readonly "rectShadows"?: (boolean);
/** PostEffects.ssr draws; false leaves smooth surfaces the reflection probes' and the environment's reflection. PostEffects keeps its settings either way. */
readonly "screenSpaceReflections"?: (boolean);
/** The directional light's shadow cascades, 1 to 4: more cover the same shadowDistance with finer shadows near the camera, at one more shadow pass and one more map each. The high preset draws 4, medium 3, low 2. Changing it makes the cascade maps again at the next frame. */
readonly "shadowCascades"?: (number);
/** The shadow maps at this fraction of their full side: the cascades (2048 texels by default) and local lights' layers (512), each within 64 texels and the device's limit (4096 for cascades). Changing it remakes the maps next frame. */
readonly "shadowMapScale"?: (number);
/** At most this many point, spot and rect lights cast shadows in a frame, the brightest at the camera kept (RenderCapabilities.maxShadowedLights caps it); 0 casts none. The high preset keeps 16, medium 8, low 2. The directional light's cascades are not counted. */
readonly "shadowedLights"?: (number);
/** Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map). Initially false in a 2D project. */
readonly "sky"?: (boolean);
/** Spread live sky capture and filtering over 13 rendered frames instead of completing each frame. Low sets true; medium, high and the default keep false so terrain ambient and reflections follow the current sky. Slicing lowers peak GPU cost but retains older lighting during fast sky changes. */
readonly "skyCaptureSliced"?: (boolean);
/** An entity or og.Camera that moved further than this between the last two ticks is drawn at the last tick, not blended: a teleport or a respawn this far needs no og.render.snap. 0 never snaps by distance. */
readonly "snapDistance"?: (number);
/** Particles with a softDistance fade against the scene behind them; false draws them hard-edged and asks no depth prepass for them. */
readonly "softParticles"?: (boolean);
/** PostEffects.ssrSteps times this (within the 4 to 128 steps the pass takes): 2 is 128 steps for the default 64, finer contact with thin and distant reflected surfaces at a higher GPU time. PostEffects keeps its steps. */
readonly "ssrStepScale"?: (number);
/** How far from itself a stationary or static light (LightMobility) has its diffuse light shaded per pixel; past it, that light comes from a grid of probes round the camera and such a rect light skips rough surfaces. Glossy surfaces keep the highlights. 0 shades every light whole. */
readonly "staticLightReach"?: (number);
/** Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1. */
readonly "toneMapped"?: (boolean);
/** Which curve tone maps the frame while toneMapped is true: agx (the default), aces, neutral or none. A 2D project initially uses none; an orthographic camera in a 3D project still uses agx. */
readonly "toneMapping"?: ("aces" | "agx" | "none" | "neutral") & (string);
/** Under TAA, the temporal resolve from the render scale (DisplaySettings.renderScale) to the output: auto (the default and medium: fsr3 below scale 1, taau at it), taau (low) or fsr3 (high). A device lacking fsr3 (RenderCapabilities.upscalers) draws taau. Switching restarts the history. */
readonly "upscaler"?: ("taau" | "fsr3" | "auto") & (string);
/** A volumetric og.Fog (Fog.volumetric) is drawn lit, with its light shafts, and so is the sun's light under an og.Water (its shafts, where it has caustics); false draws the fog as plain distance and height fog and the water's light even. */
readonly "volumetricFog"?: (boolean);
/** Fog.volumetric's samples along each view ray times this: the froxel grid's 64 slices deep (8 to 256), or the raymarch's 24 steps per pixel (4 to 96) where the renderer draws one (RenderCapabilities.volumetricFog). Changing it makes the grid again at the next frame. */
readonly "volumetricStepScale"?: (number);
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::Sky": ({
/** A multiplier on the whole sky. */
readonly "brightness"?: (number);
/** Draw and light by this sky. */
readonly "enabled"?: (boolean);
/** The glow around the sun. */
readonly "glow"?: (number);
/** How tight the glow is. */
readonly "glowPower"?: (number);
/** Below the horizon. */
readonly "ground"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** At the horizon. */
readonly "horizon"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The moon's colour. */
readonly "moonColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Towards the moon; zero puts it opposite sunDirection. */
readonly "moonDirection"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The lit moon's brightness. */
readonly "moonIntensity"?: (number);
/** How much of the moon is lit: 0 new, 0.25 the first quarter (its right half), 0.5 full, 0.75 the last quarter (its left half). */
readonly "moonPhase"?: (number);
/** The moon disc's angular radius (0: no moon). It sets below the horizon and hides the stars behind it. */
readonly "moonSize"?: (number);
/** Stars where the sky is dark: 0 none, 1 a full night sky. */
readonly "stars"?: (number);
/** The sun's colour. */
readonly "sunColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Towards the sun: the opposite of the way its light shines. Zero (the default) follows the frame's light: towards the DirectionalLight drawn (its entity's +Z; it shines down -Z; the brightest of several), or the built-in sun's (0.45, 0.8, 0.35) with none. A non-zero direction is used as given. */
readonly "sunDirection"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The disc's brightness. */
readonly "sunIntensity"?: (number);
/** The sun disc's angular radius (0: no disc). */
readonly "sunSize"?: (number);
/** Overhead. */
readonly "zenith"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
});
readonly "Reflected_og_cook::cook::source_schemas::procedural::Stylise": ({
/** How far texture coordinates are interpolated without perspective correction: 1 fully affine, so textures warp and swim across large polygons as on early consoles; 0 is off. */
readonly "affineTexturing"?: (number);
/** Colour bands by depth (2 to 8), interpolating depthBandsNearColor to depthBandsFarColor in lightness, chroma and hue. By default the layers divide depthBandsNear..depthBandsFar evenly; depthBandsFit instead fits logarithmic layers to visible geometry. 0 is off. */
readonly "depthBands"?: (number);
/** Where the last band ends; the far colour starts one band's depth before it and continues beyond it. Ignored when depthBandsFit is true. */
readonly "depthBandsFar"?: (number);
/** The farthest band's colour (linear, before exposure). */
readonly "depthBandsFarColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Fit logarithmic bands to the full visible non-sky depth range, measured each drawn frame. Nearer layers span less distance than farther ones. Ignores depthBandsNear and depthBandsFar; false keeps their absolute linear layers. The farthest layer still merges geometry within it. */
readonly "depthBandsFit"?: (boolean);
/** How far a band replaces the surfaces' own shading: 1 flat silhouettes in the band's colour; 0 a tint over the lit scene, its colours times the band's at unit luminance (a coloured filter: the shading, the value and the surfaces' own hues show through); between, a mix of the two. */
readonly "depthBandsFlatten"?: (number);
/** Where the first band starts, unless depthBandsFit is true. */
readonly "depthBandsNear"?: (number);
/** The nearest band's colour (linear, before exposure). */
readonly "depthBandsNearColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Quantise depth to this many bits, for the coarse depth buffers of early 3D (12 to 16 are period-like: distant surfaces fight); 0 keeps full precision. */
readonly "depthBits"?: (number);
/** How the steps of posterize and palette are mixed: none rounds every pixel, an ordered or blue-noise pattern alternates the two nearest steps in proportion. */
readonly "dither"?: ("none" | "bayer2" | "bayer4" | "bayer8" | "blueNoise" | "interleavedGradient") & (string);
/** The size of one dither cell in logical pixels. */
readonly "ditherScale"?: (number);
/** Where the dither's cells lie: screen, a grid over the frame (it crawls as the camera moves); surface, cells on the surfaces at least ditherScale pixels on the screen each way, the sky's on a cube round the camera; object, in each mesh instance's own space. Off screen the frame runs the prepass. */
readonly "ditherSpace"?: ("screen" | "surface" | "object") & (string);
/** The dither's amplitude in steps: 1 spreads exactly one step, less keeps flatter bands. */
readonly "ditherStrength"?: (number);
/** Halftone print: tones become dots on a rotated grid, larger where the image is darker; 0 is off, 1 dots only. */
readonly "halftone"?: (number);
/** The dot grid's angle. */
readonly "halftoneAngle"?: (number);
/** The ink of the dots (linear). */
readonly "halftoneColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** With halftoneMode cmyk: how far the cyan, magenta and yellow plates sit off the black one, in logical pixels, as a cheap print's colour slips past its line art; 0 is in register. */
readonly "halftoneMisregistration"?: (number);
/** How the halftone inks: one ink (halftoneColor) or four-colour process print (cmyk). */
readonly "halftoneMode"?: ("ink" | "cmyk") & (string);
/** Which tones the halftone prints on: all, the shade only or the highlights only (the darkest or lightest third of the tones). Under a region the dots are Ben-Day dots, a tint over the region's own colour at most 0.4 of a cell wide. They fade to flat ink on surfaces turned away from the camera. */
readonly "halftoneRegion"?: ("all" | "shade" | "highlight") & (string);
/** The dot grid's pitch in logical pixels. */
readonly "halftoneSize"?: (number);
/** Where the dots lie: screen, a grid over the frame; surface, grids on the surfaces that stay on them as the camera moves and wrap round forms (halftoneSize is then about the pitch in pixels where they lie); object, the same grids in each instance's own space, so a moving object carries its dots. */
readonly "halftoneSpace"?: ("screen" | "surface" | "object") & (string);
/** Pencil or ink cross-hatching from a tonal art map: darker tones add strokes in more directions over a paper-light ground, each set thinning and fading in from clean paper at its tone; 0 is off, 1 strokes only. */
readonly "hatching"?: (number);
/** The direction of the first stroke set. */
readonly "hatchingAngle"?: (number);
/** The stroke colour (linear): graphite, sepia or ink. */
readonly "hatchingColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Hatching from depth as well as luminance: the depth-normal prepass's silhouettes and creases (inkDepthThreshold's and inkNormalThreshold's edges) darken the tone the strokes follow at this strength, so strokes gather along contours and folds. 0 is the tone alone; above 0 the frame runs the prepass. */
readonly "hatchingDepth"?: (number);
/** Aerial perspective: view depth in metres at which stroke width and ink strength halve, thinning exponentially into bare paper. 0 is off. Above 0 runs the prepass in every pattern space and leaves the sky bare. The wash and contour weighting stay unchanged. */
readonly "hatchingDistance"?: (number);
/** Colour hatching: the ground is a light wash of its own colour and each stroke is that colour darkened towards hatchingColor, so hue and chroma stay under the strokes; false lays strokes of hatchingColor over a paper-light ground. */
readonly "hatchingKeepColor"?: (boolean);
/** How many tones the hatching distinguishes, each adding one stroke direction or density (1 to 6). */
readonly "hatchingLevels"?: (number);
/** The paper the strokes lie on (linear): one-ink hatching lifts the ground's light tones towards it, and under hatchingKeepColor the wash lies on the paper's colour at the wash's own lightness (a cream sketchbook, a grey card, a blue print); white keeps the frame's own. */
readonly "hatchingPaper"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** The spacing of hatch lines in logical pixels. */
readonly "hatchingScale"?: (number);
/** Where the strokes lie: screen, stripes over the frame that follow the tone alone; surface, strokes on the surfaces that follow the tone and the form and stay on them as the camera moves (hatchingScale is then about the spacing in pixels where they lie); object, the same in each instance's own space. */
readonly "hatchingSpace"?: ("screen" | "surface" | "object") & (string);
/** Ink lines where the tone changes sharply (luminance edges), over every surface, sprite and texture alike: comic and storybook contours without PostEffects.outline's depth prepass. inkDepth adds the prepass's silhouettes and creases. */
readonly "ink"?: (boolean);
/** The line colour (linear). */
readonly "inkColor"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** With ink: lines also where the depth-normal prepass breaks (silhouettes against what is behind, creases between faces), at this strength: comic contours round flat colours that luminance alone misses. 0 is luminance only; above 0 the frame runs the prepass PostEffects.outline uses. */
readonly "inkDepth"?: (number);
/** With inkDepth: the relative depth step to a farther neighbour that counts as a silhouette (0.04 is 4 %), as PostEffects.outlineDepthThreshold: lower inks smaller steps. */
readonly "inkDepthThreshold"?: (number);
/** With inkDepth: the normal change between neighbours of about the same depth that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees), as PostEffects.outlineNormalThreshold: lower inks gentler folds. */
readonly "inkNormalThreshold"?: (number);
/** The luminance step that counts as an edge (0 to 1 in display units): lower draws more lines. */
readonly "inkThreshold"?: (number);
/** The line width in logical pixels. */
readonly "inkWidth"?: (number);
/** How far the strokes follow the image's structure: 0 round brush cells, 0.5 the filter's classic ellipses, 1 thin strokes along every coherent contour, which keep a thin line's shape. */
readonly "paintAnisotropy"?: (number);
/** Colour bleeding across edges into its neighbours, wet in wet; 0 is none. */
readonly "paintBleed"?: (number);
/** A tileable paper or canvas texture (og.load.texture) laid over the frame by paintCanvasStrength; 0 is none. */
readonly "paintCanvas"?: ({
readonly "$ogRef": (string);
});
/** Canvas texels per logical pixel, with paintCanvasSpace screen (and on the sky). */
readonly "paintCanvasScale"?: (number);
/** Where the canvas lies: screen, over the frame (the scene slides under the paper as the camera moves); surface, on the surfaces, so the grain sticks to the scene, foreshortened with the floor and walls; object, in each instance's own space, so a moving object carries its grain. */
readonly "paintCanvasSpace"?: ("screen" | "surface" | "object") & (string);
/** How much the canvas texture modulates the frame: 0 none, 1 its full grain. */
readonly "paintCanvasStrength"?: (number);
/** Canvas texels per metre with paintCanvasSpace surface: 128 repeats a 256-texel canvas every two metres. */
readonly "paintCanvasWorldScale"?: (number);
/** The detail of a skinned mesh's pixels (a character's face and hands) where its material sets less: 0 painted as the rest, 1 kept as drawn. */
readonly "paintCharacterDetail"?: (number);
/** How far detail keeps its shape under paintRadius: a pixel's radius shrinks by its detail (its material's StandardMaterialDesc.paintDetail, or paintCharacterDetail on a skinned mesh) times this, and no stroke crosses an object's edge. Above 0 the frame runs the prepass; 0 paints every pixel alike. */
readonly "paintDetail"?: (number);
/** Pigment pooling at the edges of colour regions, as wet watercolour dries darker at its rim; 0 is none. */
readonly "paintEdgeDarkening"?: (number);
/** Painterly smoothing (an anisotropic Kuwahara filter): flat regions and incoherent texture become strokes of one colour this many logical pixels across; edges and thin lines keep their shape; 0 is off. Under a 2D project's art contract the cook paints each texture and the frame draws neither pass. */
readonly "paintRadius"?: (number);
/** Brush strokes in the paint, sized by paintRadius: marks along each stroke (the image's structure, or a slowly turning lay over flat regions), their own light and dark, and ragged edges where strokes meet. 0 is none; 1 bold, visible at a glance. */
readonly "paintStrokes"?: (number);
/** Quantise every pixel to the nearest colour of this palette (og.load.palette), measured in a perceptual space after posterize; 0 is none. */
readonly "palette"?: ({
readonly "$ogRef": (string);
});
/** The logical resolution: the scene is rendered this many pixels tall and scaled up to the output by a whole number (pixelUpscale) with nearest filtering; 0 renders at the output's resolution. The width follows the output's aspect, at most the output's width over the scale. */
readonly "pixelHeight"?: (number);
/** Snap the camera and world sprites to the logical pixel grid, so a moving orthographic camera scrolls by whole pixels and sprites in a 3D world keep their edges (pixel sprites in 3D sets). */
readonly "pixelSnap"?: (boolean);
/** The whole-number scale from the logical to the output resolution, a hard request: honoured whenever pixelHeight times it fits the output's height, else the largest that fits; 0 takes the largest that fits. A frame that does not fill the output is centred with Camera.background round it. */
readonly "pixelUpscale"?: (number);
/** Tones per colour channel after tone mapping (2 to 64): smooth gradients become flat steps; 0 is off. */
readonly "posterize"?: (number);
/** How posterize steps a colour: each channel apart (channel), or its value and its tint apart so light never changes its hue (tone). */
readonly "posterizeMode"?: ("channel" | "tone") & (string);
/** Snap every vertex of the 3D scene to a screen grid this many logical pixels wide, as fixed-point hardware did: geometry jitters by whole steps as it moves; 0 is off. */
readonly "vertexSnap"?: (number);
/** The wash thinned with water: the pigment over watercolorPaper is this much less dense, so darks lift towards the paper, hues keep and the paper stays the paper. About 0.3 to 0.45 turns a lit scene's heavy darks into pale washes; 0 leaves the frame's tones as lit. */
readonly "watercolorDilution"?: (number);
/** A dry brush skipping over the paper's tooth: its peaks (the relief past half its spread, in the paper's own clusters) left bare as the paper (watercolorPaper) in the rim at a wash's outer boundary (watercolorEdgeDarkening's) and on the light-catching tones, the brightest washes first; 0 is none. */
readonly "watercolorDryBrush"?: (number);
/** The wash's dark rim, drawn after paintRadius's strokes: pigment pooled two to three pixels wide at a wash's outer boundary (a colour step wider than a line, or a silhouette against something farther), the colour there raised to one plus the strength. Seams and creases get none; 0 is none. */
readonly "watercolorEdgeDarkening"?: (number);
/** Pigment settling into the paper's grain: a density from the paper's relief (paintCanvas, else a noise) thins each colour towards watercolorPaper on the peaks and thickens it in the pits, weighed by the pigment load so the paper and pale washes stay clean; the wash keeps its mean value. 0 is none. */
readonly "watercolorGranulation"?: (number);
/** The paper's colour (linear): what the wash thins towards and the dry brush leaves bare. */
readonly "watercolorPaper"?: (((ReadonlyArray<(number)> & { readonly length: 3 })) | ((string)));
/** Pigment drifting as the wash dries: the same density from a slow noise about a hundred canvas texels across, laid where the canvas lies (paintCanvasSpace) and seeded by the object under the pixel so no blotch spans two washes, weighed by the pigment load; 0 is none. */
readonly "watercolorTurbulence"?: (number);
/** The painted frame displaced by the paper's relief, up to this many logical pixels, so every edge wavers with the tooth as a wash's does (MNPR's substrate distortion); 0 is none. */
readonly "watercolorWobble"?: (number);
});
readonly "UiSeeds": ({
readonly "accent"?: (((InputDefinitions["HexColor"])) | ((null)));
readonly "danger"?: (((InputDefinitions["HexColor"])) | ((null)));
readonly "info"?: (((InputDefinitions["HexColor"])) | ((null)));
readonly "neutral"?: (((InputDefinitions["HexColor"])) | ((null)));
readonly "primary"?: (((InputDefinitions["HexColor"])) | ((null)));
readonly "success"?: (((InputDefinitions["HexColor"])) | ((null)));
readonly "team"?: (((InputDefinitions["HexColor"])) | ((null)));
readonly "warn"?: (((InputDefinitions["HexColor"])) | ((null)));
});
readonly "UiStyle": (((string)) | ((InputDefinitions["ContentRef"])));
}
```
