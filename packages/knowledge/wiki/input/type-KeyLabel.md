<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-KeyLabel"></a>

### `KeyLabel` (desc struct)

A key and what the player's keyboard layout prints on it: `KeyQ` and `a` on AZERTY.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `code` | `string` | `""` | — | — | — | The physical key (KeyboardEvent.code). |
| `label` | `string` | `""` | — | — | — | The character it types without modifiers, as the layout map gives it (`a`, `,`, `ö`). |

```ogt
struct KeyLabel {
  doc "A key and what the player's keyboard layout prints on it: `KeyQ` and `a` on AZERTY."
  desc true
  field code String "The physical key (KeyboardEvent.code)."
  field label String "The character it types without modifiers, as the layout map gives it (`a`, `,`, `ö`)."
}
```

<sub>`api.ogt` line 2223</sub>
