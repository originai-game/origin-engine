<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-BuildProfile"></a>

### `BuildProfile` (enum)

The build configuration; publish builds keep only the player-facing probe members.

A string: one of the variants below. The zero value is `"dev"`.

| Variant | Doc |
|---|---|
| `"dev"` | A development build: every member, the command channel, dumps. |
| `"release"` | An optimised build with every member. |
| `"publish"` | A published game: hello, tick, state, pick, entity, input and capture only. |

```ogt
enum BuildProfile {
  doc "The build configuration; publish builds keep only the player-facing probe members."
  variant dev "A development build: every member, the command channel, dumps."
  variant release "An optimised build with every member."
  variant publish "A published game: hello, tick, state, pick, entity, input and capture only."
}
```

<sub>`api.ogt` line 24088</sub>
