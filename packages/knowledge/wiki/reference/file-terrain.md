<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-terrain"></a>
### `file:terrain`

Checked `file["terrain"](input)` declaration from `@origingame/engine/author`.

```ts
/** A terrain's source, processing and material declarations. Sample counts,
ascending bands, channels, layer indices and projected edits remain domain
validation; this does not run or replace the existing terrain cooker. */
export type Input = ({
readonly "albedoArray"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "castShadows"?: ((boolean) | (null));
readonly "collision"?: ((boolean) | (null));
/** RGBA control images, one per four layers. Exclusive with biome rules. */
readonly "control"?: ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
/** Brushes and stamps in their written order, after generation/processing. */
readonly "edits"?: ((ReadonlyArray<(InputDefinitions["TerrainEdit"])> | (number extends ((InputDefinitions["TerrainEdit"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "heightBlend"?: ((number) | (null));
/** Raw hole image relative to the owning module: black/transparent cuts cells. */
readonly "holes"?: ((string) | (null));
readonly "layers"?: ((ReadonlyArray<(InputDefinitions["TerrainLayerInput"])> | (number extends ((InputDefinitions["TerrainLayerInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "lod"?: (((InputDefinitions["TerrainLod"])) | ((null)));
readonly "normalArray"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "process"?: (((InputDefinitions["TerrainProcess"])) | ((null)));
/** Shared seed for the noise source and seeded processing stages. */
readonly "seed"?: ((number) | (null));
/** Positive world extents along X and Z, in metres. */
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "source": (InputDefinitions["TerrainSource"]);
readonly "tile"?: ((number) | (null));
});
export interface InputDefinitions {
readonly "Arc": ({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "clockwise"?: (boolean);
readonly "end": (number);
readonly "radius": (number);
readonly "start": (number);
});
readonly "BiomeInput": ({
readonly "altitude"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "altitudeFade"?: ((number) | (null));
readonly "layer": (number);
readonly "moisture"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "moistureFade"?: ((number) | (null));
readonly "slope"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "slopeFade"?: ((number) | (null));
readonly "weight"?: ((number) | (null));
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
readonly "BrushMode": ("raise" | "lower" | "flatten" | "smooth" | "paint" | "hole" | "fill") & (string);
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
readonly "Ellipse": ({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "clockwise"?: (boolean);
readonly "end": (number);
readonly "radii": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "rotation"?: (number);
readonly "start": (number);
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
/** A heightfield uses metres, not a normalised 0..1 image. Range mapping is
exclusively the terrain file's explicit `height` setting. */
readonly "Heightfield": ({
/** Sample columns; 2..4097. Together with rows, fixes the height count. */
readonly "columns": (number);
/** Finite row-major heights in metres; never normalised or clipped. */
readonly "heights": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Sample rows; 2..4097, ordered from -z to +z. */
readonly "rows": (number);
});
readonly "HydraulicInput": ({
readonly "capacity"?: ((number) | (null));
readonly "deposition"?: ((number) | (null));
readonly "drops"?: ((number) | (null));
readonly "erosion"?: ((number) | (null));
readonly "evaporation"?: ((number) | (null));
readonly "gravity"?: ((number) | (null));
readonly "inertia"?: ((number) | (null));
readonly "lifetime"?: ((number) | (null));
readonly "radius"?: ((number) | (null));
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
readonly "MoistureInput": ({
readonly "flow"?: ((number) | (null));
readonly "frequency"?: ((number) | (null));
readonly "noise"?: ((number) | (null));
readonly "scale"?: ((number) | (null));
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
/** One CSG operand or merged part. The transform is the actual reflected
engine component, not a copied table of position/quaternion/scale fields. */
readonly "PlacedGeometry": ({
readonly "geometry": (InputDefinitions["Geometry"]);
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
readonly "Reflected_og_cook::cook::source_schemas::procedural::terrain::Transform": ({
/** The position, as three.js Object3D.position. */
readonly "position"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The rotation, as three.js Object3D.quaternion. */
readonly "quaternion"?: (ReadonlyArray<(number)> & { readonly length: 4 });
/** The scale, as three.js Object3D.scale. */
readonly "scale"?: (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "Reflected_og_cook::cook::source_schemas::world::geometry::Transform": ({
/** The position, as three.js Object3D.position. */
readonly "position"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The rotation, as three.js Object3D.quaternion. */
readonly "quaternion"?: (ReadonlyArray<(number)> & { readonly length: 4 });
/** The scale, as three.js Object3D.scale. */
readonly "scale"?: (ReadonlyArray<(number)> & { readonly length: 3 });
});
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
readonly "RiversInput": ({
readonly "depth"?: ((number) | (null));
readonly "threshold"?: ((number) | (null));
readonly "width"?: ((number) | (null));
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
readonly "StampMode": ("carve" | "hole") & (string);
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
readonly "TerracesInput": ({
readonly "blend"?: ((number) | (null));
readonly "step"?: ((number) | (null));
});
readonly "TerrainChannel": ({
readonly "depth"?: ((number) | (null));
/** Consecutive XZ points must differ; river Y must not rise downstream. */
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "river"?: ((boolean) | (null));
readonly "width": (number);
});
readonly "TerrainEdit": (((({
readonly "center": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "falloff"?: ((number) | (null));
readonly "kind": ("brush") & (string);
readonly "layer"?: ((number) | (null));
readonly "mode": (InputDefinitions["BrushMode"]);
readonly "radius": (number);
/** Raise/lower use metres; other brush modes clamp strength to one. */
readonly "strength"?: ((number) | (null));
})) & { readonly "depth"?: never; readonly "geometry"?: never; readonly "transform"?: never; }) | ((({
readonly "depth"?: ((number) | (null));
readonly "geometry": (InputDefinitions["Geometry"]);
readonly "kind": ("stamp") & (string);
readonly "mode"?: (((InputDefinitions["StampMode"])) | ((null)));
/** Existing Transform reflection supplies placement, never a copied field table. */
readonly "transform"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::procedural::terrain::Transform"])) | ((null)));
})) & { readonly "center"?: never; readonly "falloff"?: never; readonly "layer"?: never; readonly "radius"?: never; readonly "strength"?: never; }));
readonly "TerrainLayerInput": ({
readonly "albedo"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "albedoLayer"?: ((number) | (null));
readonly "altitude"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "altitudeFade"?: ((number) | (null));
readonly "antiTiling"?: ((number) | (null));
readonly "cellVariation"?: ((number) | (null));
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "displacement"?: ((number) | (null));
readonly "heightContrast"?: ((number) | (null));
readonly "macroVariation"?: ((number) | (null));
readonly "normal"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "normalLayer"?: ((number) | (null));
readonly "normalStrength"?: ((number) | (null));
readonly "projectSlope"?: ((number) | (null));
readonly "roughness"?: ((number) | (null));
readonly "slope"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "slopeFade"?: ((number) | (null));
readonly "tile"?: ((number) | (null));
readonly "triplanar"?: ((boolean) | (null));
});
readonly "TerrainLod": ({
readonly "ring"?: ((number) | (null));
/** Zero uses the source sample spacing. */
readonly "spacing"?: ((number) | (null));
});
readonly "TerrainMask": (((({
readonly "kind": ("none") & (string);
})) & { readonly "falloff"?: never; readonly "strength"?: never; }) | ((({
readonly "falloff"?: ((number) | (null));
readonly "kind": ("island") & (string);
})) & { readonly "strength"?: never; }) | ((({
readonly "kind": ("ridge") & (string);
readonly "strength"?: ((number) | (null));
})) & { readonly "falloff"?: never; }));
readonly "TerrainNoise": ({
/** Noise features across the longer world extent. */
readonly "frequency"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "lacunarity"?: ((number) | (null));
readonly "octaves"?: ((number) | (null));
readonly "warp"?: ((number) | (null));
});
/** Processing runs in domain order: hydraulic, thermal, terraces, shore,
drainage, authored channels and biome controls. It never evaluates TS. */
readonly "TerrainProcess": ({
readonly "biomes"?: ((ReadonlyArray<(InputDefinitions["BiomeInput"])> | (number extends ((InputDefinitions["BiomeInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "channels"?: ((ReadonlyArray<(InputDefinitions["TerrainChannel"])> | (number extends ((InputDefinitions["TerrainChannel"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "erosion"?: (((InputDefinitions["HydraulicInput"])) | ((null)));
readonly "moisture"?: (((InputDefinitions["MoistureInput"])) | ((null)));
readonly "preserve"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "preserveBorder"?: ((number) | (null));
readonly "rivers"?: (((InputDefinitions["RiversInput"])) | ((null)));
readonly "sea"?: ((number) | (null));
readonly "shore"?: ((number) | (null));
readonly "terraces"?: (((InputDefinitions["TerracesInput"])) | ((null)));
readonly "thermal"?: (((InputDefinitions["ThermalInput"])) | ((null)));
});
readonly "TerrainSource": (((({
/** Metres corresponding to image intensities zero and one. */
readonly "height"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("image") & (string);
/** PNG/EXR source relative to the owning module; first channel is height. */
readonly "source": (string);
})) & { readonly "columns"?: never; readonly "field"?: never; readonly "mask"?: never; readonly "noise"?: never; readonly "rows"?: never; }) | ((({
/** The shared heightfield's domain validator checks rows × columns and finiteness. */
readonly "field": (InputDefinitions["Heightfield"]);
readonly "kind": ("heightfield") & (string);
})) & { readonly "columns"?: never; readonly "height"?: never; readonly "mask"?: never; readonly "noise"?: never; readonly "rows"?: never; readonly "source"?: never; }) | ((({
readonly "columns"?: ((number) | (null));
readonly "height"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("noise") & (string);
readonly "mask"?: (((InputDefinitions["TerrainMask"])) | ((null)));
readonly "noise"?: (((InputDefinitions["TerrainNoise"])) | ((null)));
readonly "rows"?: ((number) | (null));
})) & { readonly "field"?: never; readonly "source"?: never; }));
readonly "ThermalInput": ({
readonly "iterations"?: ((number) | (null));
readonly "rate"?: ((number) | (null));
readonly "talus"?: ((number) | (null));
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
}
```
