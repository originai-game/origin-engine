<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-ArchetypeView"></a>

### `ArchetypeView` (interface)

One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to.

```ogt
interface ArchetypeView {
  # members follow, each quoted below
```

<sub>Module `world` line 455</sub>

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

<sub>Module `world` line 458</sub>

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

<sub>Module `world` line 459</sub>

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

<sub>Module `world` line 460</sub>

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

<sub>Module `world` line 461</sub>

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

<sub>Module `world` line 462</sub>

<a id="member-ArchetypeView-handle"></a>

#### `ArchetypeView.handle`

The handle a field of entity i holds (an Entity, a TableRow, an asset), typed as the field's value: what world.get answers for it, without reading the whole component. Its column holds two u32 words a row, not handles. Throws for a field that holds no single handle, as column does for a component the system did not declare.

Also known as: read handle column, table row from column, entity field from column

```ts
ArchetypeView.handle<C extends AnyComponent, K extends HandleField<C>>(component: C, field: K, index: number): Value<C>[K]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `component` | `C` | no | The component token. |
| `field` | `K` | no | The field, such as `row`. |
| `index` | `number` | no | The row, from 0 to count - 1. |

**Returns** `Value<C>[K]`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn handle {
  doc "The handle a field of entity i holds (an Entity, a TableRow, an asset), typed as the field's value: what world.get answers for it, without reading the whole component. Its column holds two u32 words a row, not handles. Throws for a field that holds no single handle, as column does for a component the system did not declare."
  aliases "read handle column, table row from column, entity field from column"
  host false
  generic C AnyComponent
  generic K HandleField.C
  param component C "The component token."
  param field K "The field, such as `row`."
  param index u32 "The row, from 0 to count - 1."
  returns FieldValue.C.K
}
```

<sub>Module `world` line 467</sub>

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

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn has {
  doc "Whether the archetype carries a component."
  host false
  generic C AnyComponent
  param component C "The component token."
  returns bool
}
```

<sub>Module `world` line 478</sub>

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

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn column {
  doc "The typed views of a component's fields: one typed array per field, `count * stride` long."
  doc "Writing requires the component among the system's writes; reading among its reads or writes."
  host false
  generic C AnyComponent
  param component C "The component token."
  returns View.C
}
```

<sub>Module `world` line 485</sub>

<a id="member-ArchetypeView-addScaled"></a>

#### `ArchetypeView.addScaled`

target += source * scale, row by row, in the engine: the integrate step (position += velocity * dt) as one call per archetype instead of a loop over its entities in JavaScript. It covers the rows the query's change filters pass (rows), every row without them.

Both fields are f32 or f32 vectors of the same width (Vector2, Vector3, Vector4). The target's component must be among the system's writes, the source's among its reads or writes. Each element is computed in double precision and rounded once, so the result is the same bits as `t[i] += s[i] * scale` over the columns in JavaScript.

```ts
ArchetypeView.addScaled<C extends AnyComponent, S extends AnyComponent>(target: C, targetField: string, source: S, sourceField: string, scale: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `target` | `C` | no | The component written. |
| `targetField` | `string` | no | The target's field, such as `position`. |
| `source` | `S` | no | The component read (it may be the target). |
| `sourceField` | `string` | no | The source's field, such as `v`. |
| `scale` | `number` | no | The factor, such as time.dt. |

**Returns** `Result<void>`: Refused, with nothing changed, for a field that is missing or not f32, fields of different widths, a component the system did not declare or the archetype lacks, or a stale view.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn addScaled {
  doc "target += source * scale, row by row, in the engine: the integrate step (position += velocity * dt) as one call per archetype instead of a loop over its entities in JavaScript. It covers the rows the query's change filters pass (rows), every row without them."
  doc "Both fields are f32 or f32 vectors of the same width (Vector2, Vector3, Vector4). The target's component must be among the system's writes, the source's among its reads or writes. Each element is computed in double precision and rounded once, so the result is the same bits as `t[i] += s[i] * scale` over the columns in JavaScript."
  host false
  generic C AnyComponent
  generic S AnyComponent
  param target C "The component written."
  param targetField String "The target's field, such as `position`."
  param source S "The component read (it may be the target)."
  param sourceField String "The source's field, such as `v`."
  param scale f64 "The factor, such as time.dt."
  returns Result.Void "Refused, with nothing changed, for a field that is missing or not f32, fields of different widths, a component the system did not declare or the archetype lacks, or a stale view."
}
```

<sub>Module `world` line 493</sub>
