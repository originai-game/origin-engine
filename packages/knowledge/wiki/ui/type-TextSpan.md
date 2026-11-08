<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextSpan"></a>

### `TextSpan` (struct)

A named attribute range in plain text for a store and the Svelte text spans binding. Offsets count Unicode scalar values, not UTF-8 bytes, JavaScript UTF-16 code units or grapheme clusters. Unknown names and properties remain game data. A character span may cover the complete Speaker: prefix, including its separator. Names select text::part(name) styles; text is never reinterpreted as markup.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | The style part name, for example emphasis or character. |
| `position` | `number` | `0` | — | — | — | Start in Unicode scalar values; an astral character counts once. |
| `length` | `number` | `0` | — | — | — | Number of Unicode scalars. Zero is a metadata marker. |
| `properties` | `Readonly<Record<string, string \| number \| boolean>>` | `{}` | — | — | — | Typed attribute properties: only string, finite number or boolean, never nested data or null. |

```ogt
struct TextSpan {
  doc "A named attribute range in plain text for a store and the Svelte text spans binding. Offsets count Unicode scalar values, not UTF-8 bytes, JavaScript UTF-16 code units or grapheme clusters. Unknown names and properties remain game data. A character span may cover the complete Speaker: prefix, including its separator. Names select text::part(name) styles; text is never reinterpreted as markup."
  field name String "The style part name, for example emphasis or character."
  field position u32 "Start in Unicode scalar values; an astral character counts once."
  field length u32 "Number of Unicode scalars. Zero is a metadata marker."
  field properties Map.OneOf.String.OneOf.f64.bool "Typed attribute properties: only string, finite number or boolean, never nested data or null."
}
```

<sub>Module `ui` line 650</sub>
