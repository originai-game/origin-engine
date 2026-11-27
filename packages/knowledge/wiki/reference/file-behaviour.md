<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-behaviour"></a>
### `file:behaviour`

Checked `file["behaviour"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "keys"?: (Readonly<Record<string, (InputDefinitions["Value"])>>);
readonly "name": (string);
readonly "root": (InputDefinitions["Root"]);
readonly "routes"?: (Readonly<Record<string, (ReadonlyArray<(ReadonlyArray<(number)> & { readonly length: 3 })> | (number extends ((ReadonlyArray<(number)> & { readonly length: 3 })) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never))>>);
});
export interface InputDefinitions {
readonly "Action": (((({
readonly "key"?: ((string) | (null));
readonly "kind": ("move-to") & (string);
readonly "point"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "speed"?: (((InputDefinitions["Speed"])) | ((null)));
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "key": (string);
readonly "kind": ("chase") & (string);
readonly "reach": (number);
readonly "speed"?: (((InputDefinitions["Speed"])) | ((null)));
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "distance": (number);
readonly "key": (string);
readonly "kind": ("flee") & (string);
readonly "speed"?: (((InputDefinitions["Speed"])) | ((null)));
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "kind": ("patrol") & (string);
readonly "loops": (boolean);
readonly "route": (string);
readonly "speed"?: (((InputDefinitions["Speed"])) | ((null)));
readonly "wait": (number);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "seconds"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("wander") & (string);
readonly "radius": (number);
readonly "speed"?: (((InputDefinitions["Speed"])) | ((null)));
readonly "wait": (number);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("wait") & (string);
readonly "seconds": (number);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "speed"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "kind": ("stop") & (string);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "speed"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "key": (string);
readonly "kind": ("set") & (string);
readonly "value": (InputDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "cooldown"?: never; readonly "distance"?: never; readonly "loops"?: never; readonly "name"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "speed"?: never; readonly "wait"?: never; }) | ((({
readonly "args": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "cooldown": (number);
readonly "kind": ("command") & (string);
readonly "name": (string);
})) & { readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "speed"?: never; readonly "value"?: never; readonly "wait"?: never; }) | ((({
readonly "args": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("game") & (string);
readonly "name": (string);
})) & { readonly "cooldown"?: never; readonly "distance"?: never; readonly "key"?: never; readonly "loops"?: never; readonly "point"?: never; readonly "radius"?: never; readonly "reach"?: never; readonly "route"?: never; readonly "seconds"?: never; readonly "speed"?: never; readonly "value"?: never; readonly "wait"?: never; }));
readonly "Check": ({
/** Dynamic comparison operand, exclusive with value. */
readonly "against"?: ((string) | (null));
readonly "op": (InputDefinitions["Op"]);
readonly "operand": (string);
readonly "value"?: (((InputDefinitions["Value"])) | ((null)));
});
readonly "Consideration": (((({
readonly "kind": ("constant") & (string);
readonly "value": (number);
})) & { readonly "curve"?: never; readonly "hi"?: never; readonly "lo"?: never; readonly "operand"?: never; }) | ((({
readonly "curve": (InputDefinitions["Curve"]);
readonly "hi": (number);
readonly "kind": ("read") & (string);
readonly "lo": (number);
readonly "operand": (string);
})) & { readonly "value"?: never; }));
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Curve": ("linear" | "inverse" | "quadratic" | "sqrt" | "smooth" | "step") & (string);
readonly "Node": (((({
readonly "children": (ReadonlyArray<(InputDefinitions["Node"])> | (number extends ((InputDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("selector") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "children": (ReadonlyArray<(InputDefinitions["Node"])> | (number extends ((InputDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("sequence") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "children": (ReadonlyArray<(InputDefinitions["Node"])> | (number extends ((InputDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("steps") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "all": (boolean);
readonly "children": (ReadonlyArray<(InputDefinitions["Node"])> | (number extends ((InputDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("parallel") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "check"?: never; readonly "child"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "child": (InputDefinitions["Node"]);
readonly "kind": ("invert") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "child": (InputDefinitions["Node"]);
readonly "kind": ("succeed") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "child": (InputDefinitions["Node"]);
readonly "kind": ("repeat") & (string);
readonly "name"?: ((string) | (null));
readonly "times": (number);
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; }) | ((({
readonly "child": (InputDefinitions["Node"]);
readonly "kind": ("cooldown") & (string);
readonly "name"?: ((string) | (null));
readonly "seconds": (number);
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "child": (InputDefinitions["Node"]);
readonly "kind": ("timeout") & (string);
readonly "name"?: ((string) | (null));
readonly "seconds": (number);
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "check": (InputDefinitions["Check"]);
readonly "kind": ("check") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "child"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "action": (InputDefinitions["Action"]);
readonly "kind": ("action") & (string);
readonly "name"?: ((string) | (null));
})) & { readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }) | ((({
readonly "kind": ("subtree") & (string);
readonly "name"?: ((string) | (null));
readonly "source": (InputDefinitions["ContentRef"]);
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "inertia"?: never; readonly "options"?: never; readonly "seconds"?: never; readonly "times"?: never; }) | ((({
readonly "inertia": (number);
readonly "kind": ("utility") & (string);
readonly "name"?: ((string) | (null));
readonly "options": (ReadonlyArray<(InputDefinitions["UtilityOption"])> | (number extends ((InputDefinitions["UtilityOption"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "action"?: never; readonly "all"?: never; readonly "check"?: never; readonly "child"?: never; readonly "children"?: never; readonly "seconds"?: never; readonly "source"?: never; readonly "times"?: never; }));
readonly "Op": ("is" | "not" | "below" | "above" | "atMost" | "atLeast" | "truthy") & (string);
readonly "Root": (((({
readonly "kind": ("tree") & (string);
readonly "node": (InputDefinitions["Node"]);
})) & { readonly "initial"?: never; readonly "states"?: never; }) | ((({
readonly "initial": (string);
readonly "kind": ("machine") & (string);
readonly "states": (ReadonlyArray<(InputDefinitions["State"])> | (number extends ((InputDefinitions["State"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "node"?: never; }));
readonly "Speed": (((number)) | ((string)));
readonly "State": ({
readonly "body"?: (((InputDefinitions["Node"])) | ((null)));
readonly "enter"?: (ReadonlyArray<(InputDefinitions["Action"])> | (number extends ((InputDefinitions["Action"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "exit"?: (ReadonlyArray<(InputDefinitions["Action"])> | (number extends ((InputDefinitions["Action"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
readonly "transitions"?: (ReadonlyArray<(InputDefinitions["Transition"])> | (number extends ((InputDefinitions["Transition"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Transition": ({
readonly "after"?: ((number) | (null));
readonly "checks"?: (ReadonlyArray<(InputDefinitions["Check"])> | (number extends ((InputDefinitions["Check"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "to": (string);
});
readonly "UtilityOption": ({
readonly "children": (ReadonlyArray<(InputDefinitions["Node"])> | (number extends ((InputDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "name": (string);
readonly "scores": (ReadonlyArray<(InputDefinitions["Consideration"])> | (number extends ((InputDefinitions["Consideration"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Value": (((number)) | ((boolean)) | ((ReadonlyArray<(number)> & { readonly length: 3 })));
}
```
