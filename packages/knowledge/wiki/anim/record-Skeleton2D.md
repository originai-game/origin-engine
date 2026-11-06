<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-Skeleton2D"></a>

### `og.Skeleton2D` (component)

Plays a clip of a 2D rig at the entity: the og.skeleton2d system advances it by scaled time (Time.scaledDt) after the game's systems and fires its events as og.Skeleton2DEvent; extraction draws its slots, in their order, as textured triangles of the sprite batch at `layer` (region attachments as quads, mesh attachments weighted to bones and deformed by the clip), lit by og.Lighting2D when the layer is in its range, as a sprite there is. The rig's pixels are `unit` to a world unit and +y is up; the entity's og.Transform places, turns (about z) and scales it.

Changing `clip` restarts at 0 (or at `time`, written in the same tick); writing a different time is a seek and fires no events. The pose is a function of these fields, the Transform and the og.IkTarget2D on it; the playback position is world state (the og.skeleton2d state block).

Token `og.Skeleton2D`, registered under the schema name `og.Skeleton2D`; its value type is `Skeleton2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rig` | `Rig2D` | `0` | — | — | — | The rig (og.load.rig2d's `rig`); 0 is none. |
| `clip` | `number` | `0` | ≥ 0 | — | — | The clip: an index into Rig2DInfo.clips (og.anim.clip finds one by name). |
| `time` | `number` | `0` | ≥ 0 | `s` | — | Seconds into the clip; written every tick, set it to seek. |
| `speed` | `number` | `1` | -10 … 10 | — | tunable | The playback rate: 1 as authored, 2 twice as fast, negative backwards. |
| `loop` | `boolean` | `true` | — | — | — | Wrap at the clip's end; otherwise hold the last pose and set `finished`. Rig2DInfo.clips says what the rig's author meant. |
| `flipX` | `boolean` | `false` | — | — | — | Mirror the rig left to right about the entity's position (a character facing left). |
| `layer` | `number` | `0` | — | — | — | The sprite layer it draws in, sorted with the sprites there (after them when they share the layer). |
| `skin` | `number` | `0` | ≥ 0 | — | — | The skin it wears: an index into Rig2DInfo.skins (0 is the rig's default; `rig.skins.indexOf("armor")` finds one by name). A skin shows its own attachment wherever the setup pose or a clip shows the default's; a slot the skin leaves alone shows the default's. Change it any tick: the pose is drawn from it. |
| `finished` | `boolean` | `false` | — | — | read only | Written: a clip that does not loop reached its end (or its start, played backwards). |

```ogt
component Skeleton2D {
  doc "Plays a clip of a 2D rig at the entity: the og.skeleton2d system advances it by scaled time (Time.scaledDt) after the game's systems and fires its events as og.Skeleton2DEvent; extraction draws its slots, in their order, as textured triangles of the sprite batch at `layer` (region attachments as quads, mesh attachments weighted to bones and deformed by the clip), lit by og.Lighting2D when the layer is in its range, as a sprite there is. The rig's pixels are `unit` to a world unit and +y is up; the entity's og.Transform places, turns (about z) and scales it."
  doc "Changing `clip` restarts at 0 (or at `time`, written in the same tick); writing a different time is a seek and fires no events. The pose is a function of these fields, the Transform and the og.IkTarget2D on it; the playback position is world state (the og.skeleton2d state block)."
  field rig Rig2D "The rig (og.load.rig2d's `rig`); 0 is none."
  field clip u32 "The clip: an index into Rig2DInfo.clips (og.anim.clip finds one by name)." {
    min 0
  }
  field time f32 "Seconds into the clip; written every tick, set it to seek." {
    min 0
    unit s
  }
  field speed f32 "The playback rate: 1 as authored, 2 twice as fast, negative backwards." {
    default 1
    min -10
    max 10
    tunable true
  }
  field loop bool "Wrap at the clip's end; otherwise hold the last pose and set `finished`. Rig2DInfo.clips says what the rig's author meant." {
    default true
  }
  field flipX bool "Mirror the rig left to right about the entity's position (a character facing left)."
  field layer i32 "The sprite layer it draws in, sorted with the sprites there (after them when they share the layer)."
  field skin u32 "The skin it wears: an index into Rig2DInfo.skins (0 is the rig's default; `rig.skins.indexOf(\"armor\")` finds one by name). A skin shows its own attachment wherever the setup pose or a clip shows the default's; a slot the skin leaves alone shows the default's. Change it any tick: the pose is drawn from it." {
    min 0
  }
  field finished bool "Written: a clip that does not loop reached its end (or its start, played backwards)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 12187</sub>
