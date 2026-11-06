<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-loadSlot"></a>

### `og.platform.loadSlot`

Read a slot saveSlot wrote: its manifest, then its parts, joined and checked against the manifest's length and checksum. One request for all of it.

```ts
og.platform.loadSlot(world: World, desc?: LoadSlotDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `LoadSlotDesc` | yes | The slot, and whether to read its meta alone. |

**Returns** `Result<number>`: Refused for a malformed slot name. The answer's data is `{ slot, text, meta, parts, bytes, revision }`: text and meta null for a slot never saved, text null with metaOnly; `invalid` for a key that holds no slot manifest or parts that do not match it.

Binding symbol `og_platform_load_slot` · publish builds: kept

```ogt
fn loadSlot {
  doc "Read a slot saveSlot wrote: its manifest, then its parts, joined and checked against the manifest's length and checksum. One request for all of it."
  param world World "The world."
  param desc LoadSlotDesc "The slot, and whether to read its meta alone."
  returns Result.u32 "Refused for a malformed slot name. The answer's data is `{ slot, text, meta, parts, bytes, revision }`: text and meta null for a slot never saved, text null with metaOnly; `invalid` for a key that holds no slot manifest or parts that do not match it."
}
```

<sub>`api.ogt` line 15106</sub>
