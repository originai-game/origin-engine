<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-TweenTimeline"></a>

### `TweenTimeline` (interface)

A GSAP timeline (og.tween.timeline): build it with to, from, fromTo, set, call, addLabel and add, each placed by a position and answering the timeline so calls chain; control it with play, pause, reverse, seek, progress, timeScale and kill.

```ogt
interface TweenTimeline {
  doc "A GSAP timeline (og.tween.timeline): build it with to, from, fromTo, set, call, addLabel and add, each placed by a position and answering the timeline so calls chain; control it with play, pause, reverse, seek, progress, timeScale and kill."
  # members follow, each quoted below
```

<sub>Module `tween` line 230</sub>

<a id="member-TweenTimeline-tween"></a>

#### `TweenTimeline.tween` (property)

Its handle: og.TweenCompleted and og.TweenEvent carry it, og.tween.cancel and active take it.

```ts
readonly tween: Tween
```

Binding symbol `og_tween_timeline_get_tween` · publish builds: kept

```ogt
prop tween Tween "Its handle: og.TweenCompleted and og.TweenEvent carry it, og.tween.cancel and active take it."
```

<sub>Module `tween` line 232</sub>

<a id="member-TweenTimeline-error"></a>

#### `TweenTimeline.error` (property)

Why the last refused call added nothing; null when none was refused.

```ts
readonly error: string | null
```

Binding symbol `og_tween_timeline_get_error` · publish builds: kept

```ogt
prop error Option.String "Why the last refused call added nothing; null when none was refused."
```

<sub>Module `tween` line 233</sub>

<a id="member-TweenTimeline-to"></a>

#### `TweenTimeline.to`

Tween a field of every target to `vars.to` (or through `vars.keyframes`), from its value when the tween starts.

```ts
TweenTimeline.to(targets: Entity | readonly Entity[], vars?: TimelineTweenVars, position?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `targets` | `Entity \| readonly Entity[]` | no | An entity or several (staggered by vars.stagger). |
| `vars` | `TimelineTweenVars` | yes | The field, the values and the timing. |
| `position` | `number \| string` | yes | Where: seconds or GSAP's position text; null appends. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_to` · publish builds: kept

```ogt
fn to {
  doc "Tween a field of every target to `vars.to` (or through `vars.keyframes`), from its value when the tween starts."
  param targets OneOf.Entity.List.Entity "An entity or several (staggered by vars.stagger)."
  param vars TimelineTweenVars "The field, the values and the timing."
  param position OneOf.f64.String "Where: seconds or GSAP's position text; null appends." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 234</sub>

<a id="member-TweenTimeline-from"></a>

#### `TweenTimeline.from`

Tween from `vars.from` to each target's value at this call (read now: a system needs read access to the component). The start value is written on the timeline's first render.

```ts
TweenTimeline.from(targets: Entity | readonly Entity[], vars?: TimelineTweenVars, position?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `targets` | `Entity \| readonly Entity[]` | no | An entity or several. |
| `vars` | `TimelineTweenVars` | yes | The field, the start value and the timing. |
| `position` | `number \| string` | yes | Where. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_from` · publish builds: kept

```ogt
fn from {
  doc "Tween from `vars.from` to each target's value at this call (read now: a system needs read access to the component). The start value is written on the timeline's first render."
  param targets OneOf.Entity.List.Entity "An entity or several."
  param vars TimelineTweenVars "The field, the start value and the timing."
  param position OneOf.f64.String "Where." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 243</sub>

<a id="member-TweenTimeline-fromTo"></a>

#### `TweenTimeline.fromTo`

Tween from `vars.from` to `vars.to`; both are required. The start value is written on the timeline's first render.

```ts
TweenTimeline.fromTo(targets: Entity | readonly Entity[], vars?: TimelineTweenVars, position?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `targets` | `Entity \| readonly Entity[]` | no | An entity or several. |
| `vars` | `TimelineTweenVars` | yes | The field, both values and the timing. |
| `position` | `number \| string` | yes | Where. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_from_to` · publish builds: kept

```ogt
fn fromTo {
  doc "Tween from `vars.from` to `vars.to`; both are required. The start value is written on the timeline's first render."
  param targets OneOf.Entity.List.Entity "An entity or several."
  param vars TimelineTweenVars "The field, both values and the timing."
  param position OneOf.f64.String "Where." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 252</sub>

<a id="member-TweenTimeline-set"></a>

#### `TweenTimeline.set`

Set a field of every target to `vars.to` when the playhead reaches the position (a tween of no duration; going back before it restores the value it found).

```ts
TweenTimeline.set(targets: Entity | readonly Entity[], vars?: TimelineTweenVars, position?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `targets` | `Entity \| readonly Entity[]` | no | An entity or several. |
| `vars` | `TimelineTweenVars` | yes | The field and the value. |
| `position` | `number \| string` | yes | Where. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_set` · publish builds: kept

```ogt
fn set {
  doc "Set a field of every target to `vars.to` when the playhead reaches the position (a tween of no duration; going back before it restores the value it found)."
  param targets OneOf.Entity.List.Entity "An entity or several."
  param vars TimelineTweenVars "The field and the value."
  param position OneOf.f64.String "Where." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 261</sub>

<a id="member-TweenTimeline-call"></a>

#### `TweenTimeline.call`

Send og.TweenEvent `name` when the playhead crosses the position, forwards or backwards (GSAP's call, named instead of a function).

```ts
TweenTimeline.call(name: string, position?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The event's name (at most 63 bytes). |
| `position` | `number \| string` | yes | Where. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_call` · publish builds: kept

```ogt
fn call {
  doc "Send og.TweenEvent `name` when the playhead crosses the position, forwards or backwards (GSAP's call, named instead of a function)."
  param name String "The event's name (at most 63 bytes)."
  param position OneOf.f64.String "Where." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 270</sub>

<a id="member-TweenTimeline-addLabel"></a>

#### `TweenTimeline.addLabel`

Name a time, for positions (`"label"`, `"label+=0.5"`) and seek.

```ts
TweenTimeline.addLabel(label: string, position?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `label` | `string` | no | The label. |
| `position` | `number \| string` | yes | Where; null is the end. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_add_label` · publish builds: kept

```ogt
fn addLabel {
  doc "Name a time, for positions (`\"label\"`, `\"label+=0.5\"`) and seek."
  param label String "The label."
  param position OneOf.f64.String "Where; null is the end." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 278</sub>

<a id="member-TweenTimeline-add"></a>

#### `TweenTimeline.add`

Nest a running tween, sequence, parallel group or timeline (its `tween` handle) at the position: it stops running on its own and plays as part of this one. A string adds a label.

```ts
TweenTimeline.add(child: Tween | string, position?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `child` | `Tween \| string` | no | The handle to nest, or a label. |
| `position` | `number \| string` | yes | Where. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_add` · publish builds: kept

```ogt
fn add {
  doc "Nest a running tween, sequence, parallel group or timeline (its `tween` handle) at the position: it stops running on its own and plays as part of this one. A string adds a label."
  param child OneOf.Tween.String "The handle to nest, or a label."
  param position OneOf.f64.String "Where." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 286</sub>

<a id="member-TweenTimeline-play"></a>

#### `TweenTimeline.play`

Play forwards, from a time or label when given.

```ts
TweenTimeline.play(from?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `from` | `number \| string` | yes | Where to start. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_play` · publish builds: kept

```ogt
fn play {
  doc "Play forwards, from a time or label when given."
  param from OneOf.f64.String "Where to start." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 294</sub>

<a id="member-TweenTimeline-pause"></a>

#### `TweenTimeline.pause`

Hold the playhead, at a time or label when given.

```ts
TweenTimeline.pause(at?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `at` | `number \| string` | yes | Where to hold. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_pause` · publish builds: kept

```ogt
fn pause {
  doc "Hold the playhead, at a time or label when given."
  param at OneOf.f64.String "Where to hold." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 301</sub>

<a id="member-TweenTimeline-resume"></a>

#### `TweenTimeline.resume`

Play on in the direction it had.

```ts
TweenTimeline.resume(): TweenTimeline
```

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_resume` · publish builds: kept

```ogt
fn resume {
  doc "Play on in the direction it had."
  returns TweenTimeline
}
```

<sub>Module `tween` line 308</sub>

<a id="member-TweenTimeline-reverse"></a>

#### `TweenTimeline.reverse`

Play backwards towards the start, from a time or label when given.

```ts
TweenTimeline.reverse(from?: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `from` | `number \| string` | yes | Where to start. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_reverse` · publish builds: kept

```ogt
fn reverse {
  doc "Play backwards towards the start, from a time or label when given."
  param from OneOf.f64.String "Where to start." {
    optional true
  }
  returns TweenTimeline
}
```

<sub>Module `tween` line 312</sub>

<a id="member-TweenTimeline-restart"></a>

#### `TweenTimeline.restart`

Play forwards from the start (callbacks fire again).

```ts
TweenTimeline.restart(): TweenTimeline
```

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_restart` · publish builds: kept

```ogt
fn restart {
  doc "Play forwards from the start (callbacks fire again)."
  returns TweenTimeline
}
```

<sub>Module `tween` line 319</sub>

<a id="member-TweenTimeline-seek"></a>

#### `TweenTimeline.seek`

Jump the playhead to a time or label without sending callbacks; it keeps playing or holding.

```ts
TweenTimeline.seek(position: number | string): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `position` | `number \| string` | no | Seconds or a label. |

**Returns** `TweenTimeline`.

Binding symbol `og_tween_timeline_seek` · publish builds: kept

```ogt
fn seek {
  doc "Jump the playhead to a time or label without sending callbacks; it keeps playing or holding."
  param position OneOf.f64.String "Seconds or a label."
  returns TweenTimeline
}
```

<sub>Module `tween` line 323</sub>

<a id="member-TweenTimeline-progress"></a>

#### `TweenTimeline.progress`

How far through its cycle the playhead is, 0 to 1; given a value, jumps there first (without callbacks).

```ts
TweenTimeline.progress(value?: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `value` | `number` | yes | The progress to jump to. |

**Returns** `number`.

Binding symbol `og_tween_timeline_progress` · publish builds: kept

```ogt
fn progress {
  doc "How far through its cycle the playhead is, 0 to 1; given a value, jumps there first (without callbacks)."
  param value f64 "The progress to jump to." {
    optional true
  }
  returns f64
}
```

<sub>Module `tween` line 328</sub>

<a id="member-TweenTimeline-time"></a>

#### `TweenTimeline.time`

The playhead's time within its cycle, in seconds.

```ts
TweenTimeline.time(): number
```

**Returns** `number`.

Binding symbol `og_tween_timeline_time` · publish builds: kept

```ogt
fn time {
  doc "The playhead's time within its cycle, in seconds."
  returns f64
}
```

<sub>Module `tween` line 335</sub>

<a id="member-TweenTimeline-duration"></a>

#### `TweenTimeline.duration`

One cycle, in seconds: where its last child ends.

```ts
TweenTimeline.duration(): number
```

**Returns** `number`.

Binding symbol `og_tween_timeline_duration` · publish builds: kept

```ogt
fn duration {
  doc "One cycle, in seconds: where its last child ends."
  returns f64
}
```

<sub>Module `tween` line 339</sub>

<a id="member-TweenTimeline-timeScale"></a>

#### `TweenTimeline.timeScale`

The playback rate; given a value, sets it first (2 twice as fast, 0 holds).

```ts
TweenTimeline.timeScale(value?: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `value` | `number` | yes | The new rate, 0 or more. |

**Returns** `number`.

Binding symbol `og_tween_timeline_time_scale` · publish builds: kept

```ogt
fn timeScale {
  doc "The playback rate; given a value, sets it first (2 twice as fast, 0 holds)."
  param value f64 "The new rate, 0 or more." {
    optional true
  }
  returns f64
}
```

<sub>Module `tween` line 343</sub>

<a id="member-TweenTimeline-isActive"></a>

#### `TweenTimeline.isActive`

Whether it is running and not paused.

```ts
TweenTimeline.isActive(): boolean
```

**Returns** `boolean`.

Binding symbol `og_tween_timeline_is_active` · publish builds: kept

```ogt
fn isActive {
  doc "Whether it is running and not paused."
  returns bool
}
```

<sub>Module `tween` line 350</sub>

<a id="member-TweenTimeline-kill"></a>

#### `TweenTimeline.kill`

Stop and remove it where it is (no events); its fields keep their values.

```ts
TweenTimeline.kill(): boolean
```

**Returns** `boolean`.

Binding symbol `og_tween_timeline_kill` · publish builds: kept

```ogt
fn kill {
  doc "Stop and remove it where it is (no events); its fields keep their values."
  returns bool
}
```

<sub>Module `tween` line 354</sub>
