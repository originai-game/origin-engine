<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-objectiveTracker"></a>

### `og.ui.objectiveTracker`

An objective tracker to the bar of open worlds and MMOs: each objective's title by its icon with its distance and a countdown, its steps as boxes that tick with a pop when done (dimmed after), counts that pop as they rise, progress bars, optional steps muted, failed steps crossed in danger, the focused one marked, a completed one flashing Complete and leaving; over a soft scrim so it reads over any scene.

Parts: ObjectiveTracker.quest[focused][complete], ObjectiveTracker.title, ObjectiveTracker.step[done|failed|optional], ObjectiveTracker.check[done], ObjectiveTracker.count, ObjectiveTracker.progress, ObjectiveTracker.timer, ObjectiveTracker.distance. States: on each step, group step (open, done, failed). Sounds: success when a step is done, reward when a quest completes. Accessibility: list named Objectives; each quest a group named its title; each step a list item, its value done or failed and its count (the tracker takes no taps). Probe ids: <id>/<key>, <id>/<key>/<step>.

```ts
og.ui.objectiveTracker(id: string, desc?: ObjectiveTrackerDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ObjectiveTrackerDesc` | yes | The objectives. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn objectiveTracker {
  doc "An objective tracker to the bar of open worlds and MMOs: each objective's title by its icon with its distance and a countdown, its steps as boxes that tick with a pop when done (dimmed after), counts that pop as they rise, progress bars, optional steps muted, failed steps crossed in danger, the focused one marked, a completed one flashing Complete and leaving; over a soft scrim so it reads over any scene."
  doc "Parts: ObjectiveTracker.quest[focused][complete], ObjectiveTracker.title, ObjectiveTracker.step[done|failed|optional], ObjectiveTracker.check[done], ObjectiveTracker.count, ObjectiveTracker.progress, ObjectiveTracker.timer, ObjectiveTracker.distance. States: on each step, group step (open, done, failed). Sounds: success when a step is done, reward when a quest completes. Accessibility: list named Objectives; each quest a group named its title; each step a list item, its value done or failed and its count (the tracker takes no taps). Probe ids: <id>/<key>, <id>/<key>/<step>."
  host false
  param id String "Its id."
  param desc ObjectiveTrackerDesc "The objectives."
  returns Void
}
```

<sub>Module `ui` line 6358</sub>
