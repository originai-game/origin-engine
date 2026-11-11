<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-soundInfo"></a>

### `og.load.soundInfo`

A loaded sound's format and length.

```ts
og.load.soundInfo(world: World, sound: Sound): Result<SoundInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the sound. |
| `sound` | `Sound` | no | The sound. |

**Returns** `Result<SoundInfo>`: Refused for a dead sound.

Binding symbol `og_load_sound_info` · publish builds: kept

```ogt
fn soundInfo {
  doc "A loaded sound's format and length."
  param world World "The world that owns the sound."
  param sound Sound "The sound."
  returns Result.SoundInfo "Refused for a dead sound."
}
```

<sub>Module `load` line 376</sub>
