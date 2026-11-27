<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-probe-record"></a>
### `file:probe-record`

Derived bake record, not an authored input.

| Field | Arguments | Default | Meaning |
|---|---|---|---|
| `position` | `"<x>" "<y>" "<z>"` | `0 0 0` | World position in metres, stored as quoted numeric strings. |
| `box` | `"<x>" "<y>" "<z>"` | `0 0 0` | World box size in metres, stored as quoted numeric strings. |
| `room` | `"<hash>"` | `` | Captured room content hash. |
| `commands` | `"<commands>"` | `` | Commands which reached the capture. |
| `asset` | `"<id>" "<hex hash>"` | `` | Repeated captured model output hashes. |
| `code` | `"<hex hash>"` | `none` | Source hash when code built the captured room. |
