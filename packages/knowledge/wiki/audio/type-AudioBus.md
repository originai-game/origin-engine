<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-AudioBus"></a>

### `AudioBus` (enum)

The bus a source plays on; every bus passes through master.

A string: one of the variants below. The zero value is `"master"`.

| Variant | Doc |
|---|---|
| `"master"` | The master bus: what every bus passes through. |
| `"music"` | Music. |
| `"sfx"` | Sound effects. |
| `"ui"` | Interface sounds. |

```ogt
enum AudioBus {
  doc "The bus a source plays on; every bus passes through master."
  variant master "The master bus: what every bus passes through."
  variant music "Music."
  variant sfx "Sound effects."
  variant ui "Interface sounds."
}
```

<sub>Module `audio` line 13</sub>
