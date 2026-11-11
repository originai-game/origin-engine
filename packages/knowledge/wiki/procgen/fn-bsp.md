<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-bsp"></a>

### `og.procgen.bsp`

A BSP dungeon: rooms that never overlap, every floor cell reachable from every other.

```ts
og.procgen.bsp(desc?: BspDesc): Result<BspResult>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `BspDesc` | yes | The area, the room sizes and the seed. |

**Returns** `Result<BspResult>`: Refused for an area that cannot hold one room of minRoom with its padding.

Binding symbol `og_procgen_bsp` · publish builds: kept

```ogt
fn bsp {
  doc "A BSP dungeon: rooms that never overlap, every floor cell reachable from every other."
  param desc BspDesc "The area, the room sizes and the seed."
  returns Result.BspResult "Refused for an area that cannot hold one room of minRoom with its padding."
}
```

<sub>Module `procgen` line 162</sub>
