<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-Hello"></a>

### `Hello` (struct)

The wire's hello: who is speaking.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `protocol` | `number` | `0` | — | — | — | PROBE_VERSION. |
| `wire` | `number` | `0` | — | — | — | PROBE_WIRE_VERSION. |
| `substrate` | `string` | `""` | — | — | — | Always `origin-engine`. |
| `engine` | `string` | `""` | — | — | — | `origin-engine <version>`. |
| `ticksPerSecond` | `number` | `0` | — | `hz` | — | The fixed tick rate. |
| `tier` | `RenderTier` | `"webgpu"` | — | — | — | The renderer backend. |
| `build` | `BuildProfile` | `"dev"` | — | — | — | The build configuration. |
| `members` | `readonly string[]` | `[]` | — | — | — | The probe members this build serves. |
| `modules` | `readonly string[]` | `[]` | — | — | — | The engine modules this build carries beyond the core (`physics`, `audio`, `ui`, `cook`), so a client knows which members do something. |

```ogt
struct Hello {
  doc "The wire's hello: who is speaking."
  field protocol u32 "PROBE_VERSION."
  field wire u32 "PROBE_WIRE_VERSION."
  field substrate String "Always `origin-engine`."
  field engine String "`origin-engine <version>`."
  field ticksPerSecond u32 "The fixed tick rate." {
    unit hz
  }
  field tier RenderTier "The renderer backend."
  field build BuildProfile "The build configuration."
  field members List.String "The probe members this build serves."
  field modules List.String "The engine modules this build carries beyond the core (`physics`, `audio`, `ui`, `cook`), so a client knows which members do something."
}
```

<sub>`api.ogt` line 24175</sub>
