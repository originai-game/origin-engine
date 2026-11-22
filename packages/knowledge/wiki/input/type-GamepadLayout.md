<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-GamepadLayout"></a>

### `GamepadLayout` (enum)

The order a raw pad's buttons and axes follow: who listed them, so og-input knows which mapping reads it.

A string: one of the variants below. The zero value is `"standard"`.

| Variant | Doc |
|---|---|
| `"standard"` | The W3C standard layout already (a browser's `mapping: "standard"`). |
| `"web"` | A browser's own order for a pad it does not map (its hat as two axes). |
| `"linux"` | Linux evdev, as SDL numbers it. |
| `"windows"` | DirectInput or raw HID on Windows, as SDL numbers it. |
| `"macos"` | IOKit HID on macOS, as SDL numbers it. |
| `"xinput"` | Windows.Gaming.Input's Gamepad: West, South, East, North, the bumpers, View, Menu, the stick presses, the d-pad up, right, down, left; left x, y, right x, the triggers, right y. |

```ogt
enum GamepadLayout {
  doc "The order a raw pad's buttons and axes follow: who listed them, so og-input knows which mapping reads it."
  variant standard "The W3C standard layout already (a browser's `mapping: \"standard\"`)."
  variant web "A browser's own order for a pad it does not map (its hat as two axes)."
  variant linux "Linux evdev, as SDL numbers it."
  variant windows "DirectInput or raw HID on Windows, as SDL numbers it."
  variant macos "IOKit HID on macOS, as SDL numbers it."
  variant xinput "Windows.Gaming.Input's Gamepad: West, South, East, North, the bumpers, View, Menu, the stick presses, the d-pad up, right, down, left; left x, y, right x, the triggers, right y."
}
```

<sub>Module `input` line 689</sub>
