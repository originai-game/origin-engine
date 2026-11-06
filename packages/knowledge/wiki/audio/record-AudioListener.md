<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-AudioListener"></a>

### `og.AudioListener` (component)

Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used.

Token `og.AudioListener`, registered under the schema name `og.AudioListener`; its value type is `AudioListener`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `volume` | `number` | `1` | 0 … 4 | — | tunable | A gain on every spatial source this listener hears. |
| `dopplerFactor` | `number` | `1` | 0 … 10 | — | tunable | Scales every spatial source's Doppler shift (AudioSource.dopplerFactor); 0 turns Doppler off for this listener. |
| `panningModel` | `PanningModel` | `"equalpower"` | — | — | — | How spatial sources are placed between the ears: equalpower (speakers) or hrtf (headphones). Non-spatial sources keep their pan either way; the stereo path is unchanged while it is equalpower. |

```ogt
component AudioListener {
  doc "Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used."
  field volume f32 "A gain on every spatial source this listener hears." {
    default 1
    min 0
    max 4
    tunable true
  }
  field dopplerFactor f32 "Scales every spatial source's Doppler shift (AudioSource.dopplerFactor); 0 turns Doppler off for this listener." {
    default 1
    min 0
    max 10
    tunable true
  }
  field panningModel PanningModel "How spatial sources are placed between the ears: equalpower (speakers) or hrtf (headphones). Non-spatial sources keep their pan either way; the stereo path is unchanged while it is equalpower." {
    default equalpower
  }
}
```

<sub>`api.ogt` line 12982</sub>
