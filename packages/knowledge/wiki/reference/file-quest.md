<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-quest"></a>
### `file:quest`

Checked `file["quest"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "category"?: ((string) | (null));
readonly "giver"?: ((string) | (null));
/** Quest variables always belong to the shared story, not a conversation. */
readonly "globals"?: (Readonly<Record<string, (InputDefinitions["Value"])>>);
readonly "id": (string);
readonly "region"?: ((string) | (null));
readonly "rewards"?: (ReadonlyArray<(InputDefinitions["Reward"])> | (number extends ((InputDefinitions["Reward"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "stages": (ReadonlyArray<(InputDefinitions["Stage"])> | (number extends ((InputDefinitions["Stage"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "summary"?: ((string) | (null));
readonly "title": (string);
});
export interface InputDefinitions {
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Expr": (((({
readonly "kind": ("literal") & (string);
readonly "value": (InputDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("variable") & (string);
readonly "name": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "id": (string);
readonly "kind": ("once") & (string);
})) & { readonly "args"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("not") & (string);
readonly "value": (InputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("negate") & (string);
readonly "value": (InputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }) | ((({
readonly "kind": ("binary") & (string);
readonly "left": (InputDefinitions["Expr"]);
readonly "op": (InputDefinitions["Op"]);
readonly "right": (InputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "quest"?: never; readonly "value"?: never; }) | ((({
readonly "args": (ReadonlyArray<(InputDefinitions["Expr"])> | (number extends ((InputDefinitions["Expr"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("call") & (string);
readonly "name": (string);
})) & { readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("seen") & (string);
readonly "node": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("status") & (string);
readonly "quest": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("stage") & (string);
readonly "quest": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("count") & (string);
readonly "objective": (string);
readonly "quest": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "name"?: never; readonly "node"?: never; readonly "op"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("member") & (string);
readonly "member": (string);
readonly "name": (string);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("enum-value") & (string);
readonly "name": (string);
readonly "value": (InputDefinitions["Value"]);
})) & { readonly "args"?: never; readonly "id"?: never; readonly "left"?: never; readonly "member"?: never; readonly "node"?: never; readonly "objective"?: never; readonly "op"?: never; readonly "quest"?: never; readonly "right"?: never; }));
readonly "Objective": ({
readonly "filter"?: (((InputDefinitions["Expr"])) | ((null)));
readonly "hidden"?: ((boolean) | (null));
readonly "id": (string);
readonly "marker"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
readonly "on"?: ((string) | (null));
readonly "optional"?: ((boolean) | (null));
readonly "target"?: ((number) | (null));
readonly "text": (InputDefinitions["Text"]);
readonly "when"?: (((InputDefinitions["Expr"])) | ((null)));
});
readonly "Op": ("add" | "sub" | "mul" | "div" | "rem" | "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "and" | "or" | "xor") & (string);
readonly "Reward": ({
readonly "amount"?: ((number) | (null));
readonly "image"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "name": (string);
readonly "rarity"?: ((string) | (null));
});
readonly "Stage": ({
readonly "fail"?: (((InputDefinitions["Expr"])) | ((null)));
readonly "id": (string);
readonly "next"?: ((string) | (null));
readonly "objectives"?: (ReadonlyArray<(InputDefinitions["Objective"])> | (number extends ((InputDefinitions["Objective"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "summary"?: ((string) | (null));
});
readonly "Text": ({
readonly "args"?: (Readonly<Record<string, (InputDefinitions["Expr"])>>);
/** Localization key, not the rendered sentence. */
readonly "key": (string);
});
readonly "Value": (((number)) | ((boolean)) | ((string)));
}
```
