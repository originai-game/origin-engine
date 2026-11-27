<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-import"></a>
### `file:import`

Checked `file["import"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = (((({
readonly "animation"?: (((InputDefinitions["Animation"])) | ((null)));
readonly "collider"?: (((InputDefinitions["Collider"])) | ((null)));
readonly "conform"?: (((InputDefinitions["Conform"])) | ((null)));
readonly "fracture"?: (((InputDefinitions["Fracture"])) | ((null)));
/** Source forward axis rotated to +Z; must be perpendicular to up. */
readonly "front"?: (((InputDefinitions["Axis2"])) | ((null)));
readonly "impostor"?: ((boolean) | (null));
readonly "impostorBelow"?: ((boolean) | (null));
readonly "impostorGrid"?: ((number) | (null));
/** Atlas width (grid × frame size) must not exceed 4096 pixels. */
readonly "impostorSize"?: ((number) | (null));
readonly "kind": ("model") & (string);
readonly "lodError"?: ((number) | (null));
/** At most eight strictly decreasing fractions beginning at one. */
readonly "lods"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "process"?: (((InputDefinitions["Process"])) | ((null)));
readonly "raycast"?: ((boolean) | (null));
/** Opt-in probe-region skin relaxation (12 passes); absent means off. */
readonly "relaxSkin"?: ((boolean) | (null));
/** GLB/glTF/OBJ/FBX/VOX input. External buffers and images are discovered by
the importer and must enter the closed input/dependency graph as well. */
readonly "source": (string);
readonly "streamTextures"?: ((boolean) | (null));
readonly "up"?: (((InputDefinitions["Axis2"])) | ((null)));
readonly "voxClips"?: (ReadonlyArray<(InputDefinitions["VoxClip"])> | (number extends ((InputDefinitions["VoxClip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "voxFps"?: ((number) | (null));
readonly "voxOrigin"?: (((InputDefinitions["VoxOrigin"])) | ((null)));
readonly "voxScale"?: ((number) | (null));
})) & { readonly "artboards"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "fonts"?: never; readonly "level"?: never; readonly "normal"?: never; readonly "pixel"?: never; readonly "settings"?: never; readonly "solid"?: never; readonly "stream"?: never; readonly "sway"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "usage"?: never; }) | ((({
readonly "conform"?: (((InputDefinitions["TextureConform"])) | ((null)));
readonly "kind": ("texture") & (string);
readonly "normal"?: (((InputDefinitions["Normal"])) | ((null)));
readonly "source": (string);
/** Pixel textures have no mip chain and cannot stream (domain validation). */
readonly "stream"?: ((boolean) | (null));
readonly "sway"?: (((InputDefinitions["Sway"])) | ((null)));
/** Distinct from the root import kind discriminant. */
readonly "usage"?: (((InputDefinitions["TextureUsage"])) | ((null)));
})) & { readonly "animation"?: never; readonly "artboards"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "collider"?: never; readonly "fonts"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "level"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "pixel"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "settings"?: never; readonly "solid"?: never; readonly "streamTextures"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }) | ((({
/** Rounded to whole kb/s; absent selects the mono/stereo codec default. */
readonly "bitrate"?: ((number) | (null));
readonly "chunk"?: ((number) | (null));
readonly "codec"?: (((InputDefinitions["Codec"])) | ((null)));
readonly "kind": ("sound") & (string);
readonly "source": (string);
/** QOA only. Absent selects automatic streaming above 1 MiB; Opus streams
its own codec blocks rather than QOA chunks. */
readonly "stream"?: ((boolean) | (null));
})) & { readonly "animation"?: never; readonly "artboards"?: never; readonly "collider"?: never; readonly "conform"?: never; readonly "fonts"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "level"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "normal"?: never; readonly "pixel"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "settings"?: never; readonly "solid"?: never; readonly "streamTextures"?: never; readonly "sway"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "usage"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }) | ((({
readonly "kind": ("font") & (string);
/** Omitted: outline font. Pixel grid changes only this font's raster mode. */
readonly "pixel"?: ((number) | (null));
readonly "source": (string);
})) & { readonly "animation"?: never; readonly "artboards"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "collider"?: never; readonly "conform"?: never; readonly "fonts"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "level"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "normal"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "settings"?: never; readonly "solid"?: never; readonly "stream"?: never; readonly "streamTextures"?: never; readonly "sway"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "usage"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }) | ((({
readonly "kind": ("tilemap") & (string);
readonly "level"?: ((string) | (null));
readonly "solid"?: (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Explicit Tiled/LDtk exchange text; never an engine author DSL. */
readonly "source": (string);
/** Embedded Tiled/LDtk image paths as written in the map to texture refs. */
readonly "textures"?: (Readonly<Record<string, (InputDefinitions["ContentRef"])>>);
readonly "tileset"?: (((InputDefinitions["ContentRef"])) | ((null)));
/** External TSX/TSJ or LDtk image paths as written in the map to tileset refs. */
readonly "tilesets"?: (Readonly<Record<string, (InputDefinitions["ContentRef"])>>);
/** World units per tile width. */
readonly "unit"?: ((number) | (null));
})) & { readonly "animation"?: never; readonly "artboards"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "collider"?: never; readonly "conform"?: never; readonly "fonts"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "normal"?: never; readonly "pixel"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "settings"?: never; readonly "stream"?: never; readonly "streamTextures"?: never; readonly "sway"?: never; readonly "up"?: never; readonly "usage"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }) | ((({
/** Omitted: all artboards. Explicit selection retains authored order. */
readonly "artboards"?: ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "fonts"?: (Readonly<Record<string, (InputDefinitions["FontSource"])>>);
readonly "kind": ("rive") & (string);
readonly "settings"?: (Readonly<Record<string, (InputDefinitions["Artboard"])>>);
readonly "source": (string);
})) & { readonly "animation"?: never; readonly "bitrate"?: never; readonly "chunk"?: never; readonly "codec"?: never; readonly "collider"?: never; readonly "conform"?: never; readonly "fracture"?: never; readonly "front"?: never; readonly "impostor"?: never; readonly "impostorBelow"?: never; readonly "impostorGrid"?: never; readonly "impostorSize"?: never; readonly "level"?: never; readonly "lodError"?: never; readonly "lods"?: never; readonly "normal"?: never; readonly "pixel"?: never; readonly "process"?: never; readonly "raycast"?: never; readonly "relaxSkin"?: never; readonly "solid"?: never; readonly "stream"?: never; readonly "streamTextures"?: never; readonly "sway"?: never; readonly "textures"?: never; readonly "tileset"?: never; readonly "tilesets"?: never; readonly "unit"?: never; readonly "up"?: never; readonly "usage"?: never; readonly "voxClips"?: never; readonly "voxFps"?: never; readonly "voxOrigin"?: never; readonly "voxScale"?: never; }));
export interface InputDefinitions {
readonly "Align": ("center" | "top" | "bottom" | "left" | "right" | "top-left" | "top-right" | "bottom-left" | "bottom-right") & (string);
readonly "Anchor": (((({
readonly "kind": ("none") & (string);
})) & { readonly "bottom"?: never; readonly "distance"?: never; readonly "ends"?: never; readonly "normal"?: never; }) | ((({
readonly "bottom": (boolean);
readonly "ends": (boolean);
readonly "kind": ("supports") & (string);
})) & { readonly "distance"?: never; readonly "normal"?: never; }) | ((({
readonly "distance": (number);
readonly "kind": ("plane") & (string);
readonly "normal": (ReadonlyArray<(number)> & { readonly length: 3 });
})) & { readonly "bottom"?: never; readonly "ends"?: never; }));
readonly "Animation": ({
readonly "clips"?: (ReadonlyArray<(InputDefinitions["Clip"])> | (number extends ((InputDefinitions["Clip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "compress"?: ((boolean) | (null));
readonly "events"?: (ReadonlyArray<(InputDefinitions["Event"])> | (number extends ((InputDefinitions["Event"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "footIk"?: ((boolean) | (null));
readonly "libraries"?: (ReadonlyArray<(InputDefinitions["ContentRef"])> | (number extends ((InputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "mirrors"?: (ReadonlyArray<(InputDefinitions["Mirror"])> | (number extends ((InputDefinitions["Mirror"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "rate"?: ((number) | (null));
readonly "rig"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "tolerance"?: ((number) | (null));
});
readonly "Artboard": ({
readonly "align"?: (((InputDefinitions["Align"])) | ((null)));
readonly "fit"?: (((InputDefinitions["Fit"])) | ((null)));
readonly "name"?: ((string) | (null));
readonly "role"?: (((InputDefinitions["Role"])) | ((null)));
readonly "stateMachine"?: ((string) | (null));
});
readonly "Axis": ("x" | "y" | "z") & (string);
readonly "Axis2": ("+x" | "-x" | "+y" | "-y" | "+z" | "-z") & (string);
readonly "Bake": ({
/** Zero fills all uncovered texels. */
readonly "dilate"?: ((number) | (null));
readonly "distance"?: ((number) | (null));
readonly "normals"?: ((boolean) | (null));
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
});
readonly "Channel": ({
readonly "interpolation"?: (((InputDefinitions["Interpolation"])) | ((null)));
readonly "keys": (ReadonlyArray<(InputDefinitions["Key"])> | (number extends ((InputDefinitions["Key"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "node": (string);
readonly "path": (InputDefinitions["ChannelPath"]);
});
readonly "ChannelPath": ("translation" | "rotation" | "scale" | "weights" | "visibility") & (string);
readonly "Clip": ({
readonly "channels": (ReadonlyArray<(InputDefinitions["Channel"])> | (number extends ((InputDefinitions["Channel"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
});
readonly "Codec": ("qoa" | "opus") & (string);
readonly "Collider": (((boolean)) | ((InputDefinitions["ColliderMethod"])));
readonly "ColliderMethod": ("convex" | "vhacd") & (string);
readonly "Conform": ("off" | "character" | "effect") & (string);
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Decimate": ({
readonly "crossColorBorders"?: ((boolean) | (null));
readonly "lockBorder"?: ((boolean) | (null));
readonly "maxError"?: ((number) | (null));
readonly "mergeCharts"?: ((number) | (null));
readonly "ratio"?: ((number) | (null));
/** Zero leaves the triangle-count budget unset. */
readonly "target"?: ((number) | (null));
});
readonly "Event": ({
readonly "clip": (string);
readonly "marker": (string);
readonly "time": (number);
});
readonly "Fit": ("contain" | "cover" | "fill" | "none" | "scale-down" | "fit-width" | "fit-height") & (string);
readonly "FontSource": ({
/** Omitted: use the embedded font with this source name. */
readonly "source"?: ((string) | (null));
});
readonly "Fracture": ({
readonly "anchor"?: (((InputDefinitions["Anchor"])) | ((null)));
readonly "brick"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "children"?: ((number) | (null));
readonly "chunks"?: ((number) | (null));
readonly "grain"?: (((InputDefinitions["Axis"])) | ((null)));
readonly "impact"?: (((InputDefinitions["Impact"])) | ((null)));
readonly "interior"?: (((InputDefinitions["Material"])) | ((null)));
readonly "interiorScale"?: ((number) | (null));
readonly "mortar"?: (((InputDefinitions["Material"])) | ((null)));
readonly "pattern"?: (((InputDefinitions["Pattern"])) | ((null)));
readonly "rings"?: ((number) | (null));
readonly "seed"?: ((number) | (null));
/** Explicit centres replace sampling; count 2..128, distinct, volume
Voronoi only, exclusive with chunks/impact (domain validation). */
readonly "sites"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "style"?: (((InputDefinitions["Style"])) | ((null)));
});
readonly "Impact": ({
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "radius": (number);
});
readonly "Interpolation": ("step" | "linear" | "cubicSpline") & (string);
readonly "Joint": ({
readonly "name": (string);
/** Empty string denotes root; all other parents must precede the child. */
readonly "parent": (string);
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "Key": ({
readonly "time": (number);
/** Metres, xyzw unit quaternion, scale factors, morph weights or 0/1.
Cubic spline keys contain in-tangent, value, out-tangent (per second). */
readonly "values": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
/** Omitted fields remain absent until the material domain normalizes them. */
readonly "Material": ({
/** Alpha-mask cutoff takes precedence over opacity blending. */
readonly "alphaTest"?: ((number) | (null));
/** Ambient occlusion in red. */
readonly "aoMap"?: ((string) | (null));
readonly "autumnColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "autumnColor2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "clearcoat"?: ((number) | (null));
readonly "clearcoatRoughness"?: ((number) | (null));
/** Linear base RGB (glTF factor); not a runtime colour handle. */
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "domeNormals"?: ((number) | (null));
readonly "doubleSided"?: ((boolean) | (null));
/** Linear emitted RGB, multiplied by emissiveIntensity. */
readonly "emissive"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "emissiveIntensity"?: ((number) | (null));
readonly "grime"?: ((number) | (null));
readonly "grimeColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "grimeHeight"?: ((number) | (null));
readonly "ior"?: ((number) | (null));
/** Source image inputs; not cooked texture references. Resolution and path
containment are checked at the input boundary. */
readonly "map"?: ((string) | (null));
readonly "metalness"?: ((number) | (null));
/** Roughness in green, metalness in blue. */
readonly "metalnessMap"?: ((string) | (null));
readonly "normalMap"?: ((string) | (null));
readonly "normalScale"?: ((number) | (null));
readonly "opacity"?: ((number) | (null));
readonly "roughness"?: ((number) | (null));
/** Zero evergreen; one deciduous. */
readonly "season"?: ((number) | (null));
readonly "seasonShift"?: ((number) | (null));
readonly "stains"?: ((number) | (null));
readonly "stochastic"?: ((number) | (null));
readonly "thickness"?: ((number) | (null));
/** Diffuse transmission through thin leaves; enables both faces. */
readonly "translucency"?: ((number) | (null));
readonly "transmission"?: ((number) | (null));
});
readonly "Mirror": ({
readonly "clip": (string);
readonly "name"?: ((string) | (null));
});
readonly "Normal": (((({
readonly "kind": ("auto") & (string);
})) & { readonly "source"?: never; }) | (({
readonly "kind": ("image") & (string);
readonly "source": (string);
})));
readonly "Pattern": ("auto" | "volume" | "slab") & (string);
readonly "Process": ({
readonly "bake"?: (((InputDefinitions["Bake"])) | ((null)));
readonly "decimate"?: (((InputDefinitions["Decimate"])) | ((null)));
readonly "project"?: (((InputDefinitions["Project"])) | ((null)));
readonly "repair"?: (((InputDefinitions["Repair"])) | ((null)));
readonly "rig"?: (((InputDefinitions["Rig"])) | ((null)));
readonly "transfer"?: (((InputDefinitions["Transfer"])) | ((null)));
readonly "unwrap"?: (((InputDefinitions["Unwrap"])) | ((null)));
});
readonly "Project": ({
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
readonly "views": (ReadonlyArray<(InputDefinitions["View"])> | (number extends ((InputDefinitions["View"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Projection": (((({
readonly "fov": (number);
readonly "kind": ("perspective") & (string);
})) & { readonly "ortho"?: never; }) | ((({
readonly "kind": ("orthographic") & (string);
readonly "ortho": (number);
})) & { readonly "fov"?: never; }));
readonly "Repair": ({
readonly "floaterFraction"?: ((number) | (null));
readonly "floaters"?: ((number) | (null));
readonly "fuse"?: ((boolean) | (null));
readonly "holeEdges"?: ((number) | (null));
readonly "holesThrough"?: ((boolean) | (null));
readonly "minEdge"?: ((number) | (null));
readonly "nonManifold"?: ((boolean) | (null));
readonly "thicken"?: ((number) | (null));
readonly "weld"?: ((number) | (null));
readonly "weldAll"?: ((boolean) | (null));
});
readonly "Rig": (((({
readonly "kind": ("auto") & (string);
})) & { readonly "joints"?: never; }) | (({
readonly "joints": (ReadonlyArray<(InputDefinitions["Joint"])> | (number extends ((InputDefinitions["Joint"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("joints") & (string);
})));
readonly "Role": ("button" | "link" | "text" | "heading" | "image" | "checkbox" | "radio" | "switch" | "slider" | "spinbutton" | "textbox" | "combobox" | "listbox" | "option" | "tab" | "tablist" | "tabpanel" | "radiogroup" | "progressbar" | "meter" | "timer" | "status" | "alert" | "log" | "dialog" | "alertdialog" | "tooltip" | "separator" | "group" | "region" | "navigation" | "toolbar" | "menu" | "menubar" | "menuitem" | "menuitemcheckbox" | "menuitemradio" | "list" | "listitem" | "tree" | "treeitem" | "table" | "grid" | "row" | "rowheader" | "columnheader" | "cell" | "gridcell" | "figure" | "scrollbar") & (string);
readonly "Style": ("voronoi" | "concrete" | "brick" | "wood" | "glass") & (string);
readonly "Sway": ({
readonly "kind": (InputDefinitions["SwayKind"]);
readonly "parts"?: ((string) | (null));
});
readonly "SwayKind": ("clumps" | "tiers") & (string);
readonly "TextureConform": (((({
readonly "kind": ("off") & (string);
})) & { readonly "size"?: never; }) | (({
readonly "kind": ("sprite") & (string);
readonly "size"?: ((number) | (null));
})) | (({
readonly "kind": ("ui") & (string);
readonly "size"?: ((number) | (null));
})) | (({
readonly "kind": ("auto") & (string);
readonly "size"?: ((number) | (null));
})));
readonly "TextureUsage": ("color" | "normal" | "linear" | "single" | "pixel") & (string);
readonly "Transfer": ({
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
readonly "source": (InputDefinitions["ContentRef"]);
});
readonly "Unwrap": ({
readonly "angle"?: ((number) | (null));
readonly "conformal"?: ((boolean) | (null));
readonly "padding"?: ((number) | (null));
readonly "resolution"?: ((number) | (null));
readonly "seams"?: ((boolean) | (null));
readonly "shared"?: ((boolean) | (null));
});
readonly "View": ({
/** Square RGBA source; actual dimensions/decoding are input-domain checks. */
readonly "image": (string);
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "projection": (InputDefinitions["Projection"]);
readonly "target": (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "VoxClip": ({
/** Inclusive frame indices; first <= last and source bounds are domain checks. */
readonly "first": (number);
readonly "last": (number);
readonly "name": (string);
});
readonly "VoxOrigin": ("center" | "bottom") & (string);
}
```
