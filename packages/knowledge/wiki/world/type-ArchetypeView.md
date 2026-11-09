<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-ArchetypeView"></a>

### `ArchetypeView` (interface)

One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to.

```ogt
interface ArchetypeView {
  doc "One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to."
  # members follow, each quoted below
```

<sub>Module `world` line 427</sub>

<a id="member-ArchetypeView-count"></a>

#### `ArchetypeView.count` (property)

The number of entities in the archetype.

```ts
readonly count: number
```

Binding symbol `og_archetype_view_get_count` · publish builds: kept

```ogt
prop count u32 "The number of entities in the archetype."
```

<sub>Module `world` line 429</sub>

<a id="member-ArchetypeView-tick"></a>

#### `ArchetypeView.tick` (property)

The tick this view belongs to.

```ts
readonly tick: number
```

Binding symbol `og_archetype_view_get_tick` · publish builds: kept

```ogt
prop tick u32 "The tick this view belongs to."
```

<sub>Module `world` line 430</sub>

<a id="member-ArchetypeView-entities"></a>

#### `ArchetypeView.entities` (property)

The entities as (index, generation) pairs: entity i is at 2i and 2i + 1. These are not handles: for a handle use entity(i) (world.despawn, get, set, insert and remove refuse a bare index, naming the call).

```ts
readonly entities: Uint32Array
```

Binding symbol `og_archetype_view_get_entities` · publish builds: kept

```ogt
prop entities Uint32Array "The entities as (index, generation) pairs: entity i is at 2i and 2i + 1. These are not handles: for a handle use entity(i) (world.despawn, get, set, insert and remove refuse a bare index, naming the call)."
```

<sub>Module `world` line 431</sub>

<a id="member-ArchetypeView-rows"></a>

#### `ArchetypeView.rows` (property)

The rows the query's change filters pass (SystemDesc.changed, added, removed; QueryDesc's), in row order; every row, 0 to count - 1, for a query without them. Columns still hold every row: loop over rows and index the columns with them.

```ts
readonly rows: Uint32Array
```

Binding symbol `og_archetype_view_get_rows` · publish builds: kept

```ogt
prop rows Uint32Array "The rows the query's change filters pass (SystemDesc.changed, added, removed; QueryDesc's), in row order; every row, 0 to count - 1, for a query without them. Columns still hold every row: loop over rows and index the columns with them."
```

<sub>Module `world` line 432</sub>

<a id="member-ArchetypeView-entity"></a>

#### `ArchetypeView.entity`

The handle of entity i.

```ts
ArchetypeView.entity(index: number): Entity
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | The row, from 0 to count - 1. |

**Returns** `Entity`.

Binding symbol `og_archetype_view_entity` · publish builds: kept

```ogt
fn entity {
  doc "The handle of entity i."
  param index u32 "The row, from 0 to count - 1."
  returns Entity
}
```

<sub>Module `world` line 433</sub>

<a id="member-ArchetypeView-has"></a>

#### `ArchetypeView.has`

Whether the archetype carries a component.

```ts
ArchetypeView.has<C extends AnyComponent>(component: C): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `component` | `C` | no | The component token. |

**Returns** `boolean`.

Binding symbol `og_archetype_view_has` · publish builds: kept

```ogt
fn has {
  doc "Whether the archetype carries a component."
  generic C AnyComponent
  param component C "The component token."
  returns bool
}
```

<sub>Module `world` line 438</sub>

<a id="member-ArchetypeView-column"></a>

#### `ArchetypeView.column`

The typed views of a component's fields: one typed array per field, `count * stride` long.

Writing requires the component among the system's writes; reading among its reads or writes.

```ts
ArchetypeView.column<C extends AnyComponent>(component: C): View<C>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `component` | `C` | no | The component token. |

**Returns** `View<C>`.

Binding symbol `og_archetype_view_column` · publish builds: kept

```ogt
fn column {
  doc "The typed views of a component's fields: one typed array per field, `count * stride` long."
  doc "Writing requires the component among the system's writes; reading among its reads or writes."
  generic C AnyComponent
  param component C "The component token."
  returns View.C
}
```

<sub>Module `world` line 444</sub>
