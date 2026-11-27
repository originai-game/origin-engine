<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `abilities` module

The records og.systems.abilities carries out: an og.Ability is used on a press of its action or by og.abilities.use when it has a charge, its lock has run out and its pool holds its cost; its charges return one by one. An og.Status on a target merges with one of the same kind by its stacking rule and runs out after its ticks; an og.Stat's value is its base with the og.Modifier entities naming it applied in a fixed order. Every number is the game's: an ability of 0 charges is never used.

- **TypeScript**: functions are the namespace `og.abilities`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module](index.md).

```ogt
module abilities {
  doc "The records og.systems.abilities carries out: an og.Ability is used on a press of its action or by og.abilities.use when it has a charge, its lock has run out and its pool holds its cost; its charges return one by one. An og.Status on a target merges with one of the same kind by its stacking rule and runs out after its ticks; an og.Stat's value is its base with the og.Modifier entities naming it applied in a fixed order. Every number is the game's: an ability of 0 charges is never used."
  # declarations follow, each on its own page
```

<sub>Module `abilities` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`StatusStacking`](abilities/type-StatusStacking.md) | enum |  | What a second og.Status of the same kind on the same target does to the first. |
| [`StatusStacking()`](#column-StatusStacking) | column kind |  | A StatusStacking field: the variant's index. |
| [`ModifierOp`](abilities/type-ModifierOp.md) | enum |  | How an og.Modifier changes its og.Stat: all adds, then the percents summed, then each multiply. |
| [`ModifierOp()`](#column-ModifierOp) | column kind |  | A ModifierOp field: the variant's index. |
| [`og.Ability`](abilities/record-Ability.md) | component |  | Something its owner uses: on a press of `action` (og.input.consume, taken only when it can be used, so an early press waits in the buffer) or by og.abilities.use, it spends a charge and its cost and … |
| [`og.AbilityUsed`](abilities/record-AbilityUsed.md) | event |  | An og.Ability was used: sent the tick it was. |
| [`og.Stat`](abilities/record-Stat.md) | component |  | A number something has, on an entity of its own: `value` is `base` with every og.Modifier whose stat it is applied: adds, then percents, then multiplies, each kind in ascending order of what it adds,… |
| [`og.Modifier`](abilities/record-Modifier.md) | component |  | A change to an og.Stat, on an entity of its own: a child of an og.Status counts its amount once per stack and goes with it; one without counts once until it is despawned. |
| [`og.Status`](abilities/record-Status.md) | component |  | An effect on a target, its parent (og.Parent): spawned there to apply it. |
| [`og.AbilityEntities`](abilities/record-AbilityEntities.md) | resource |  | Every og.Ability, og.Stat, og.Status and og.Modifier in entity order, written by og.systems.abilities' first systems each tick for its last. |
| [`og.abilities.ready`](abilities/fn-ready.md) | function | `(world: World, ability: Entity): boolean` | Whether an ability can be used now: a charge, no lock, and its pool's value at least its cost. |
| [`og.abilities.use`](abilities/fn-use.md) | function | `(world: World, ability: Entity): Result<void>` | Use an ability now: a charge, its lock and its cost, then og.AbilityUsed. |

## Column kinds

Each is a factory, `StatusStacking(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-StatusStacking"></a>`StatusStacking` | `StatusStacking` | `Uint32Array` | 1 | A StatusStacking field: the variant's index. |
| <a id="column-ModifierOp"></a>`ModifierOp` | `ModifierOp` | `Uint32Array` | 1 | A ModifierOp field: the variant's index. |

```ogt
column StatusStacking {
  doc "A StatusStacking field: the variant's index."
  array Uint32Array
  stride 1
  value StatusStacking
}
```

<sub>Module `abilities` line 11</sub>

```ogt
column ModifierOp {
  doc "A ModifierOp field: the variant's index."
  array Uint32Array
  stride 1
  value ModifierOp
}
```

<sub>Module `abilities` line 25</sub>
