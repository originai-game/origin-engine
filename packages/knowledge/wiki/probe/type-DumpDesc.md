<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DumpDesc"></a>

### `DumpDesc` (desc struct)

What to dump.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `what` | `DumpKind \| string \| null` | `null` | — | — | — | A DumpKind, `ui:layers`, `ui:motion`, or a registered subsystem dump's name (`physics`, `render`, …; an unknown name's error lists them), answered as a `subsystem` DumpResult. Required: a dump without one is refused. |
| `id` | `string` | `""` | — | — | — | The probe id, for an entity dump. |

```ogt
struct DumpDesc {
  doc "What to dump."
  desc true
  field what Option.OneOf.DumpKind.String "A DumpKind, `ui:layers`, `ui:motion`, or a registered subsystem dump's name (`physics`, `render`, …; an unknown name's error lists them), answered as a `subsystem` DumpResult. Required: a dump without one is refused."
  field id String "The probe id, for an entity dump."
}
```

<sub>Module `probe` line 429</sub>
