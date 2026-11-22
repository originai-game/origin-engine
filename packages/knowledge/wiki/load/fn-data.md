<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-data"></a>

### `og.load.data`

Read a file of a content catalog at run time: static, versioned content the game's host publishes (a world's index pages, a shared level), well past a store value's 64 KiB. The file is read from the host's content base against its version manifest's integrity; game code names no URL. The handle is pending, then ready (og.load.dataJson, og.load.dataBytes) or failed, og.load.error starting with its reason.

Completion is host input, journaled with its tick and integrity, so a replay reads the same bytes in at the same tick. og.load.release frees it.

See `loading §14`.

```ts
og.load.data(world: World, desc?: DataDesc): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `desc` | `DataDesc` | yes | The file and how to read it. |

**Returns** `Result<Asset>`: The handle (pending), or why it was refused now: a malformed catalog, version or path.

Binding symbol `og_load_data` · publish builds: kept

```ogt
fn data {
  doc "Read a file of a content catalog at run time: static, versioned content the game's host publishes (a world's index pages, a shared level), well past a store value's 64 KiB. The file is read from the host's content base against its version manifest's integrity; game code names no URL. The handle is pending, then ready (og.load.dataJson, og.load.dataBytes) or failed, og.load.error starting with its reason."
  guide "loading §14"
  doc "Completion is host input, journaled with its tick and integrity, so a replay reads the same bytes in at the same tick. og.load.release frees it."
  param world World "The world that owns the asset."
  param desc DataDesc "The file and how to read it."
  returns Result.Asset "The handle (pending), or why it was refused now: a malformed catalog, version or path."
}
```

<sub>Module `load` line 653</sub>
