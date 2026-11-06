<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-setEnabled"></a>

### `og.debug.setEnabled`

Show or hide every debug draw and view (`debug:on`, `debug:off`); on by default outside publish builds.

```ts
og.debug.setEnabled(world: World, enabled: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `enabled` | `boolean` | no | Whether they show. |

**Returns** `void`.

Binding symbol `og_debug_set_enabled` · publish builds: dropped

```ogt
fn setEnabled {
  doc "Show or hide every debug draw and view (`debug:on`, `debug:off`); on by default outside publish builds."
  param world World "The world drawn in."
  param enabled bool "Whether they show."
}
```

<sub>`api.ogt` line 25815</sub>
