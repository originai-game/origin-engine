<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-animate"></a>

### `og.ui.animate`

Any number on a spring or an ease (docs/ui.md §3.1: every value is animatable): call it every tick with the target and it answers this tick's value. A retarget keeps its value and velocity; under reduced motion it jumps.

```ts
og.ui.animate(id: string, target: number, transition: Transition): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id under the open element. |
| `target` | `number` | no | Where it is going. |
| `transition` | `Transition` | no | How it moves. |

**Returns** `number`.

Binding symbol `og_ui_animate` · publish builds: kept

```ogt
fn animate {
  doc "Any number on a spring or an ease (docs/ui.md §3.1: every value is animatable): call it every tick with the target and it answers this tick's value. A retarget keeps its value and velocity; under reduced motion it jumps."
  param id String "Its id under the open element."
  param target f32 "Where it is going."
  param transition Transition "How it moves."
  returns f32
}
```

<sub>Module `ui` line 2781</sub>
