<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-navmesh"></a>
### `file:navmesh`

Checked `file["navmesh"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
/** Ordered profiles; unnamed profile is the default, otherwise the first. */
readonly "agents"?: (ReadonlyArray<(InputDefinitions["Agent"])> | (number extends ((InputDefinitions["Agent"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Later shapes override earlier overlaps. */
readonly "areas"?: (ReadonlyArray<(InputDefinitions["Area"])> | (number extends ((InputDefinitions["Area"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "bounds"?: (((InputDefinitions["Bounds"])) | ((null)));
readonly "boxes"?: (ReadonlyArray<(InputDefinitions["NavBox"])> | (number extends ((InputDefinitions["NavBox"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "cellHeight"?: ((number) | (null));
/** Voxel dimensions in metres; omitted: 0.2 horizontally, 0.1 vertically. */
readonly "cellSize"?: ((number) | (null));
readonly "links"?: (ReadonlyArray<(InputDefinitions["Link"])> | (number extends ((InputDefinitions["Link"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Contour simplification error; omitted: 1.3. */
readonly "maxError"?: ((number) | (null));
readonly "models"?: (ReadonlyArray<(InputDefinitions["NavModel"])> | (number extends ((InputDefinitions["NavModel"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "scene"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "terrains"?: (ReadonlyArray<(InputDefinitions["NavTerrain"])> | (number extends ((InputDefinitions["NavTerrain"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Clamped to 8..256 and truncated by normalization; omitted: 32. */
readonly "tileSize"?: ((number) | (null));
});
export interface InputDefinitions {
readonly "Agent": ({
readonly "climb"?: ((number) | (null));
readonly "height"?: ((number) | (null));
readonly "name"?: ((string) | (null));
/** Metres; omitted radius/height/climb: 0.4/1.8/0.4. */
readonly "radius"?: ((number) | (null));
/** Maximum walkable slope; omitted: 45 degrees. */
readonly "slope"?: ((number) | (null));
});
readonly "Area": ({
readonly "id": (number);
readonly "shape": (InputDefinitions["AreaShape"]);
});
/** Explicit branch selection replaces the parser's radius/point precedence. */
readonly "AreaShape": (((({
readonly "center"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "kind": ("box") & (string);
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "yaw"?: ((number) | (null));
})) & { readonly "bottom"?: never; readonly "height"?: never; readonly "points"?: never; readonly "radius"?: never; readonly "top"?: never; }) | ((({
readonly "center"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "height"?: ((number) | (null));
readonly "kind": ("cylinder") & (string);
/** Metres; negative values clamp to zero during normalization. */
readonly "radius": (number);
})) & { readonly "bottom"?: never; readonly "points"?: never; readonly "size"?: never; readonly "top"?: never; readonly "yaw"?: never; }) | ((({
readonly "bottom"?: ((number) | (null));
readonly "kind": ("convex") & (string);
/** x/z coordinates in metres; vertical bounds are bottom/top. */
readonly "points": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "top"?: ((number) | (null));
})) & { readonly "center"?: never; readonly "height"?: never; readonly "radius"?: never; readonly "size"?: never; readonly "yaw"?: never; }));
readonly "Bounds": ({
readonly "max"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "min"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
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
readonly "Link": ({
readonly "bidirectional"?: ((boolean) | (null));
/** Negative cost/duration clamp to zero; defaults are zero. */
readonly "cost"?: ((number) | (null));
readonly "duration"?: ((number) | (null));
readonly "end": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "id": (number);
readonly "kind"?: (((InputDefinitions["LinkKind"])) | ((null)));
readonly "open"?: ((boolean) | (null));
readonly "start": (ReadonlyArray<(number)> & { readonly length: 3 });
readonly "traverse"?: (((InputDefinitions["Traverse"])) | ((null)));
});
readonly "LinkKind": ("jump" | "ladder" | "door" | "teleport") & (string);
readonly "NavBox": ({
readonly "center"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Normalized by the domain; zero-length rotations are invalid. */
readonly "quaternion"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
});
readonly "NavModel": ({
readonly "model": (InputDefinitions["ContentRef"]);
readonly "transform"?: (((InputDefinitions["Reflected_og_cook::cook::source_schemas::world::geometry::Transform"])) | ((null)));
});
readonly "NavTerrain": ({
readonly "position"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Sampling stride; normalization clamps to >=1 and truncates; omitted: 1. */
readonly "step"?: ((number) | (null));
readonly "terrain": (InputDefinitions["ContentRef"]);
});
readonly "Reflected_og_cook::cook::source_schemas::world::geometry::Transform": ({
/** The position, as three.js Object3D.position. */
readonly "position"?: (ReadonlyArray<(number)> & { readonly length: 3 });
/** The rotation, as three.js Object3D.quaternion. */
readonly "quaternion"?: (ReadonlyArray<(number)> & { readonly length: 4 });
/** The scale, as three.js Object3D.scale. */
readonly "scale"?: (ReadonlyArray<(number)> & { readonly length: 3 });
});
readonly "Traverse": ("engine" | "game") & (string);
}
```
