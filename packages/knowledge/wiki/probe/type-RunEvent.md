<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-RunEvent"></a>

### `RunEvent` (struct)

One thing that happened in a run, at its tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick it happened at (og.Time.tick). |
| `kind` | `string` | `""` | — | — | — | `input` (an event through the probe's input), `step` (a command channel step that ran), `screen` (the UI's screen stack changed), `assertion` (an assertion site's first firing, an each-bucket's first holding or a first violation) or `expect` (an expectation evaluated). |
| `what` | `string` | `""` | — | — | — | A line a reader skims: `pointer down 320,180`, `tap:Play`, `title > shop`, `violation hp-positive`, `fail state:started==true`. |
| `detail` | `unknown` | `null` | — | — | — | The whole of it: the input event, the step's action and args, the screens before and after, the assertion event, the ExpectResult. |

```ogt
struct RunEvent {
  doc "One thing that happened in a run, at its tick."
  field tick u32 "The tick it happened at (og.Time.tick)."
  field kind String "`input` (an event through the probe's input), `step` (a command channel step that ran), `screen` (the UI's screen stack changed), `assertion` (an assertion site's first firing, an each-bucket's first holding or a first violation) or `expect` (an expectation evaluated)."
  field what String "A line a reader skims: `pointer down 320,180`, `tap:Play`, `title > shop`, `violation hp-positive`, `fail state:started==true`."
  field detail Json "The whole of it: the input event, the step's action and args, the screens before and after, the assertion event, the ExpectResult."
}
```

<sub>Module `probe` line 251</sub>
