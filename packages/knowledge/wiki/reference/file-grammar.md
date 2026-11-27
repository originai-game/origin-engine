<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-grammar"></a>
### `file:grammar`

Checked `file["grammar"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "params"?: (Readonly<Record<string, (number)>>);
readonly "rules": (Readonly<Record<string, (ReadonlyArray<(InputDefinitions["Operation"])> | (number extends ((InputDefinitions["Operation"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))>>);
readonly "start": (string);
});
export interface InputDefinitions {
readonly "Axis": ("x" | "y") & (string);
readonly "CaseArm": ({
readonly "condition": (InputDefinitions["Condition"]);
readonly "rule": (string);
});
readonly "Condition": (((({
readonly "kind": ("wider") & (string);
readonly "value": (InputDefinitions["Number"]);
})) & { readonly "name"?: never; }) | ((({
readonly "kind": ("taller") & (string);
readonly "value": (InputDefinitions["Number"]);
})) & { readonly "name"?: never; }) | (({
readonly "kind": ("param") & (string);
readonly "name": (string);
readonly "value": (InputDefinitions["Number"]);
})));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Faces": ({
readonly "back"?: ((string) | (null));
readonly "bottom"?: ((string) | (null));
readonly "front"?: ((string) | (null));
readonly "side"?: ((string) | (null));
readonly "sides"?: ((string) | (null));
readonly "top"?: ((string) | (null));
});
/** A literal or a named grammar parameter. This is not an expression evaluator. */
readonly "Number": (((number)) | ((string)));
readonly "Operation": (((({
readonly "depth": (InputDefinitions["Number"]);
readonly "jamb"?: ((string) | (null));
readonly "op": ("recess") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "level": (number);
readonly "op": ("detail") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "label": (string);
readonly "op": ("label") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "depth": (InputDefinitions["Number"]);
readonly "faces": (InputDefinitions["Faces"]);
readonly "op": ("extrude") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "eave"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "axis": (InputDefinitions["Axis"]);
readonly "op": ("split") & (string);
readonly "parts": (ReadonlyArray<(InputDefinitions["SplitPart"])> | (number extends ((InputDefinitions["SplitPart"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "arms"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "border": (string);
readonly "inner": (string);
readonly "op": ("inset") & (string);
readonly "width": (InputDefinitions["Number"]);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; }) | ((({
readonly "eave"?: ((string) | (null));
readonly "gable"?: ((string) | (null));
readonly "op": ("roof") & (string);
readonly "overhang"?: (((InputDefinitions["Number"])) | ((null)));
readonly "roof": (InputDefinitions["Roof"]);
readonly "slope": (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "faces"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "parts"?: never; readonly "width"?: never; }) | ((({
readonly "model": (InputDefinitions["ContentRef"]);
readonly "op": ("insert") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "op": ("pick") & (string);
readonly "options": (ReadonlyArray<(InputDefinitions["WeightedRule"])> | (number extends ((InputDefinitions["WeightedRule"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "arms": (ReadonlyArray<(InputDefinitions["CaseArm"])> | (number extends ((InputDefinitions["CaseArm"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "op": ("case") & (string);
readonly "otherwise"?: ((string) | (null));
})) & { readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }) | ((({
readonly "op": ("drop") & (string);
})) & { readonly "arms"?: never; readonly "axis"?: never; readonly "border"?: never; readonly "depth"?: never; readonly "eave"?: never; readonly "faces"?: never; readonly "gable"?: never; readonly "inner"?: never; readonly "jamb"?: never; readonly "label"?: never; readonly "level"?: never; readonly "model"?: never; readonly "options"?: never; readonly "otherwise"?: never; readonly "overhang"?: never; readonly "parts"?: never; readonly "roof"?: never; readonly "slope"?: never; readonly "width"?: never; }));
readonly "Roof": (((({
readonly "kind": ("flat") & (string);
})) & { readonly "angle"?: never; }) | (({
readonly "angle": (InputDefinitions["Number"]);
readonly "kind": ("gable") & (string);
})) | (({
readonly "angle": (InputDefinitions["Number"]);
readonly "kind": ("hip") & (string);
})));
readonly "Size": ("abs" | "fill" | "repeat") & (string);
readonly "SplitPart": ({
readonly "length": (InputDefinitions["Number"]);
readonly "rule": (string);
readonly "size": (InputDefinitions["Size"]);
});
readonly "WeightedRule": ({
readonly "rule": (string);
readonly "weight": (number);
});
}
```
