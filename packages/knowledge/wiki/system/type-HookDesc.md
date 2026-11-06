<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-HookDesc"></a>

### `HookDesc` (desc struct)

A hook's event and declared access.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `on` | `HookEvent` | `"add"` | — | — | — | Add or remove. |
| `reads` | `readonly (AnyReflected \| AnyEvent)[]` | `[]` | — | — | — | What it reads, as a system declares it. |
| `writes` | `readonly (AnyReflected \| AnyEvent)[]` | `[]` | — | — | — | What it writes, as a system declares it. |

```ogt
struct HookDesc {
  doc "A hook's event and declared access."
  desc true
  field on HookEvent "Add or remove."
  field reads List.OneOf.AnyReflected.AnyEvent "What it reads, as a system declares it."
  field writes List.OneOf.AnyReflected.AnyEvent "What it writes, as a system declares it."
}
```

<sub>`api.ogt` line 1362</sub>
