<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`interact`](../interact.md) · [index](../index.md)

<a id="record-Toggle"></a>

### `og.Toggle` (component)

Something on or off: using it (og.Interactable) flips `on`; a `plate` is on exactly while its entity's sensor collider holds anything (og.physics.sensorContents), and using it does nothing. An og.Door with this as its `toggle` follows it.

Also known as: lever, switch, button, pressure plate, floor switch, on off

Token `og.Toggle`, registered under the schema name `og.Toggle`; its value type is `Toggle`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `on` | `boolean` | `false` | — | — | — | Whether it is on. |
| `plate` | `boolean` | `false` | — | — | — | On while anything is inside its sensor (an og.Collider with `sensor` on its entity). |

```ogt
component Toggle {
  aliases "lever, switch, button, pressure plate, floor switch, on off"
  doc "Something on or off: using it (og.Interactable) flips `on`; a `plate` is on exactly while its entity's sensor collider holds anything (og.physics.sensorContents), and using it does nothing. An og.Door with this as its `toggle` follows it."
  field on bool "Whether it is on."
  field plate bool "On while anything is inside its sensor (an og.Collider with `sensor` on its entity)."
}
```

<sub>Module `interact` line 84</sub>
