<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="record-Lockstep"></a>

### `og.Lockstep` (resource)

Lockstep's settings and this peer's place. The game sets enabled, players, inputDelay and hashEvery (the authority's are the room's once it starts: changes while running wait for the next start); the engine sets localPlayer and running. It is this peer's own: never hashed, never compared.

Token `og.Lockstep`, registered under the schema name `og.Lockstep`; its value type is `Lockstep`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Whether the world takes its ticks in lockstep when in a room. |
| `players` | `number` | `2` | 1 … 16 | — | — | The seats: the peers lockstep waits for before it starts, and whose inputs each tick waits for. |
| `localPlayer` | `number` | `0` | — | — | read only | This peer's seat, the og.PlayerInput.player its input drives; `players` while it watches (a peer beyond the seats). 0 before the start. |
| `inputDelay` | `number` | `2` | 1 … 30 | `ticks` | — | Ticks between a player's input and the tick it acts on: the time the input has to cross the room. 2 (33 ms at 60 Hz) on a LAN, 4 to 6 over the internet. |
| `hashEvery` | `number` | `10` | 0 … 600 | `ticks` | — | How often the peers exchange their world hashes, in ticks (every tick is hashed, so the first divergent tick is exact); 0 compares nothing. |
| `running` | `boolean` | `false` | — | — | read only | Whether lockstep runs: the room filled the seats and the authority started (until this peer leaves the room or turns it off). |
| `mode` | `NetMode` | `"lockstep"` | — | — | — | lockstep (the default: every tick waits for every input) or rollback (ticks run on predicted inputs and are rewound when an input differs; set before the room starts, the authority's is the room's). |
| `stallTimeout` | `number` | `5000` | 0 … 600000 | `ms` | — | Milliseconds a running peer may stay silent before its seat is dropped (og.SeatDropped); 0 never drops a peer still in the room, so the room waits for it as long as it takes. The authority's is the room's. |
| `rejoin` | `boolean` | `true` | — | — | — | Whether a dropped player that comes back (or a newcomer of the same name) takes its seat again (og.SeatRejoined); when off, a seat once dropped stays empty and newcomers take only seats never held. The authority's is the room's. |
| `maxPrediction` | `number` | `8` | 1 … 30 | `ticks` | — | In rollback, the most ticks this peer runs past the last tick whose inputs are all in before it waits (a peer also predicts no further than it can re-simulate in half a frame). 8 (133 ms at 60 Hz) covers most internet latency on top of inputDelay. |

```ogt
resource Lockstep {
  doc "Lockstep's settings and this peer's place. The game sets enabled, players, inputDelay and hashEvery (the authority's are the room's once it starts: changes while running wait for the next start); the engine sets localPlayer and running. It is this peer's own: never hashed, never compared."
  field enabled bool "Whether the world takes its ticks in lockstep when in a room."
  field players u32 "The seats: the peers lockstep waits for before it starts, and whose inputs each tick waits for." {
    default 2
    min 1
    max 16
  }
  field localPlayer u32 "This peer's seat, the og.PlayerInput.player its input drives; `players` while it watches (a peer beyond the seats). 0 before the start." {
    readonly true
  }
  field inputDelay u32 "Ticks between a player's input and the tick it acts on: the time the input has to cross the room. 2 (33 ms at 60 Hz) on a LAN, 4 to 6 over the internet." {
    default 2
    min 1
    max 30
    unit ticks
  }
  field hashEvery u32 "How often the peers exchange their world hashes, in ticks (every tick is hashed, so the first divergent tick is exact); 0 compares nothing." {
    default 10
    min 0
    max 600
    unit ticks
  }
  field running bool "Whether lockstep runs: the room filled the seats and the authority started (until this peer leaves the room or turns it off)." {
    readonly true
  }
  field mode NetMode "lockstep (the default: every tick waits for every input) or rollback (ticks run on predicted inputs and are rewound when an input differs; set before the room starts, the authority's is the room's)."
  field stallTimeout u32 "Milliseconds a running peer may stay silent before its seat is dropped (og.SeatDropped); 0 never drops a peer still in the room, so the room waits for it as long as it takes. The authority's is the room's." {
    default 5000
    min 0
    max 600000
    unit ms
  }
  field rejoin bool "Whether a dropped player that comes back (or a newcomer of the same name) takes its seat again (og.SeatRejoined); when off, a seat once dropped stays empty and newcomers take only seats never held. The authority's is the room's." {
    default true
  }
  field maxPrediction u32 "In rollback, the most ticks this peer runs past the last tick whose inputs are all in before it waits (a peer also predicts no further than it can re-simulate in half a frame). 8 (133 ms at 60 Hz) covers most internet latency on top of inputDelay." {
    default 8
    min 1
    max 30
    unit ticks
  }
}
```

<sub>`api.ogt` line 26148</sub>
