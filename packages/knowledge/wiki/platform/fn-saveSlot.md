<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-saveSlot"></a>

### `og.platform.saveSlot`

Save a long text under a slot, in as many store keys as it needs: a mid-size 3D world's snapshot is over a key's 64 KiB (93 entities with skinned characters, cloths and minds: about 78 KB). The parts go in the bank of keys the slot's last save did not use (`<slot>.a*` or `<slot>.b*`), then the manifest (`<slot>`: the bank, the part count, the length, a checksum and `meta`) is written last, at the revision read before: so a save cut short leaves the last one whole, and another device's save landing meanwhile is a conflict, never a mix. One request for all of it; a slot takes at most 17 of the player's 64 keys for this game (its two banks and the manifest: 1 MiB, the platform's budget for a player and game), and a world that would take more saves a selection of what it keeps (world.snapshot's SnapshotDesc).

```ts
og.platform.saveSlot(world: World, desc?: SaveSlotDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `SaveSlotDesc` | yes | The slot, the text and what a title shows of it. |

**Returns** `Result<number>`: Refused for a malformed slot name, a text over 512 KiB or a meta over 4 KiB. The answer arrives when the manifest is written: data `{ slot, parts, bytes, revision }`; an error is the first refused part's, or the manifest's (`conflict`: another save of the slot landed meanwhile; the slot holds that one).

Binding symbol `og_platform_save_slot` · publish builds: kept

```ogt
fn saveSlot {
  doc "Save a long text under a slot, in as many store keys as it needs: a mid-size 3D world's snapshot is over a key's 64 KiB (93 entities with skinned characters, cloths and minds: about 78 KB). The parts go in the bank of keys the slot's last save did not use (`<slot>.a*` or `<slot>.b*`), then the manifest (`<slot>`: the bank, the part count, the length, a checksum and `meta`) is written last, at the revision read before: so a save cut short leaves the last one whole, and another device's save landing meanwhile is a conflict, never a mix. One request for all of it; a slot takes at most 17 of the player's 64 keys for this game (its two banks and the manifest: 1 MiB, the platform's budget for a player and game), and a world that would take more saves a selection of what it keeps (world.snapshot's SnapshotDesc)."
  param world World "The world."
  param desc SaveSlotDesc "The slot, the text and what a title shows of it."
  returns Result.u32 "Refused for a malformed slot name, a text over 512 KiB or a meta over 4 KiB. The answer arrives when the manifest is written: data `{ slot, parts, bytes, revision }`; an error is the first refused part's, or the manifest's (`conflict`: another save of the slot landed meanwhile; the slot holds that one)."
}
```

<sub>Module `platform` line 200</sub>
