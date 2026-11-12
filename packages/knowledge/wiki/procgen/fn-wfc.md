<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-wfc"></a>

### `og.procgen.wfc`

A grid of tiles whose edges all fit, by wave function collapse.

Also known as: wave function collapse, WFC, wavefunction collapse, tile constraint solver, socket tiles, procedural tilemap

```ts
og.procgen.wfc(desc?: WfcDesc): Result<WfcResult>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `WfcDesc` | yes | The grid, the tiles and the seed. |

**Returns** `Result<WfcResult>`: Refused for an empty tile list, a fixed cell outside the grid or naming no tile, or no tiling in `attempts` tries (naming the cell the last try ran out at).

Binding symbol `og_procgen_wfc` · publish builds: kept

```ogt
fn wfc {
  doc "A grid of tiles whose edges all fit, by wave function collapse."
  aliases "wave function collapse, WFC, wavefunction collapse, tile constraint solver, socket tiles, procedural tilemap"
  param desc WfcDesc "The grid, the tiles and the seed."
  returns Result.WfcResult "Refused for an empty tile list, a fixed cell outside the grid or naming no tile, or no tiling in `attempts` tries (naming the cell the last try ran out at)."
}
```

<sub>Module `procgen` line 220</sub>
