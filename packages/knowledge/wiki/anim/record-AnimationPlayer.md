<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-AnimationPlayer"></a>

### `og.AnimationPlayer` (component)

Plays one clip on a skeleton. Alone it poses the skeleton from its rest pose; with an og.AnimationGraph it plays over the graph's result with `weight`. Changing `clip` cross-fades from the previous one over `fade`; a clip change starts the new clip at 0 (or at `time` if the game wrote it in the same tick).

Also known as: play animation, Animator, AnimationPlayer, clip playback

Token `og.AnimationPlayer`, registered under the schema name `og.AnimationPlayer`; its value type is `AnimationPlayer`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `ClipAsset` | `0` | — | — | — | The clip asset; 0 plays nothing. |
| `time` | `number` | `0` | ≥ 0 | `s` | — | The playback position; written every tick, set it to seek (no markers fire across a seek). |
| `speed` | `number` | `1` | -10 … 10 | — | tunable | Playback rate (1 = as authored, negative plays backwards). |
| `loop` | `boolean` | `true` | — | — | — | Wrap at the end of the clip; otherwise hold the last frame and set `finished`. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the pose this player decides (0 = none, 1 = all). |
| `fade` | `number` | `0.2` | 0 … 10 | `s` | tunable | Cross-fade duration when `clip` changes. |
| `stepFps` | `number` | `0` | 0 … 60 | — | tunable | Stepped playback: each pose is held for 1/stepFps seconds, the clip sampled at its time floored to that step (12 is stop motion "on twos", 8 on threes), the same on every tick rate; 0 samples every tick. `time`, markers and root motion stay continuous; a one-shot still ends on its last frame. |
| `finished` | `boolean` | `false` | — | — | read only | Written: a non-looping clip reached its end. |

```ogt
component AnimationPlayer {
  doc "Plays one clip on a skeleton. Alone it poses the skeleton from its rest pose; with an og.AnimationGraph it plays over the graph's result with `weight`. Changing `clip` cross-fades from the previous one over `fade`; a clip change starts the new clip at 0 (or at `time` if the game wrote it in the same tick)."
  aliases "play animation, Animator, AnimationPlayer, clip playback"
  field clip ClipAsset "The clip asset; 0 plays nothing."
  field time f32 "The playback position; written every tick, set it to seek (no markers fire across a seek)." {
    min 0
    unit s
  }
  field speed f32 "Playback rate (1 = as authored, negative plays backwards)." {
    default 1
    min -10
    max 10
    tunable true
  }
  field loop bool "Wrap at the end of the clip; otherwise hold the last frame and set `finished`." {
    default true
  }
  field weight f32 "How much of the pose this player decides (0 = none, 1 = all)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field fade f32 "Cross-fade duration when `clip` changes." {
    default 0.2
    min 0
    max 10
    unit s
    tunable true
  }
  field stepFps f32 "Stepped playback: each pose is held for 1/stepFps seconds, the clip sampled at its time floored to that step (12 is stop motion \"on twos\", 8 on threes), the same on every tick rate; 0 samples every tick. `time`, markers and root motion stay continuous; a one-shot still ends on its last frame." {
    min 0
    max 60
    tunable true
  }
  field finished bool "Written: a non-looping clip reached its end." {
    readonly true
  }
}
```

<sub>Module `anim` line 127</sub>
