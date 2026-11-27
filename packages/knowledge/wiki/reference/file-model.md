<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-model"></a>
### `file:model`

Checked `file["model"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ((({
readonly "desc": (InputDefinitions["GeometryModel"]);
readonly "kind": ("geometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["ProgramModel"]);
readonly "kind": ("program") & (string);
})));
export interface InputDefinitions {
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
readonly "mirrors"?: (ReadonlyArray<(InputDefinitions["Mirror2"])> | (number extends ((InputDefinitions["Mirror2"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "rate"?: ((number) | (null));
readonly "rig"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "tolerance"?: ((number) | (null));
});
readonly "Arc": ({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "clockwise"?: (boolean);
readonly "end": (number);
readonly "radius": (number);
readonly "start": (number);
});
readonly "Axis": ("x" | "y" | "z") & (string);
readonly "Bake": ({
/** Zero fills all uncovered texels. */
readonly "dilate"?: ((number) | (null));
readonly "distance"?: ((number) | (null));
readonly "normals"?: ((boolean) | (null));
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
});
readonly "BoxGeometry": ({
/** Z extent, metres; unset 1. */
readonly "depth"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Y extent, metres; unset 1. */
readonly "height"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** X extent, metres; unset 1. */
readonly "width"?: ((number) | (null));
});
readonly "Branch": ({
/** -1 is the trunk; otherwise an earlier branch index. */
readonly "parent"?: ((number) | (null));
readonly "path": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "radii": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "segments"?: ((number) | (null));
});
readonly "BufferGeometry": ({
/** One linear RGB or RGBA colour per vertex. */
readonly "colors"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Counter-clockwise triangle indices; absent uses consecutive vertices. */
readonly "indices"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** One xyz normal per vertex; absent computes normals. */
readonly "normals"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Finite x/y/z vertex positions in metres. Triangle count, attribute
counts and index bounds are checked together by the domain validator. */
readonly "positions": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** One xyzw tangent frame per vertex; w is handedness. */
readonly "tangents"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** One uv pair per vertex; v down the image. */
readonly "uvs"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
});
readonly "CapsuleGeometry": ({
readonly "capSegments"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Straight middle length, not total capsule length. */
readonly "height"?: ((number) | (null));
readonly "heightSegments"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Channel": ({
readonly "interpolation"?: (((InputDefinitions["Interpolation"])) | ((null)));
readonly "keys": (ReadonlyArray<(InputDefinitions["Key"])> | (number extends ((InputDefinitions["Key"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "node": (string);
readonly "path": (InputDefinitions["ChannelPath"]);
});
readonly "ChannelPath": ("translation" | "rotation" | "scale" | "weights" | "visibility") & (string);
readonly "CircleGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
readonly "segments"?: ((number) | (null));
readonly "thetaLength"?: ((number) | (null));
readonly "thetaStart"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Clip": ({
readonly "channels": (ReadonlyArray<(InputDefinitions["Channel"])> | (number extends ((InputDefinitions["Channel"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
});
readonly "Collider": (((boolean)) | ((InputDefinitions["ColliderMethod"])));
readonly "ColliderMethod": ("convex" | "vhacd") & (string);
readonly "ConeGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "height"?: ((number) | (null));
readonly "heightSegments"?: ((number) | (null));
readonly "openEnded"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
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
readonly "CsgGeometry": ({
readonly "a": (InputDefinitions["PlacedGeometry"]);
readonly "b": (InputDefinitions["PlacedGeometry"]);
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "op"?: (((InputDefinitions["CsgOp"])) | ((null)));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "CsgOp": ("union" | "subtract" | "intersect") & (string);
readonly "Curve": ({
readonly "closed"?: ((boolean) | (null));
readonly "kind"?: (((InputDefinitions["CurveKind"])) | ((null)));
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Catmull-Rom tangent scale; unset 0.5. */
readonly "tension"?: ((number) | (null));
});
readonly "CurveKind": ("catmullRom" | "bezier" | "linear") & (string);
readonly "CylinderGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "height"?: ((number) | (null));
readonly "heightSegments"?: ((number) | (null));
readonly "openEnded"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radiusBottom"?: ((number) | (null));
readonly "radiusTop"?: ((number) | (null));
readonly "thetaLength"?: ((number) | (null));
readonly "thetaStart"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
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
readonly "Ellipse": ({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "clockwise"?: (boolean);
readonly "end": (number);
readonly "radii": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "rotation"?: (number);
readonly "start": (number);
});
readonly "Event": ({
readonly "clip": (string);
readonly "marker": (string);
readonly "time": (number);
});
readonly "ExtrudeGeometry": ({
readonly "bevelEnabled"?: ((boolean) | (null));
readonly "bevelOffset"?: ((number) | (null));
readonly "bevelSegments"?: ((number) | (null));
/** Unset is bevelThickness - 0.1, not a second fixed default. */
readonly "bevelSize"?: ((number) | (null));
readonly "bevelThickness"?: ((number) | (null));
readonly "curveSegments"?: ((number) | (null));
readonly "depth"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "extrudePath"?: (((InputDefinitions["Curve"])) | ((null)));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Unset uses the generator's default square. */
readonly "shapes"?: ((ReadonlyArray<(InputDefinitions["Shape"])> | (number extends ((InputDefinitions["Shape"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "steps"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Flowers": ({
readonly "centerColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "color2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "count"?: ((number) | (null));
readonly "cup"?: ((number) | (null));
readonly "petals"?: ((number) | (null));
readonly "share"?: ((number) | (null));
readonly "size"?: ((number) | (null));
readonly "width"?: ((number) | (null));
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
/** Exactly one geometry, named as the corresponding render mechanism. */
readonly "Geometry": ((({
readonly "desc": (InputDefinitions["ModelGeometry"]);
readonly "kind": ("modelGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["BufferGeometry"]);
readonly "kind": ("bufferGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["BoxGeometry"]);
readonly "kind": ("boxGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["PlaneGeometry"]);
readonly "kind": ("planeGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["SphereGeometry"]);
readonly "kind": ("sphereGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["CylinderGeometry"]);
readonly "kind": ("cylinderGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["ConeGeometry"]);
readonly "kind": ("coneGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["CapsuleGeometry"]);
readonly "kind": ("capsuleGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["TorusGeometry"]);
readonly "kind": ("torusGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["RingGeometry"]);
readonly "kind": ("ringGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["CircleGeometry"]);
readonly "kind": ("circleGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["TubeGeometry"]);
readonly "kind": ("tubeGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["ShapeGeometry"]);
readonly "kind": ("shapeGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["ExtrudeGeometry"]);
readonly "kind": ("extrudeGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["LatheGeometry"]);
readonly "kind": ("latheGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["PolyhedronGeometry"]);
readonly "kind": ("polyhedronGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["SolidGeometry"]);
readonly "kind": ("icosahedronGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["SolidGeometry"]);
readonly "kind": ("dodecahedronGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["SolidGeometry"]);
readonly "kind": ("octahedronGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["SolidGeometry"]);
readonly "kind": ("tetrahedronGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["SweepGeometry"]);
readonly "kind": ("sweepGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["CsgGeometry"]);
readonly "kind": ("csgGeometry") & (string);
})) | (({
readonly "desc": (InputDefinitions["MergeGeometries"]);
readonly "kind": ("mergeGeometries") & (string);
})));
readonly "GeometryModel": ({
readonly "conform"?: (((InputDefinitions["Conform"])) | ((null)));
readonly "fracture"?: (((InputDefinitions["Fracture"])) | ((null)));
/** Ordered declarations: a named dependency must precede its consumer. */
readonly "items": (ReadonlyArray<(InputDefinitions["Item"])> | (number extends ((InputDefinitions["Item"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "options"?: (((InputDefinitions["ModelOptions"])) | ((null)));
readonly "process"?: (((InputDefinitions["Process"])) | ((null)));
});
readonly "Growth": ("deciduous" | "evergreen") & (string);
readonly "Impact": ({
readonly "position": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "radius": (number);
});
readonly "Interpolation": ("step" | "linear" | "cubicSpline") & (string);
readonly "Item": ({
readonly "input": (InputDefinitions["ItemInput"]);
readonly "name": (string);
});
readonly "ItemInput": ((({
readonly "desc": (InputDefinitions["Material"]);
readonly "kind": ("material") & (string);
})) | (({
readonly "desc": (InputDefinitions["Pivot"]);
readonly "kind": ("pivot") & (string);
})) | (({
readonly "desc": (InputDefinitions["Part"]);
readonly "kind": ("part") & (string);
})) | (({
readonly "desc": (InputDefinitions["Plant"]);
readonly "kind": ("plant") & (string);
})));
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
readonly "LatheGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "phiLength"?: ((number) | (null));
readonly "phiStart"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Radial x and height y, metres; unset uses the generator's profile. */
readonly "points"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "segments"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "LeafShape": ("blade" | "card" | "cross") & (string);
readonly "Leaves": ({
readonly "angle"?: ((number) | (null));
readonly "autumnColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "autumnColor2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "color2"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "colorVariation"?: ((number) | (null));
readonly "count"?: ((number) | (null));
readonly "domeNormals"?: ((number) | (null));
readonly "end"?: ((number) | (null));
readonly "facing"?: ((number) | (null));
readonly "material"?: ((string) | (null));
readonly "rows"?: ((number) | (null));
readonly "season"?: ((number) | (null));
readonly "seasonShift"?: ((number) | (null));
readonly "shape"?: (((InputDefinitions["LeafShape"])) | ((null)));
readonly "size"?: ((number) | (null));
readonly "sizeVariance"?: ((number) | (null));
readonly "start"?: ((number) | (null));
readonly "translucency"?: ((number) | (null));
/** Width as a share of leaf length. */
readonly "width"?: ((number) | (null));
});
readonly "Level": ({
readonly "angle"?: ((number) | (null));
readonly "children"?: ((number) | (null));
readonly "gnarliness"?: ((number) | (null));
readonly "length"?: ((number) | (null));
/** Trunk radius in metres; branch radius as a share of the parent's. */
readonly "radius"?: ((number) | (null));
readonly "sections"?: ((number) | (null));
readonly "segments"?: ((number) | (null));
readonly "start"?: ((number) | (null));
readonly "taper"?: ((number) | (null));
readonly "twist"?: ((number) | (null));
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
readonly "MergeGeometries": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "parts": (ReadonlyArray<(InputDefinitions["PlacedGeometry"])> | (number extends ((InputDefinitions["PlacedGeometry"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Mirror": ("none" | "x" | "y" | "z") & (string);
readonly "Mirror2": ({
readonly "clip": (string);
readonly "name"?: ((string) | (null));
});
readonly "ModelGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Select a mesh node, or merge the visible default scene at rest. */
readonly "node"?: ((string) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** A model cooked before this geometry; dependency cycles are domain errors. */
readonly "source": (InputDefinitions["ContentRef"]);
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "ModelOptions": ({
readonly "animation"?: (((InputDefinitions["Animation"])) | ((null)));
readonly "collider"?: (((InputDefinitions["Collider"])) | ((null)));
readonly "impostor"?: ((boolean) | (null));
readonly "impostorBelow"?: ((boolean) | (null));
readonly "impostorGrid"?: ((number) | (null));
/** Atlas width (grid × frame size) must not exceed 4096 pixels. */
readonly "impostorSize"?: ((number) | (null));
readonly "lodError"?: ((number) | (null));
/** At most eight strictly decreasing fractions beginning at one. */
readonly "lods"?: ((ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "raycast"?: ((boolean) | (null));
/** Opt-in probe-region skin relaxation (12 passes); absent means off. */
readonly "relaxSkin"?: ((boolean) | (null));
readonly "streamTextures"?: ((boolean) | (null));
});
readonly "Part": ({
readonly "geometry": (InputDefinitions["Geometry"]);
readonly "material"?: ((string) | (null));
readonly "pivot"?: ((string) | (null));
/** Omitted: identity placement. Explicit placements must be nonempty; a
single placement also represents the old part's own TRS, with no XOR. */
readonly "placements"?: ((ReadonlyArray<(InputDefinitions["Placement"])> | (number extends ((InputDefinitions["Placement"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
});
/** Ordered Path2 operations. These are structured calls, not embedded OGT. */
readonly "Path2Command": ((({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "op": ("moveTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "op": ("lineTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 4 });
readonly "op": ("quadraticCurveTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(number)> & { readonly length: 6 });
readonly "op": ("bezierCurveTo") & (string);
})) | (({
readonly "args": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "op": ("splineThru") & (string);
})) | (({
readonly "args": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "op": ("points") & (string);
})) | (({
readonly "args": (InputDefinitions["Arc"]);
readonly "op": ("arc") & (string);
})) | (({
readonly "args": (InputDefinitions["Arc"]);
readonly "op": ("absarc") & (string);
})) | (({
readonly "args": (InputDefinitions["Ellipse"]);
readonly "op": ("ellipse") & (string);
})) | (({
readonly "args": (InputDefinitions["Ellipse"]);
readonly "op": ("absellipse") & (string);
})) | ((({
readonly "op": ("closePath") & (string);
})) & { readonly "args"?: never; }));
readonly "Pattern": ("auto" | "volume" | "slab") & (string);
readonly "Pivot": ({
/** Previously declared pivot. Positive scale and nonsingular rotation are
model-domain constraints; reflected Transform is not duplicated here. */
readonly "parent"?: ((string) | (null));
readonly "transform"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::world::geometry::Transform"])) | ((null)));
});
/** One CSG operand or merged part. The transform is the actual reflected
engine component, not a copied table of position/quaternion/scale fields. */
readonly "PlacedGeometry": ({
readonly "geometry": (InputDefinitions["Geometry"]);
readonly "mirror"?: (((InputDefinitions["Mirror"])) | ((null)));
readonly "transform"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::world::geometry::Transform"])) | ((null)));
});
readonly "Placement": ({
/** Reflections bake into geometry with corrected triangle winding. */
readonly "mirror"?: (((InputDefinitions["Mirror"])) | ((null)));
readonly "transform"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::world::geometry::Transform"])) | ((null)));
});
readonly "PlaneGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Y extent, metres; unset 1. */
readonly "height"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** X extent, metres; unset 1. The plane faces +Z. */
readonly "width"?: ((number) | (null));
});
readonly "Plant": ({
readonly "barkColor"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Model-space minimum/maximum, metres; ordering is a domain check. */
readonly "bounds"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> & { readonly length: 2 }) | (null));
readonly "branch1"?: (((InputDefinitions["Level"])) | ((null)));
readonly "branch2"?: (((InputDefinitions["Level"])) | ((null)));
readonly "branch3"?: (((InputDefinitions["Level"])) | ((null)));
readonly "branches"?: (ReadonlyArray<(InputDefinitions["Branch"])> | (number extends ((InputDefinitions["Branch"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "flowerGeometry"?: (((InputDefinitions["Geometry"])) | ((null)));
readonly "flowers"?: (((InputDefinitions["Flowers"])) | ((null)));
/** Direction xyz and bending strength. */
readonly "force"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "growth"?: (((InputDefinitions["Growth"])) | ((null)));
readonly "kind"?: (((InputDefinitions["PlantKind"])) | ((null)));
readonly "leafGeometry"?: (((InputDefinitions["Geometry"])) | ((null)));
readonly "leaves"?: (((InputDefinitions["Leaves"])) | ((null)));
readonly "levels"?: ((number) | (null));
readonly "material"?: ((string) | (null));
readonly "path"?: (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "seed"?: ((number) | (null));
readonly "trunk"?: (((InputDefinitions["Level"])) | ((null)));
});
readonly "PlantKind": ("oak" | "birch" | "willow" | "pine" | "palm" | "bamboo" | "cactus" | "shrub" | "fern" | "reed" | "daisy" | "tulip") & (string);
readonly "PolyhedronGeometry": ({
readonly "detail"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** At least one triangle, indexing the corner triples. */
readonly "indices": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Flat xyz corner coordinates, metres. */
readonly "vertices": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Process": ({
readonly "bake"?: (((InputDefinitions["Bake"])) | ((null)));
readonly "decimate"?: (((InputDefinitions["Decimate"])) | ((null)));
readonly "project"?: (((InputDefinitions["Project"])) | ((null)));
readonly "repair"?: (((InputDefinitions["Repair"])) | ((null)));
readonly "rig"?: (((InputDefinitions["Rig"])) | ((null)));
readonly "transfer"?: (((InputDefinitions["Transfer"])) | ((null)));
readonly "unwrap"?: (((InputDefinitions["Unwrap"])) | ((null)));
});
/** Program material overrides have their own source semantics (sheen and
roughness/emissive maps); they are not the procedural material's field set. */
readonly "ProgramMaterial": ({
readonly "alphaTest"?: ((number) | (null));
readonly "aoMap"?: ((string) | (null));
readonly "aoMapIntensity"?: ((number) | (null));
readonly "clearcoat"?: ((number) | (null));
readonly "clearcoatRoughness"?: ((number) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "doubleSided"?: ((boolean) | (null));
readonly "emissive"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "emissiveIntensity"?: ((number) | (null));
readonly "emissiveMap"?: ((string) | (null));
readonly "iridescence"?: ((number) | (null));
readonly "map"?: ((string) | (null));
readonly "metalness"?: ((number) | (null));
readonly "name": (string);
readonly "normalMap"?: ((string) | (null));
readonly "normalScale"?: ((number) | (null));
readonly "opacity"?: ((number) | (null));
readonly "roughness"?: ((number) | (null));
readonly "roughnessMap"?: ((string) | (null));
readonly "sheen"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "sheenRoughness"?: ((number) | (null));
readonly "transmission"?: ((number) | (null));
readonly "transparent"?: ((boolean) | (null));
});
readonly "ProgramModel": ({
readonly "atlas"?: ((number) | (null));
readonly "conform"?: (((InputDefinitions["Conform"])) | ((null)));
readonly "materials"?: (ReadonlyArray<(InputDefinitions["ProgramMaterial"])> | (number extends ((InputDefinitions["ProgramMaterial"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "options"?: (((InputDefinitions["ModelOptions"])) | ((null)));
readonly "params"?: (Readonly<Record<string, (number)>>);
readonly "process"?: (((InputDefinitions["Process"])) | ((null)));
readonly "program": (InputDefinitions["ContentRef"]);
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
readonly "Reflected_og_cook::cook::source_schemas::world::geometry::Transform": ({
/** The position, as three.js Object3D.position. */
readonly "position"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The rotation, as three.js Object3D.quaternion. */
readonly "quaternion"?: (ReadonlyArray<(number)> & { readonly length: 4 });
/** The scale, as three.js Object3D.scale. */
readonly "scale"?: (ReadonlyArray<(number)> & { readonly length: 3 });
});
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
readonly "RingGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "innerRadius"?: ((number) | (null));
readonly "outerRadius"?: ((number) | (null));
readonly "phiSegments"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "thetaLength"?: ((number) | (null));
readonly "thetaSegments"?: ((number) | (null));
readonly "thetaStart"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Shape": ({
readonly "commands": (ReadonlyArray<(InputDefinitions["Path2Command"])> | (number extends ((InputDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Hole contours cannot contain more holes. */
readonly "holes"?: (ReadonlyArray<(ReadonlyArray<(InputDefinitions["Path2Command"])> | (number extends ((InputDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))> | (number extends ((ReadonlyArray<(InputDefinitions["Path2Command"])> | (number extends ((InputDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "ShapeGeometry": ({
readonly "curveSegments"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Unset uses the generator's default triangle. */
readonly "shapes"?: ((ReadonlyArray<(InputDefinitions["Shape"])> | (number extends ((InputDefinitions["Shape"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "SolidGeometry": ({
readonly "detail"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "SphereGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "heightSegments"?: ((number) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radius"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "widthSegments"?: ((number) | (null));
});
readonly "Style": ("voronoi" | "concrete" | "brick" | "wood" | "glass") & (string);
readonly "SweepGeometry": ({
readonly "caps"?: ((boolean) | (null));
readonly "curveSegments"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
readonly "path": (InputDefinitions["Curve"]);
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
/** Ordered interpolation keys along the path (t, scale). */
readonly "scale"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Cross-section without holes. Curve flattening uses curveSegments. */
readonly "section": (ReadonlyArray<(InputDefinitions["Path2Command"])> | (number extends ((InputDefinitions["Path2Command"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "steps"?: ((number) | (null));
/** Ordered interpolation keys (t, radians). */
readonly "twist"?: ((ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "TorusGeometry": ({
readonly "arc"?: ((number) | (null));
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
readonly "tube"?: ((number) | (null));
readonly "tubularSegments"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "Transfer": ({
readonly "padding"?: ((number) | (null));
readonly "size"?: ((number) | (null));
readonly "source": (InputDefinitions["ContentRef"]);
});
readonly "TubeGeometry": ({
/** Runtime descriptor flag; a cooked model is static source data. */
readonly "dynamic"?: ((boolean) | (null));
/** A sampled path or a Catmull-Rom point list, never a runtime Path handle. */
readonly "path": (InputDefinitions["TubePath"]);
/** Runtime descriptor flag; model import settings own cooked collision. */
readonly "physics"?: ((boolean) | (null));
readonly "radialSegments"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
readonly "tubularSegments"?: ((number) | (null));
/** Scale the imported UVs. Unset means [1, 1]. */
readonly "uvRepeat"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
});
readonly "TubePath": (((({
readonly "closed"?: ((boolean) | (null));
readonly "kind": ("points") & (string);
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "curve"?: never; }) | ((({
readonly "curve": (InputDefinitions["Curve"]);
readonly "kind": ("curve") & (string);
})) & { readonly "closed"?: never; readonly "points"?: never; }));
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
}
```
