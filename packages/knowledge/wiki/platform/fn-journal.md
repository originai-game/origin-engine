<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-journal"></a>

### `og.platform.journal`

Everything the host handed this engine, oldest first, with the tick it was handed at: platform answers are host input, and `og replay record --platform-journal` hands these back at the same ticks, so a web session, whose answers come when the page delivers them, replays natively. A publish build keeps none.

See `saves §7`.

```ts
og.platform.journal(world: World): readonly PlatformHanded[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly PlatformHanded[]`.

Binding symbol `og_platform_journal` · publish builds: kept

```ogt
fn journal {
  doc "Everything the host handed this engine, oldest first, with the tick it was handed at: platform answers are host input, and `og replay record --platform-journal` hands these back at the same ticks, so a web session, whose answers come when the page delivers them, replays natively. A publish build keeps none."
  guide "saves §7"
  param world World "The world."
  returns List.PlatformHanded
}
```

<sub>Module `platform` line 475</sub>
