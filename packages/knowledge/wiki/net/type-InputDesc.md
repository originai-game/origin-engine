<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="type-InputDesc"></a>

### `InputDesc` (desc struct)

This peer's player's input: og.PlayerInput's fields but `player` and `tick`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `buttons` | `number` | `0` | — | — | — | The buttons held, one bit each. |
| `x` | `number` | `0` | — | — | — | The move axis, -1 to 1. |
| `y` | `number` | `0` | — | — | — | The move axis, -1 to 1. |
| `aimX` | `number` | `0` | — | — | — | A second axis. |
| `aimY` | `number` | `0` | — | — | — | A second axis. |

```ogt
struct InputDesc {
  doc "This peer's player's input: og.PlayerInput's fields but `player` and `tick`."
  desc true
  field buttons u32 "The buttons held, one bit each."
  field x f32 "The move axis, -1 to 1."
  field y f32 "The move axis, -1 to 1."
  field aimX f32 "A second axis."
  field aimY f32 "A second axis."
}
```

<sub>Module `net` line 119</sub>
