<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DumpKind"></a>

### `DumpKind` (enum)

What dump() reports.

A string: one of the variants below. The zero value is `"world"`.

| Variant | Doc |
|---|---|
| `"world"` | Archetype tables, systems and their matched counts. |
| `"entity"` | One entity's components and transform. |
| `"ui"` | The UI tree with bounds. |
| `"desync"` | The first desync of a lockstep session (og.net): its tick, both hashes and the component-level diff of the two peers' snapshots. |
| `"drawn"` | What the main camera's cull keeps of the frame: every entity with something to draw, whether it is drawn and why. |

```ogt
enum DumpKind {
  doc "What dump() reports."
  variant world "Archetype tables, systems and their matched counts."
  variant entity "One entity's components and transform."
  variant ui "The UI tree with bounds."
  variant desync "The first desync of a lockstep session (og.net): its tick, both hashes and the component-level diff of the two peers' snapshots."
  variant drawn "What the main camera's cull keeps of the frame: every entity with something to draw, whether it is drawn and why."
}
```

<sub>Module `probe` line 409</sub>
