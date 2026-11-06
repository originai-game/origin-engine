<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-clear"></a>

### `og.debug.clear`

Remove every debug draw that is still showing.

```ts
og.debug.clear(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |

**Returns** `void`.

Binding symbol `og_debug_clear` · publish builds: dropped

```ogt
fn clear {
  doc "Remove every debug draw that is still showing."
  param world World "The world drawn in."
}
```

<sub>`api.ogt` line 25674</sub>
