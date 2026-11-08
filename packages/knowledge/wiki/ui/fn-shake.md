<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-shake"></a>

### `og.ui.shake`

Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snapshots carry it. Still under reduced motion. A refused press on a disabled button already shakes when the style's motion.shake is set.

```ts
og.ui.shake(desc?: ShakeDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `ShakeDesc` | yes | Amount, duration and sound. |

**Returns** `void`.

Binding symbol `og_ui_shake` · publish builds: kept

```ogt
fn shake {
  doc "Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snapshots carry it. Still under reduced motion. A refused press on a disabled button already shakes when the style's motion.shake is set."
  param desc ShakeDesc "Amount, duration and sound."
  returns Void
}
```

<sub>Module `ui` line 1879</sub>
