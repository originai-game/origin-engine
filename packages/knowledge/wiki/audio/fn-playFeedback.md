<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="fn-playFeedback"></a>

### `og.audio.playFeedback`

Play a presentation-only one-shot on the UI bus in this tick (the next tick when called between ticks). Creates no world entity or event, changes no simulation hash, and is not saved or replayed on restore. Silently unheard when no audio sink is attached. Use AudioSource for gameplay audio.

```ts
og.audio.playFeedback(world: World, sound: Sound, desc?: AudioFeedbackDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose presentation audio sink plays it. |
| `sound` | `Sound` | no | A loaded sound. |
| `desc` | `AudioFeedbackDesc` | yes | Volume and playback rate. |

**Returns** `Result<void>`: Refused for an unloaded sound or invalid volume/playbackRate.

Binding symbol `og_audio_play_feedback` · publish builds: kept

```ogt
fn playFeedback {
  doc "Play a presentation-only one-shot on the UI bus in this tick (the next tick when called between ticks). Creates no world entity or event, changes no simulation hash, and is not saved or replayed on restore. Silently unheard when no audio sink is attached. Use AudioSource for gameplay audio."
  param world World "The world whose presentation audio sink plays it."
  param sound Sound "A loaded sound."
  param desc AudioFeedbackDesc "Volume and playback rate." {
    optional true
  }
  returns Result.Void "Refused for an unloaded sound or invalid volume/playbackRate."
}
```

<sub>Module `audio` line 736</sub>
