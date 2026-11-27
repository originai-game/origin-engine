<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-dialogue"></a>
### `file:dialogue`

Checked `file["dialogue"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ((((InputDefinitions["TypedDialogue"])) & { readonly "assets"?: never; readonly "yarn"?: never; }) | (((InputDefinitions["YarnImport"])) & { readonly "barks"?: never; readonly "locale"?: never; readonly "name"?: never; readonly "nodes"?: never; readonly "smart"?: never; readonly "speakers"?: never; readonly "start"?: never; readonly "strings"?: never; readonly "vars"?: never; }));
export interface InputDefinitions {
readonly "Bark": ({
readonly "cooldown"?: ((number) | (null));
readonly "duration"?: ((number) | (null));
readonly "id": (string);
readonly "lines": (ReadonlyArray<(InputDefinitions["Line"])> | (number extends ((InputDefinitions["Line"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "once"?: ((boolean) | (null));
readonly "priority"?: ((number) | (null));
readonly "random"?: ((boolean) | (null));
readonly "shared"?: ((number) | (null));
readonly "speaker"?: ((string) | (null));
readonly "steps"?: (ReadonlyArray<(InputDefinitions["Step"])> | (number extends ((InputDefinitions["Step"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "topic": (string);
readonly "weight"?: ((number) | (null));
readonly "when"?: (((InputDefinitions["Expr"])) | ((null)));
});
readonly "Candidate": ({
readonly "condition"?: (((InputDefinitions["Expr"])) | ((null)));
readonly "node": (string);
readonly "weight": (number);
});
readonly "Check": ({
readonly "active": (boolean);
readonly "stat": (string);
readonly "target": (InputDefinitions["Expr"]);
});
readonly "Choice": ({
readonly "check"?: (((InputDefinitions["Check"])) | ((null)));
readonly "condition"?: (((InputDefinitions["Expr"])) | ((null)));
readonly "fail"?: ((string) | (null));
readonly "goto"?: ((string) | (null));
readonly "hide"?: (boolean);
readonly "id": (string);
readonly "kind"?: ((string) | (null));
readonly "reason"?: (((InputDefinitions["Text"])) | ((null)));
readonly "steps"?: (ReadonlyArray<(InputDefinitions["Step"])> | (number extends ((InputDefinitions["Step"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "tags"?: (Readonly<Record<string, (string)>>);
readonly "text": (InputDefinitions["Text"]);
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
readonly "Flow": ({
readonly "chapter"?: ((string) | (null));
readonly "dissolve"?: ((number) | (null));
readonly "ending"?: ((string) | (null));
readonly "music"?: (((InputDefinitions["Music"])) | ((null)));
readonly "title"?: ((string) | (null));
});
readonly "Line": ({
readonly "expression"?: ((string) | (null));
/** Stable author identity, independent of source position. */
readonly "id": (string);
readonly "shot"?: ((string) | (null));
readonly "speaker"?: ((string) | (null));
readonly "text": (InputDefinitions["Text"]);
readonly "voice"?: (((InputDefinitions["ContentRef"])) | ((null)));
});
readonly "Music": ({
readonly "duck"?: ((number) | (null));
readonly "fade"?: ((number) | (null));
/** Omit to stop the current music. */
readonly "sound"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "volume"?: ((number) | (null));
});
readonly "Node": ({
readonly "choiceFallback"?: ((string) | (null));
readonly "choices"?: (ReadonlyArray<(InputDefinitions["Choice"])> | (number extends ((InputDefinitions["Choice"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "flow"?: (((InputDefinitions["Flow"])) | ((null)));
readonly "id": (string);
readonly "segment"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "shot"?: ((string) | (null));
readonly "steps"?: (ReadonlyArray<(InputDefinitions["Step"])> | (number extends ((InputDefinitions["Step"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Tick count and fallback choice identity, not an array index. */
readonly "timer"?: ((readonly [(number), (string)]) | (null));
/** Owning source node; omit unless this is an explicit continuation block. */
readonly "title"?: ((string) | (null));
});
readonly "Op": ("add" | "sub" | "mul" | "div" | "rem" | "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "and" | "or" | "xor") & (string);
readonly "QuestOp": (((({
readonly "kind": ("start") & (string);
readonly "quest": (string);
})) & { readonly "amount"?: never; readonly "objective"?: never; readonly "stage"?: never; }) | ((({
readonly "amount": (InputDefinitions["Expr"]);
readonly "kind": ("progress") & (string);
readonly "objective": (string);
readonly "quest": (string);
})) & { readonly "stage"?: never; }) | ((({
readonly "kind": ("stage") & (string);
readonly "quest": (string);
readonly "stage": (string);
})) & { readonly "amount"?: never; readonly "objective"?: never; }) | ((({
readonly "kind": ("complete") & (string);
readonly "quest": (string);
})) & { readonly "amount"?: never; readonly "objective"?: never; readonly "stage"?: never; }) | ((({
readonly "kind": ("fail") & (string);
readonly "quest": (string);
})) & { readonly "amount"?: never; readonly "objective"?: never; readonly "stage"?: never; }));
readonly "Side": ("left" | "right" | "center") & (string);
readonly "Speaker": ({
readonly "expression"?: ((string) | (null));
readonly "id": (string);
readonly "name"?: ((string) | (null));
readonly "portrait"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "side"?: (((InputDefinitions["Side"])) | ((null)));
});
readonly "Step": (((({
readonly "expression"?: ((string) | (null));
/** Stable author identity, independent of source position. */
readonly "id": (string);
readonly "kind": ("line") & (string);
readonly "shot"?: ((string) | (null));
readonly "speaker"?: ((string) | (null));
readonly "text": (InputDefinitions["Text"]);
readonly "voice"?: (((InputDefinitions["ContentRef"])) | ((null)));
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "value"?: never; }) | ((({
readonly "kind": ("set") & (string);
readonly "name": (string);
readonly "value": (InputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "voice"?: never; }) | ((({
readonly "args": (ReadonlyArray<(InputDefinitions["Expr"])> | (number extends ((InputDefinitions["Expr"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("call") & (string);
readonly "name": (string);
})) & { readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "args": (ReadonlyArray<(InputDefinitions["Expr"])> | (number extends ((InputDefinitions["Expr"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("command") & (string);
readonly "name": (string);
})) & { readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "id": (string);
readonly "kind": ("mark-once") & (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "condition": (InputDefinitions["Expr"]);
readonly "kind": ("if") & (string);
readonly "node": (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("goto") & (string);
readonly "node": (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("jump") & (string);
readonly "node": (InputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("end") & (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("emit") & (string);
readonly "name": (string);
readonly "value"?: (((InputDefinitions["Expr"])) | ((null)));
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("quest") & (string);
readonly "operation": (InputDefinitions["QuestOp"]);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "node"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "continuation": (string);
readonly "kind": ("detour") & (string);
readonly "node": (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "continuation": (string);
readonly "kind": ("dynamic-detour") & (string);
readonly "node": (InputDefinitions["Expr"]);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "candidates": (ReadonlyArray<(InputDefinitions["Candidate"])> | (number extends ((InputDefinitions["Candidate"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "kind": ("select") & (string);
})) & { readonly "args"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }) | ((({
readonly "kind": ("return") & (string);
})) & { readonly "args"?: never; readonly "candidates"?: never; readonly "condition"?: never; readonly "continuation"?: never; readonly "expression"?: never; readonly "id"?: never; readonly "name"?: never; readonly "node"?: never; readonly "operation"?: never; readonly "shot"?: never; readonly "speaker"?: never; readonly "text"?: never; readonly "value"?: never; readonly "voice"?: never; }));
readonly "Text": ({
readonly "args"?: (Readonly<Record<string, (InputDefinitions["Expr"])>>);
/** Localization key, not the rendered sentence. */
readonly "key": (string);
});
readonly "TypedDialogue": ({
readonly "barks"?: (ReadonlyArray<(InputDefinitions["Bark"])> | (number extends ((InputDefinitions["Bark"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "locale"?: ((string) | (null));
readonly "name": (string);
readonly "nodes": (ReadonlyArray<(InputDefinitions["Node"])> | (number extends ((InputDefinitions["Node"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Lazily evaluated variables expressed in the same typed expression tree. */
readonly "smart"?: (Readonly<Record<string, (InputDefinitions["Expr"])>>);
readonly "speakers"?: (ReadonlyArray<(InputDefinitions["Speaker"])> | (number extends ((InputDefinitions["Speaker"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "start": (string);
/** Stable line key to ICU base text. ICU remains a string language. */
readonly "strings"?: (Readonly<Record<string, (string)>>);
readonly "vars"?: (ReadonlyArray<(InputDefinitions["Variable"])> | (number extends ((InputDefinitions["Variable"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Value": (((number)) | ((boolean)) | ((string)));
readonly "Variable": ({
readonly "global"?: (boolean);
readonly "name": (string);
readonly "value": (InputDefinitions["Value"]);
});
readonly "YarnImport": ({
/** Yarn resource names mapped explicitly to checked content references. */
readonly "assets"?: (Readonly<Record<string, (InputDefinitions["ContentRef"])>>);
/** Explicit Yarn source text from input.text; never automatically scanned. */
readonly "yarn": (string);
});
}
```
