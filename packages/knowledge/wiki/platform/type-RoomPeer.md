<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-RoomPeer"></a>

### `RoomPeer` (struct)

One peer in the room.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `peer` | `number` | `0` | — | — | — | Its number: the room numbers peers in join order from 0 and never gives one twice. |
| `name` | `string` | `""` | — | — | — | Its name, as its host gave it; empty for none. |
| `local` | `boolean` | `false` | — | — | — | Whether it is this engine. |

```ogt
struct RoomPeer {
  doc "One peer in the room."
  field peer u32 "Its number: the room numbers peers in join order from 0 and never gives one twice."
  field name String "Its name, as its host gave it; empty for none."
  field local bool "Whether it is this engine."
}
```

<sub>`api.ogt` line 15396</sub>
