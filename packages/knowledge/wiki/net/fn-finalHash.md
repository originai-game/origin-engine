<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="fn-finalHash"></a>

### `og.net.finalHash`

The shared hash (og.net.hash) of the final world of `tick`, hex: the world every peer agrees on once each input for the ticks before it arrived, kept for the last 1024 final ticks while hashing (hashEvery above 0). In lockstep every tick run is final; in rollback a tick is final some ticks after it first ran. Null before the tick is final, after it is no longer kept, or outside lockstep.

```ts
og.net.finalHash(world: World, tick: number): string | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `tick` | `number` | no | The tick, in ticks: the world after tick - 1 ran, as og.Time.tick reads. |

**Returns** `string | null`.

Binding symbol `og_net_final_hash` · publish builds: kept

```ogt
fn finalHash {
  doc "The shared hash (og.net.hash) of the final world of `tick`, hex: the world every peer agrees on once each input for the ticks before it arrived, kept for the last 1024 final ticks while hashing (hashEvery above 0). In lockstep every tick run is final; in rollback a tick is final some ticks after it first ran. Null before the tick is final, after it is no longer kept, or outside lockstep."
  param world World "The world."
  param tick f64 "The tick, in ticks: the world after tick - 1 ran, as og.Time.tick reads."
  returns Option.String
}
```

<sub>`api.ogt` line 26488</sub>
