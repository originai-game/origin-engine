<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DesyncDump"></a>

### `DesyncDump` (struct)

The first desync of this peer's lockstep session (og.net), as `dump_desync` writes it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `desync` | `boolean` | `false` | — | — | — | Whether a desync happened; false lists the ticks compared with each peer so far. |
| `tick` | `number \| null` | `null` | — | — | — | The first divergent tick: og.Time.tick of the two worlds that differ. |
| `local` | `unknown` | `null` | — | — | — | This peer: `{ peer, player, hash }`. |
| `remote` | `unknown` | `null` | — | — | — | The other peer: `{ peer, player, hash }`. |
| `diff` | `unknown` | `null` | — | — | — | What differs: `{ components: [{ entity, component, fields: [{ field, local, remote }] }], entities: { onlyLocal, onlyRemote }, resources, events, blocks, allocator }`, in the local world's entity order (the first entry is the first difference in storage order); `{ pending }` until the other peer's snapshot arrived. |

```ogt
struct DesyncDump {
  doc "The first desync of this peer's lockstep session (og.net), as `dump_desync` writes it."
  field desync bool "Whether a desync happened; false lists the ticks compared with each peer so far."
  field tick Option.u32 "The first divergent tick: og.Time.tick of the two worlds that differ."
  field local Json "This peer: `{ peer, player, hash }`."
  field remote Json "The other peer: `{ peer, player, hash }`."
  field diff Json "What differs: `{ components: [{ entity, component, fields: [{ field, local, remote }] }], entities: { onlyLocal, onlyRemote }, resources, events, blocks, allocator }`, in the local world's entity order (the first entry is the first difference in storage order); `{ pending }` until the other peer's snapshot arrived."
}
```

<sub>`api.ogt` line 24436</sub>
