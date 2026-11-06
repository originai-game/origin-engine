<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-RoomDesc"></a>

### `RoomDesc` (desc struct)

A room to create.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | What the room is called (at most 64 of letters, digits, `.`, `_` and `-`); empty for none. The host gives the room its id. |
| `capacity` | `number` | `2` | 1 … 16 | — | — | The most peers it takes. |

```ogt
struct RoomDesc {
  doc "A room to create."
  desc true
  field name String "What the room is called (at most 64 of letters, digits, `.`, `_` and `-`); empty for none. The host gives the room its id."
  field capacity u32 "The most peers it takes." {
    default 2
    min 1
    max 16
  }
}
```

<sub>`api.ogt` line 15367</sub>
