<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-Query"></a>

### `Query` (interface)

A set of matched archetypes. Iterate archetypes, never entities one by one.

```ogt
interface Query {
  doc "A set of matched archetypes. Iterate archetypes, never entities one by one."
  # members follow, each quoted below
```

<sub>`api.ogt` line 1269</sub>

<a id="member-Query-archetypes"></a>

#### `Query.archetypes`

The matched archetypes this tick.

```ts
Query.archetypes(): readonly ArchetypeView[]
```

**Returns** `readonly ArchetypeView[]`.

Binding symbol `og_query_archetypes` · publish builds: kept

```ogt
fn archetypes {
  doc "The matched archetypes this tick."
  returns List.ArchetypeView
}
```

<sub>`api.ogt` line 1271</sub>

<a id="member-Query-count"></a>

#### `Query.count`

The number of matched entities.

```ts
Query.count(): number
```

**Returns** `number`.

Binding symbol `og_query_count` · publish builds: kept

```ogt
fn count {
  doc "The number of matched entities."
  returns u32
}
```

<sub>`api.ogt` line 1275</sub>
