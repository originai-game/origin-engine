<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-AudioSnapshot"></a>

### `og.AudioSnapshot` (component)

A mixer snapshot (combat, underwater, paused): per-bus volumes and filters the mix blends towards while `active`, and back when not. Each tick og.audio moves `weight` by tickDuration / fade and sends the blend in the stream; bus gains ramp linearly across each tick.

At weight w a bus's gain is multiplied by 1 + w * (volume - 1); its low-pass moves from 20 kHz and its high-pass from 20 Hz to the snapshot's cutoffs, log2-linear in w. Snapshots multiply gains and meet filters (the lowest low-pass and highest high-pass, AudioBusFx's included); they never write og.Audio or og.AudioBusFx.

See `audio §4`.

Token `og.AudioSnapshot`, registered under the schema name `og.AudioSnapshot`; its value type is `AudioSnapshot`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `active` | `boolean` | `false` | — | — | — | Blends in while true, out while false. |
| `fade` | `number` | `0.5` | 0 … 30 | `s` | tunable | Seconds a full blend in or out takes; 0 is at once. |
| `weight` | `number` | `0` | 0 … 1 | — | read only | How far it is blended in, 0 to 1 (written by og.audio each tick). |
| `volume` | `readonly number[]` | `[]` | — | — | a list of at most 16 `f32` (`List.f32.16`): reads a frozen copy, writes a whole array | Per bus, by index (master, music, sfx, ui, then user buses 4 to 15), the gain it multiplies in at full weight; buses past the list's length keep 1: `{ volume: [1, 0.3] }` turns the music down. |
| `lowpass` | `readonly number[]` | `[]` | — | — | a list of at most 16 `f32` (`List.f32.16`): reads a frozen copy, writes a whole array | Per bus, a low-pass cutoff in Hz at full weight; 0 or past the list's length is none: `{ lowpass: [0, 0, 600] }` muffles sfx (underwater). |
| `highpass` | `readonly number[]` | `[]` | — | — | a list of at most 16 `f32` (`List.f32.16`): reads a frozen copy, writes a whole array | Per bus, a high-pass cutoff in Hz at full weight; 0 or past the list's length is none. |

```ogt
component AudioSnapshot {
  doc "A mixer snapshot (combat, underwater, paused): per-bus volumes and filters the mix blends towards while `active`, and back when not. Each tick og.audio moves `weight` by tickDuration / fade and sends the blend in the stream; bus gains ramp linearly across each tick."
  doc "At weight w a bus's gain is multiplied by 1 + w * (volume - 1); its low-pass moves from 20 kHz and its high-pass from 20 Hz to the snapshot's cutoffs, log2-linear in w. Snapshots multiply gains and meet filters (the lowest low-pass and highest high-pass, AudioBusFx's included); they never write og.Audio or og.AudioBusFx."
  guide "audio §4"
  field active bool "Blends in while true, out while false."
  field fade f32 "Seconds a full blend in or out takes; 0 is at once." {
    default 0.5
    min 0
    max 30
    unit s
    tunable true
  }
  field weight f32 "How far it is blended in, 0 to 1 (written by og.audio each tick)." {
    min 0
    max 1
    readonly true
  }
  field volume List.f32.16 "Per bus, by index (master, music, sfx, ui, then user buses 4 to 15), the gain it multiplies in at full weight; buses past the list's length keep 1: `{ volume: [1, 0.3] }` turns the music down."
  field lowpass List.f32.16 "Per bus, a low-pass cutoff in Hz at full weight; 0 or past the list's length is none: `{ lowpass: [0, 0, 600] }` muffles sfx (underwater)."
  field highpass List.f32.16 "Per bus, a high-pass cutoff in Hz at full weight; 0 or past the list's length is none."
}
```

<sub>Module `audio` line 472</sub>
