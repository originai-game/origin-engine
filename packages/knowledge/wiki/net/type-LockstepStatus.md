<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="type-LockstepStatus"></a>

### `LockstepStatus` (struct)

Lockstep on this peer, as the probe and a HUD read it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `running` | `boolean` | `false` | — | — | — | Whether lockstep runs. |
| `peer` | `number \| null` | `null` | — | — | — | This peer's number; null in no room. |
| `peers` | `readonly number[]` | `[]` | — | — | — | The peers present, by number. |
| `startTick` | `number \| null` | `null` | — | `ticks` | — | The tick lockstep started (or this peer joined it) at. |
| `players` | `number \| null` | `null` | — | — | — | The seats, while running. |
| `seats` | `readonly LockstepSeat[]` | `[]` | — | — | — | Every seat's holds, in order. |
| `spectators` | `readonly number[]` | `[]` | — | — | — | The running peers without a seat. |
| `waitingFor` | `readonly number[]` | `[]` | — | — | — | The seats whose input for the next tick has not arrived. |
| `waits` | `number` | `0` | — | — | — | Tick boundaries this peer waited at for an input. |
| `desync` | `number \| null` | `null` | — | — | — | The first divergent tick, once og.Desync was sent. |
| `rollback` | `boolean` | `false` | — | — | — | Whether the room runs in rollback (og.Lockstep.mode, the authority's). |
| `maxPrediction` | `number \| null` | `null` | — | `ticks` | — | The room's maxPrediction while running in rollback; 0 in lockstep. |
| `confirmedTick` | `number \| null` | `null` | — | `ticks` | — | The last tick whose every seat's input has arrived. |
| `finalTick` | `number \| null` | `null` | — | `ticks` | — | The last tick whose world is final: run on real inputs only, the world og.net.finalHash and the hashes the peers compare cover. |
| `acknowledged` | `number \| null` | `null` | — | `ticks` | — | The last tick through which every other running peer has all of this peer's inputs (each input message acknowledges); null alone or before the start. |
| `predicted` | `number` | `0` | — | `ticks` | — | Ticks the world now runs past its last confirmed tick (0 in lockstep). |
| `rollbacks` | `number` | `0` | — | — | — | Rewinds this peer made since lockstep started. |
| `resimulatedTicks` | `number` | `0` | — | `ticks` | — | Ticks this peer ran again after a rewind. |
| `stalled` | `readonly number[]` | `[]` | — | — | — | The running peers silent for a second or more (og.PeerStalled). |
| `drops` | `number` | `0` | — | — | — | Seats (and spectators) this peer has seen dropped since it joined the room. |
| `snapshotBytes` | `number` | `0` | — | `bytes` | — | The bytes of the snapshots kept for a rewind: one per tick not yet final. |

```ogt
struct LockstepStatus {
  doc "Lockstep on this peer, as the probe and a HUD read it."
  field running bool "Whether lockstep runs."
  field peer Option.u32 "This peer's number; null in no room."
  field peers List.u32 "The peers present, by number."
  field startTick Option.f64 "The tick lockstep started (or this peer joined it) at." {
    unit ticks
  }
  field players Option.u32 "The seats, while running."
  field seats List.LockstepSeat "Every seat's holds, in order."
  field spectators List.u32 "The running peers without a seat."
  field waitingFor List.u32 "The seats whose input for the next tick has not arrived."
  field waits f64 "Tick boundaries this peer waited at for an input."
  field desync Option.u32 "The first divergent tick, once og.Desync was sent."
  field rollback bool "Whether the room runs in rollback (og.Lockstep.mode, the authority's)."
  field maxPrediction Option.u32 "The room's maxPrediction while running in rollback; 0 in lockstep." {
    unit ticks
  }
  field confirmedTick Option.f64 "The last tick whose every seat's input has arrived." {
    unit ticks
  }
  field finalTick Option.f64 "The last tick whose world is final: run on real inputs only, the world og.net.finalHash and the hashes the peers compare cover." {
    unit ticks
  }
  field acknowledged Option.f64 "The last tick through which every other running peer has all of this peer's inputs (each input message acknowledges); null alone or before the start." {
    unit ticks
  }
  field predicted u32 "Ticks the world now runs past its last confirmed tick (0 in lockstep)." {
    unit ticks
  }
  field rollbacks f64 "Rewinds this peer made since lockstep started."
  field resimulatedTicks f64 "Ticks this peer ran again after a rewind." {
    unit ticks
  }
  field stalled List.u32 "The running peers silent for a second or more (og.PeerStalled)."
  field drops f64 "Seats (and spectators) this peer has seen dropped since it joined the room."
  field snapshotBytes f64 "The bytes of the snapshots kept for a rewind: one per tick not yet final." {
    unit bytes
  }
}
```

<sub>`api.ogt` line 26366</sub>
