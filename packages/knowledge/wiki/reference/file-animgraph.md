<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-animgraph"></a>
### `file:animgraph`

Checked `file["animgraph"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "bindings"?: (ReadonlyArray<(InputDefinitions["Binding"])> | (number extends ((InputDefinitions["Binding"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "layers": (ReadonlyArray<(InputDefinitions["LayerInput"])> | (number extends ((InputDefinitions["LayerInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "motions": (Readonly<Record<string, (InputDefinitions["MotionInput"])>>);
readonly "params"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "sprite"?: (boolean);
});
export interface InputDefinitions {
readonly "Binding": ({
readonly "clip": (string);
readonly "name": (string);
readonly "source": (InputDefinitions["ContentRef"]);
});
readonly "ConditionInput": ({
readonly "op": (InputDefinitions["Op"]);
readonly "param": (string);
readonly "value": (number);
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
readonly "Interrupt": ("any" | "none" | "source" | "next") & (string);
readonly "LayerInput": ({
readonly "entry": (string);
readonly "mask"?: (Readonly<Record<string, (number)>>);
readonly "mode"?: (((InputDefinitions["LayerMode"])) | ((null)));
readonly "name": (string);
readonly "states": (ReadonlyArray<(InputDefinitions["StateInput"])> | (number extends ((InputDefinitions["StateInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "transitions"?: (ReadonlyArray<(InputDefinitions["TransitionInput"])> | (number extends ((InputDefinitions["TransitionInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "weight"?: ((number) | (null));
readonly "weightParam"?: ((string) | (null));
});
readonly "LayerMode": ("override" | "additive") & (string);
readonly "MotionInput": (((({
readonly "clip": (string);
readonly "kind": ("clip") & (string);
readonly "looping"?: ((boolean) | (null));
readonly "mirror"?: ((boolean) | (null));
readonly "speed"?: ((number) | (null));
})) & { readonly "children"?: never; readonly "param"?: never; readonly "x"?: never; readonly "y"?: never; }) | ((({
readonly "children": (ReadonlyArray<(InputDefinitions["ThresholdInput"])> | (number extends ((InputDefinitions["ThresholdInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("blend1d") & (string);
readonly "param": (string);
})) & { readonly "clip"?: never; readonly "looping"?: never; readonly "mirror"?: never; readonly "speed"?: never; readonly "x"?: never; readonly "y"?: never; }) | ((({
readonly "children": (ReadonlyArray<(InputDefinitions["PointInput"])> | (number extends ((InputDefinitions["PointInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("blend2d") & (string);
readonly "x": (string);
readonly "y": (string);
})) & { readonly "clip"?: never; readonly "looping"?: never; readonly "mirror"?: never; readonly "param"?: never; readonly "speed"?: never; }));
readonly "Op": ("gt" | "ge" | "lt" | "le" | "eq" | "ne") & (string);
readonly "PointInput": ({
readonly "motion": (string);
readonly "point": (ReadonlyArray<(number)> & { readonly length: 2 });
});
readonly "StateInput": ({
readonly "motion": (string);
readonly "name": (string);
readonly "speed"?: ((number) | (null));
readonly "sync"?: ((string) | (null));
});
readonly "ThresholdInput": ({
readonly "motion": (string);
readonly "threshold": (number);
});
readonly "TransitionInput": ({
readonly "conditions"?: (ReadonlyArray<(InputDefinitions["ConditionInput"])> | (number extends ((InputDefinitions["ConditionInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "exit"?: ((number) | (null));
readonly "fade"?: ((number) | (null));
readonly "from"?: ((string) | (null));
readonly "interrupt"?: (((InputDefinitions["Interrupt"])) | ((null)));
readonly "to": (string);
});
}
```
