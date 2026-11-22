<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="record-AttackLink"></a>

### `og.AttackLink` (component)

A way from one move to another: a press of `action` buffered (og.input.consume) while the move it leaves is at a tick in [open, close) starts `to`. A press before the window waits in the action's buffer (ActionDesc.bufferTicks) and is taken the tick the window opens; one after it closes does not chain.

See `combat §1`.

Also known as: combo link, chain, cancel into, follow-up attack, combo route

Token `og.AttackLink`, registered under the schema name `og.AttackLink`; its value type is `AttackLink`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | `str(32)`: at most 32 bytes of UTF-8 | The input action whose press takes it (og.input.defineAction's name). |
| `to` | `Entity` | `0` | — | — | — | The og.AttackMove it starts. |
| `open` | `number` | `0` | ≥ 0 | — | — | The first tick (from the start of the move it leaves) it can be taken. |
| `close` | `number` | `0` | ≥ 0 | — | — | The tick it can no longer be taken; at or before open, never (a starter's window is not read). |

```ogt
component AttackLink {
  aliases "combo link, chain, cancel into, follow-up attack, combo route"
  doc "A way from one move to another: a press of `action` buffered (og.input.consume) while the move it leaves is at a tick in [open, close) starts `to`. A press before the window waits in the action's buffer (ActionDesc.bufferTicks) and is taken the tick the window opens; one after it closes does not chain."
  guide "combat §1"
  field action str.32 "The input action whose press takes it (og.input.defineAction's name)."
  field to Entity "The og.AttackMove it starts."
  field open u32 "The first tick (from the start of the move it leaves) it can be taken." {
    min 0
  }
  field close u32 "The tick it can no longer be taken; at or before open, never (a starter's window is not read)." {
    min 0
  }
}
```

<sub>Module `combat` line 75</sub>
