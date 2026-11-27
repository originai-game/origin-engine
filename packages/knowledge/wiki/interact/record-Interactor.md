<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`interact`](../interact.md) · [index](../index.md)

<a id="record-Interactor"></a>

### `og.Interactor` (component)

Who interacts: each tick og.systems.interact looks from its og.Transform (a root entity's) plus `eye`, along its forward (+Z), for og.Interactable entities by og.play.candidates, focuses one by `rule`, and uses it on a press of `action` (og.input.consume) or once the action has been held on it for its holdTicks. While it carries an og.Grab it focuses nothing and the next press throws it.

Also known as: interaction focus, player interact, use key, look at target, interaction prompt source, hold to interact

Token `og.Interactor`, registered under the schema name `og.Interactor`; its value type is `Interactor`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | `str(32)`: at most 32 bytes of UTF-8 | The input action that uses the focus (og.input.defineAction's name); empty uses nothing. |
| `range` | `number` | `0` | ≥ 0 | `m` | tunable | How far it looks; 0 focuses nothing. |
| `cone` | `number` | `0` | ≥ 0 | `rad` | tunable | The half angle about its forward it looks within; 0, or π and more, takes every direction. |
| `sight` | `boolean` | `false` | — | — | — | Skip candidates a collider stands in front of (its own entity's excepted). |
| `sightMask` | `number` | `0` | — | — | — | The collider layers that block sight; 0 is every layer. |
| `rule` | `InteractRule` | `"nearest"` | — | — | — | How it picks among the candidates. |
| `eye` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | Where it looks from, in its own frame. |
| `focus` | `Entity` | `0` | — | — | read only | Written: the og.Interactable focused this tick (0: none), what a prompt shows. |
| `progress` | `number` | `0` | — | — | read only | Written: ticks the action has been held on the focus (0: not holding), for a hold prompt's ring against og.Interactable.holdTicks. |
| `holding` | `Entity` | `0` | — | — | read only | Written: the og.Grab it carries (0: none). |

```ogt
component Interactor {
  aliases "interaction focus, player interact, use key, look at target, interaction prompt source, hold to interact"
  doc "Who interacts: each tick og.systems.interact looks from its og.Transform (a root entity's) plus `eye`, along its forward (+Z), for og.Interactable entities by og.play.candidates, focuses one by `rule`, and uses it on a press of `action` (og.input.consume) or once the action has been held on it for its holdTicks. While it carries an og.Grab it focuses nothing and the next press throws it."
  field action str.32 "The input action that uses the focus (og.input.defineAction's name); empty uses nothing."
  field range f32 "How far it looks; 0 focuses nothing." {
    min 0
    unit m
    tunable true
  }
  field cone f32 "The half angle about its forward it looks within; 0, or π and more, takes every direction." {
    min 0
    unit rad
    tunable true
  }
  field sight bool "Skip candidates a collider stands in front of (its own entity's excepted)."
  field sightMask u32 "The collider layers that block sight; 0 is every layer."
  field rule InteractRule "How it picks among the candidates."
  field eye Vector3 "Where it looks from, in its own frame." {
    unit m
    worldspace none
  }
  field focus Entity "Written: the og.Interactable focused this tick (0: none), what a prompt shows." {
    readonly true
  }
  field progress u32 "Written: ticks the action has been held on the focus (0: not holding), for a hold prompt's ring against og.Interactable.holdTicks." {
    readonly true
  }
  field holding Entity "Written: the og.Grab it carries (0: none)." {
    readonly true
  }
}
```

<sub>Module `interact` line 30</sub>
