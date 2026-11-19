<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-PlatformHanded"></a>

### `PlatformHanded` (struct)

One thing the host handed the engine, as the engine took it: an entry of og.platform.journal.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | `ticks` | — | The world's tick (world.tick) when it was handed; it applies at the next tick's start. |
| `kind` | `string` | `""` | — | — | — | `host` (what answers was attached, or its clock told), `answer`, `notice`, or `shared` (a shared-world event, og_shared_deliver's, without its token). |
| `request` | `number` | `0` | — | — | — | The call an answer answers; 0 for the others. |
| `value` | `unknown` | `null` | — | — | — | What was handed, with the time the host gave it: the host's info, the answer or the notice. |

```ogt
struct PlatformHanded {
  doc "One thing the host handed the engine, as the engine took it: an entry of og.platform.journal."
  field tick f64 "The world's tick (world.tick) when it was handed; it applies at the next tick's start." {
    unit ticks
  }
  field kind String "`host` (what answers was attached, or its clock told), `answer`, `notice`, or `shared` (a shared-world event, og_shared_deliver's, without its token)."
  field request u32 "The call an answer answers; 0 for the others."
  field value Json "What was handed, with the time the host gave it: the host's info, the answer or the notice."
}
```

<sub>Module `platform` line 465</sub>
