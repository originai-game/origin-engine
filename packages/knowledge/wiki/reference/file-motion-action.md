<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-motion-action"></a>
### `file:motion-action`

Checked `file["motion-action"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
readonly "clip": (string);
readonly "hand"?: (((InputDefinitions["Hand"])) | ((null)));
/** In the head's local frame. */
readonly "handTarget"?: ((ReadonlyArray<(number)> & { readonly length: 3 }) | (null));
/** Must be greater than holdStart (domain validation). */
readonly "holdEnd": (number);
readonly "holdStart"?: (number);
readonly "rig"?: (((InputDefinitions["ContentRef"])) | ((null)));
readonly "source": (InputDefinitions["ContentRef"]);
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
readonly "Hand": ("left" | "right") & (string);
}
```
