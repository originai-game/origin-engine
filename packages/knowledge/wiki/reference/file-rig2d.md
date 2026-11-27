<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-rig2d"></a>
### `file:rig2d`

Checked `file["rig2d"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ((((InputDefinitions["Authored"])) & { readonly "atlas"?: never; readonly "dragonbones"?: never; readonly "parts"?: never; readonly "pivots"?: never; readonly "template"?: never; }) | (((InputDefinitions["Dragonbones"])) & { readonly "armatures"?: never; readonly "parts"?: never; readonly "pivots"?: never; readonly "rig"?: never; readonly "template"?: never; }) | (((InputDefinitions["Parts"])) & { readonly "armatures"?: never; readonly "atlas"?: never; readonly "dragonbones"?: never; readonly "rig"?: never; readonly "texture"?: never; }));
export interface InputDefinitions {
readonly "Armature": ({
readonly "attachments"?: (Readonly<Record<string, (InputDefinitions["Attachment"])>>);
readonly "bones": (ReadonlyArray<(InputDefinitions["Bone"])> | (number extends ((InputDefinitions["Bone"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "clips"?: (ReadonlyArray<(InputDefinitions["Clip"])> | (number extends ((InputDefinitions["Clip"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "iks"?: (ReadonlyArray<(InputDefinitions["Ik"])> | (number extends ((InputDefinitions["Ik"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "paths"?: (ReadonlyArray<(InputDefinitions["PathConstraint"])> | (number extends ((InputDefinitions["PathConstraint"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Slot attachment swaps, names resolved in this armature. */
readonly "skins"?: (Readonly<Record<string, (Readonly<Record<string, ((string) | (null))>>)>>);
readonly "slots"?: (ReadonlyArray<(InputDefinitions["Slot"])> | (number extends ((InputDefinitions["Slot"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "transforms"?: (ReadonlyArray<(InputDefinitions["TransformConstraint"])> | (number extends ((InputDefinitions["TransformConstraint"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Attachment": (((({
readonly "at"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "kind": ("region") & (string);
readonly "rect": (ReadonlyArray<(number)> & { readonly length: 4 });
readonly "rotation"?: ((number) | (null));
readonly "size"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "skew"?: ((number) | (null));
readonly "slot": (string);
})) & { readonly "armature"?: never; readonly "clip"?: never; readonly "closed"?: never; readonly "constant"?: never; readonly "lengths"?: never; readonly "points"?: never; readonly "polygon"?: never; readonly "scale"?: never; readonly "triangles"?: never; readonly "vertices"?: never; }) | ((({
readonly "kind": ("mesh") & (string);
readonly "slot": (string);
readonly "triangles": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "vertices": (ReadonlyArray<(InputDefinitions["Vertex"])> | (number extends ((InputDefinitions["Vertex"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "armature"?: never; readonly "at"?: never; readonly "clip"?: never; readonly "closed"?: never; readonly "constant"?: never; readonly "lengths"?: never; readonly "points"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotation"?: never; readonly "scale"?: never; readonly "size"?: never; readonly "skew"?: never; }) | ((({
readonly "kind": ("box") & (string);
readonly "polygon": (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 2 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 2 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "slot": (string);
})) & { readonly "armature"?: never; readonly "at"?: never; readonly "clip"?: never; readonly "closed"?: never; readonly "constant"?: never; readonly "lengths"?: never; readonly "points"?: never; readonly "rect"?: never; readonly "rotation"?: never; readonly "scale"?: never; readonly "size"?: never; readonly "skew"?: never; readonly "triangles"?: never; readonly "vertices"?: never; }) | ((({
readonly "closed": (boolean);
readonly "constant": (boolean);
readonly "kind": ("path") & (string);
readonly "lengths": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "points": (ReadonlyArray<(ReadonlyArray<(InputDefinitions["Influence"])> | (number extends ((InputDefinitions["Influence"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))> | (number extends ((ReadonlyArray<(InputDefinitions["Influence"])> | (number extends ((InputDefinitions["Influence"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "slot": (string);
})) & { readonly "armature"?: never; readonly "at"?: never; readonly "clip"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "rotation"?: never; readonly "scale"?: never; readonly "size"?: never; readonly "skew"?: never; readonly "triangles"?: never; readonly "vertices"?: never; }) | ((({
readonly "armature": (string);
readonly "at"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "clip"?: ((string) | (null));
readonly "kind": ("child") & (string);
readonly "rotation"?: ((number) | (null));
readonly "scale"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "skew"?: ((number) | (null));
readonly "slot": (string);
})) & { readonly "closed"?: never; readonly "constant"?: never; readonly "lengths"?: never; readonly "points"?: never; readonly "polygon"?: never; readonly "rect"?: never; readonly "size"?: never; readonly "triangles"?: never; readonly "vertices"?: never; }));
readonly "Authored": ({
readonly "armatures"?: (Readonly<Record<string, (InputDefinitions["Armature"])>>);
readonly "conform"?: (((InputDefinitions["TilesetConform"])) | ((null)));
readonly "filter"?: (((InputDefinitions["Filter"])) | ((null)));
readonly "rig": (InputDefinitions["Armature"]);
readonly "texture": (InputDefinitions["ContentRef"]);
readonly "unit"?: ((number) | (null));
});
readonly "Bone": ({
readonly "at"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "length"?: ((number) | (null));
readonly "name": (string);
readonly "parent"?: ((string) | (null));
readonly "rotation"?: ((number) | (null));
readonly "scale"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "skew"?: ((number) | (null));
});
readonly "BoneKeys": ({
readonly "bone": (string);
readonly "rotate"?: (ReadonlyArray<(InputDefinitions["Key"])> | (number extends ((InputDefinitions["Key"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "scale"?: (ReadonlyArray<(InputDefinitions["Key3"])> | (number extends ((InputDefinitions["Key3"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "skew"?: (ReadonlyArray<(InputDefinitions["Key2"])> | (number extends ((InputDefinitions["Key2"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "translate"?: (ReadonlyArray<(InputDefinitions["Key3"])> | (number extends ((InputDefinitions["Key3"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Clip": ({
readonly "bones"?: (ReadonlyArray<(InputDefinitions["BoneKeys"])> | (number extends ((InputDefinitions["BoneKeys"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "deforms"?: (Readonly<Record<string, (ReadonlyArray<(InputDefinitions["Key6"])> | (number extends ((InputDefinitions["Key6"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))>>);
readonly "duration": (number);
readonly "events"?: (ReadonlyArray<(InputDefinitions["Key9"])> | (number extends ((InputDefinitions["Key9"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "iks"?: (Readonly<Record<string, (ReadonlyArray<(InputDefinitions["Key8"])> | (number extends ((InputDefinitions["Key8"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))>>);
readonly "looping"?: ((boolean) | (null));
readonly "name": (string);
readonly "order"?: (ReadonlyArray<(InputDefinitions["Key7"])> | (number extends ((InputDefinitions["Key7"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "slots"?: (ReadonlyArray<(InputDefinitions["SlotKeys"])> | (number extends ((InputDefinitions["SlotKeys"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
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
readonly "Dragonbones": ({
/** Explicit atlas exchange text from input.text. */
readonly "atlas": (string);
readonly "conform"?: (((InputDefinitions["TilesetConform"])) | ((null)));
/** Explicit DragonBones 5.5 exchange text from input.text. */
readonly "dragonbones": (string);
readonly "filter"?: (((InputDefinitions["Filter"])) | ((null)));
readonly "texture": (InputDefinitions["ContentRef"]);
readonly "unit"?: ((number) | (null));
});
readonly "Ease": (((({
readonly "kind": ("linear") & (string);
})) & { readonly "points"?: never; }) | ((({
readonly "kind": ("step") & (string);
})) & { readonly "points"?: never; }) | (({
readonly "kind": ("bezier") & (string);
readonly "points": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})));
readonly "Filter": ("nearest" | "linear") & (string);
readonly "Ik": ({
readonly "bones": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "counterclockwise"?: ((boolean) | (null));
readonly "name": (string);
readonly "target": (string);
readonly "weight"?: ((number) | (null));
});
readonly "IkValue": ({
readonly "counterclockwise": (boolean);
readonly "weight": (number);
});
readonly "Influence": ({
/** Bone-local pixel position, y up. */
readonly "at": (ReadonlyArray<(number)> & { readonly length: 2 });
readonly "bone": (string);
readonly "weight": (number);
});
readonly "Key": ({
readonly "ease"?: (((InputDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (InputDefinitions["Rotation"]);
});
readonly "Key2": ({
readonly "ease"?: (((InputDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (number);
});
readonly "Key3": ({
readonly "ease"?: (((InputDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (ReadonlyArray<(number)> & { readonly length: 2 });
});
readonly "Key4": ({
readonly "ease"?: (((InputDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value"?: ((string) | (null));
});
readonly "Key5": ({
readonly "ease"?: (((InputDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (ReadonlyArray<(number)> & { readonly length: 4 });
});
readonly "Key6": ({
readonly "ease"?: (((InputDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Key7": ({
readonly "ease"?: (((InputDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Key8": ({
readonly "ease"?: (((InputDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (InputDefinitions["IkValue"]);
});
readonly "Key9": ({
readonly "ease"?: (((InputDefinitions["Ease"])) | ((null)));
readonly "time": (number);
readonly "value": (string);
});
readonly "Parts": ({
readonly "conform"?: (((InputDefinitions["TilesetConform"])) | ((null)));
readonly "filter"?: (((InputDefinitions["Filter"])) | ((null)));
/** Canonical module-relative PSD/PSB or PNG-parts directory under assets/
or sources/. Directory paths have no trailing slash; no sibling lookup. */
readonly "parts": (string);
readonly "pivots"?: (Readonly<Record<string, (ReadonlyArray<(number)> & { readonly length: 2 })>>);
readonly "template": (string);
readonly "unit"?: ((number) | (null));
});
readonly "PathConstraint": ({
readonly "bones": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "mix"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "name": (string);
readonly "path": (string);
readonly "position"?: ((number) | (null));
readonly "positionUnit"?: (((InputDefinitions["PathUnit"])) | ((null)));
readonly "rotate"?: (((InputDefinitions["PathRotate"])) | ((null)));
readonly "rotation"?: ((number) | (null));
readonly "spacing"?: ((number) | (null));
readonly "spacingMode"?: (((InputDefinitions["PathSpacing"])) | ((null)));
});
readonly "PathRotate": ("tangent" | "chain" | "chainScale") & (string);
readonly "PathSpacing": ("fixed" | "percent" | "length") & (string);
readonly "PathUnit": ("fixed" | "percent") & (string);
readonly "Rotation": ({
readonly "degrees": (number);
readonly "turn"?: (((InputDefinitions["Turn"])) | ((null)));
});
readonly "Slot": ({
readonly "attachment"?: ((string) | (null));
readonly "bone": (string);
readonly "color"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "name": (string);
readonly "offset"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
});
readonly "SlotKeys": ({
readonly "attachment"?: (ReadonlyArray<(InputDefinitions["Key4"])> | (number extends ((InputDefinitions["Key4"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "color"?: (ReadonlyArray<(InputDefinitions["Key5"])> | (number extends ((InputDefinitions["Key5"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "offset"?: (ReadonlyArray<(InputDefinitions["Key5"])> | (number extends ((InputDefinitions["Key5"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "slot": (string);
});
readonly "TilesetConform": (((({
readonly "kind": ("off") & (string);
})) & { readonly "size"?: never; }) | (({
readonly "kind": ("size") & (string);
readonly "size": (number);
})));
readonly "TransformConstraint": ({
readonly "at"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "bones": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "mix"?: ((ReadonlyArray<(number)> & { readonly length: 4 }) | (null));
readonly "name": (string);
readonly "rotation"?: ((number) | (null));
readonly "scale"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
readonly "shear"?: ((number) | (null));
readonly "target": (string);
});
readonly "Turn": (((({
readonly "kind": ("shorter") & (string);
})) & { readonly "turns"?: never; }) | (({
readonly "kind": ("clockwise") & (string);
readonly "turns": (number);
})) | (({
readonly "kind": ("counterclockwise") & (string);
readonly "turns": (number);
})));
readonly "Vertex": ({
readonly "influences": (ReadonlyArray<(InputDefinitions["Influence"])> | (number extends ((InputDefinitions["Influence"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "uv": (ReadonlyArray<(number)> & { readonly length: 2 });
});
}
```
