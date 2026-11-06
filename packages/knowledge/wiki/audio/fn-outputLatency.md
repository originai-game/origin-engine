<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="fn-outputLatency"></a>

### `og.audio.outputLatency`

How long after its world time a sample leaves the speaker, as the host reports it: on the web the AudioContext's baseLatency and outputLatency and the worklet ring's buffer, natively the device's latency (cpal's playback less callback instant) and the mixer's buffer; 0 headless (the null and capture sinks are driven to the tick). Host input, written to og.Audio between ticks, so a replay carries it. A rhythm game hears a note at its time plus this: judge a press against the note's time plus the latency, then a calibrated rest (the display, the hand).

Inside a system og.Audio must be among its reads or writes.

```ts
og.audio.outputLatency(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`: Seconds; 0 headless.

Binding symbol `og_audio_output_latency` · publish builds: kept

```ogt
fn outputLatency {
  doc "How long after its world time a sample leaves the speaker, as the host reports it: on the web the AudioContext's baseLatency and outputLatency and the worklet ring's buffer, natively the device's latency (cpal's playback less callback instant) and the mixer's buffer; 0 headless (the null and capture sinks are driven to the tick). Host input, written to og.Audio between ticks, so a replay carries it. A rhythm game hears a note at its time plus this: judge a press against the note's time plus the latency, then a calibrated rest (the display, the hand)."
  doc "Inside a system og.Audio must be among its reads or writes."
  param world World "The world."
  returns f64 "Seconds; 0 headless."
}
```

<sub>`api.ogt` line 13533</sub>
