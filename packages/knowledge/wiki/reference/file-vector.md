<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-vector"></a>
### `file:vector`

Checked `file["vector"](input)` declaration from `@origingame/engine/author`.

```ts
/** Exactly one SVG source. SVG syntax/element support belongs to the vector
domain validator. No embedded OGT or untyped configuration is accepted. */
export type Input = (((({
/** Explicit canonical SVG path relative to the owning module
(e.g. assets/ui/emblem.svg), without sibling/assets fallback. */
readonly "source": (string);
/** Positive flattening error in SVG view-box units; omitted uses the
longer view-box side divided by 2000. */
readonly "tolerance"?: ((number) | (null));
})) & { readonly "svg"?: never; }) | ((({
readonly "svg": (string);
/** Positive flattening error in SVG view-box units. */
readonly "tolerance"?: ((number) | (null));
})) & { readonly "source"?: never; }));
```
