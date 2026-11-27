<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-GamepadInput"></a>

### `GamepadInput` (struct)

A gamepad's raw state, whole (an engine extension of protocol 5): the host sends one whenever a pad changes, and a playtest can send its own. og-input maps it to og.Gamepad's standard layout through SDL_GameControllerDB mappings.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pad` | `number` | `0` | ≤ 7 | — | optional | Which pad, 0 to 7; omitted is 0. A host keeps a pad's slot while it is connected. |
| `connected` | `boolean` | `false` | — | — | optional | Whether the pad is connected (omitted is true); a disconnected pad holds nothing. |
| `layout` | `GamepadLayout` | `"standard"` | — | — | optional | The order its buttons and axes follow (omitted is standard). |
| `guid` | `string` | `""` | — | — | optional | Its SDL GUID, 32 hex digits (bus, CRC, vendor, product and version, little-endian): which mapping reads it and its family. Omitted is none. |
| `buttons` | `readonly number[]` | `[]` | — | — | optional | Each raw button's value, 0 to 1, up to 24 (omitted is none). In the standard layout these are GamepadButton's, the triggers' analog values at 6 and 7. |
| `axes` | `readonly number[]` | `[]` | — | — | optional | Each raw axis, -1 to 1, up to 8; missing values are 0. In the standard layout: left stick x, y, right stick x, y, y down. |
| `hat` | `number` | `0` | ≤ 15 | — | optional | Hat 0's held directions: 1 up, 2 right, 4 down, 8 left (omitted is none). |

```ogt
struct GamepadInput {
  doc "A gamepad's raw state, whole (an engine extension of protocol 5): the host sends one whenever a pad changes, and a playtest can send its own. og-input maps it to og.Gamepad's standard layout through SDL_GameControllerDB mappings."
  field pad u32 "Which pad, 0 to 7; omitted is 0. A host keeps a pad's slot while it is connected." {
    max 7
    optional true
  }
  field connected bool "Whether the pad is connected (omitted is true); a disconnected pad holds nothing." {
    optional true
  }
  field layout GamepadLayout "The order its buttons and axes follow (omitted is standard)." {
    optional true
  }
  field guid String "Its SDL GUID, 32 hex digits (bus, CRC, vendor, product and version, little-endian): which mapping reads it and its family. Omitted is none." {
    optional true
  }
  field buttons List.f32 "Each raw button's value, 0 to 1, up to 24 (omitted is none). In the standard layout these are GamepadButton's, the triggers' analog values at 6 and 7." {
    optional true
  }
  field axes List.f32 "Each raw axis, -1 to 1, up to 8; missing values are 0. In the standard layout: left stick x, y, right stick x, y, y down." {
    optional true
  }
  field hat u32 "Hat 0's held directions: 1 up, 2 right, 4 down, 8 left (omitted is none)." {
    max 15
    optional true
  }
}
```

<sub>Module `input` line 690</sub>
