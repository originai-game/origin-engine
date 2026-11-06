<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-capabilities"></a>

### `og.destruction.capabilities`

What destruction does on this device.

```ts
og.destruction.capabilities(world: World): DestructionCapabilities
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `DestructionCapabilities`.

Binding symbol `og_destruction_capabilities` · publish builds: kept

```ogt
fn capabilities {
  doc "What destruction does on this device."
  param world World "The world."
  returns DestructionCapabilities
}
```

<sub>`api.ogt` line 27374</sub>
