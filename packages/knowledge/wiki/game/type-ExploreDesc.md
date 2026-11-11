<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-ExploreDesc"></a>

### `ExploreDesc` (desc struct)

`og explore`'s view of a game (GameDesc.explore): the state its archive of cells abstracts and the inputs it perturbs from them. Every field is optional.

A cell counts a named entity's fields by its name, unnamed entities as their number and each field's set of values. Numbers are bucketed (an integer up to 16 by value, a number within -1..1 in eighths, others by power of two, NaN and the infinities apart), bools, strings and enums by value. The time is a cell's too, in 64 slices of the search's horizon, unless `ignore` names `@tick`.

See `explore §5`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `cells` | `readonly string[]` | `[]` | — | — | — | The fields a cell reads, as `Record.field` or a whole `Record` (a resource or a component): what tells two states apart. Empty: every field of the game's own resources and components. See `explore §5`. |
| `ignore` | `readonly string[]` | `[]` | — | — | — | Fields left out of cells (`Record.field` or `Record`): timers, counters and caches that make every state look new; `@tick` for a game whose state does not depend on when things happen. |
| `actions` | `readonly string[]` | `[]` | — | — | — | The actions it presses, by name. Empty: every button action the game defines (og.input.actions), each by its first key or pad binding. |
| `intents` | `readonly ExploreIntent[]` | `[]` | — | — | — | The intents it sends (og.input.intent's events): the recorded input a UI's controls stand for. None by default. |

```ogt
struct ExploreDesc {
  doc "`og explore`'s view of a game (GameDesc.explore): the state its archive of cells abstracts and the inputs it perturbs from them. Every field is optional."
  doc "A cell counts a named entity's fields by its name, unnamed entities as their number and each field's set of values. Numbers are bucketed (an integer up to 16 by value, a number within -1..1 in eighths, others by power of two, NaN and the infinities apart), bools, strings and enums by value. The time is a cell's too, in 64 slices of the search's horizon, unless `ignore` names `@tick`."
  guide "explore §5"
  desc true
  field cells List.String "The fields a cell reads, as `Record.field` or a whole `Record` (a resource or a component): what tells two states apart. Empty: every field of the game's own resources and components." {
    guide "explore §5"
  }
  field ignore List.String "Fields left out of cells (`Record.field` or `Record`): timers, counters and caches that make every state look new; `@tick` for a game whose state does not depend on when things happen."
  field actions List.String "The actions it presses, by name. Empty: every button action the game defines (og.input.actions), each by its first key or pad binding."
  field intents List.ExploreIntent "The intents it sends (og.input.intent's events): the recorded input a UI's controls stand for. None by default."
}
```

<sub>Module `game` line 63</sub>
