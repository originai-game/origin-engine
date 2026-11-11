<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SpatialIndex"></a>

### `SpatialIndex` (interface)

A spatial hash of entities' positions as they were when world.spatial built it: it does not follow later moves (build it again after they move). Answers are ordered by distance, then by entity, so equal distances come out alike on every run and host. Valid for the system call or the call outside systems it was built in.

```ogt
interface SpatialIndex {
  doc "A spatial hash of entities' positions as they were when world.spatial built it: it does not follow later moves (build it again after they move). Answers are ordered by distance, then by entity, so equal distances come out alike on every run and host. Valid for the system call or the call outside systems it was built in."
  # members follow, each quoted below
```

<sub>Module `world` line 225</sub>

<a id="member-SpatialIndex-count"></a>

#### `SpatialIndex.count` (property)

The entities it holds.

```ts
readonly count: number
```

Binding symbol `og_spatial_index_get_count` · publish builds: kept

```ogt
prop count u32 "The entities it holds."
```

<sub>Module `world` line 227</sub>

<a id="member-SpatialIndex-within"></a>

#### `SpatialIndex.within`

Every entity at most `radius` from `center`, nearest first.

Also known as: radius query, nearby entities, within distance, range query, Unity Physics.OverlapSphere, neighbours

```ts
SpatialIndex.within(center: Vector3, radius: number): readonly Neighbour[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `center` | `Vector3` | no | The centre. |
| `radius` | `number` | no | The radius in metres; a negative one finds nothing. |

**Returns** `readonly Neighbour[]`.

Binding symbol `og_spatial_index_within` · publish builds: kept

```ogt
fn within {
  doc "Every entity at most `radius` from `center`, nearest first."
  aliases "radius query, nearby entities, within distance, range query, Unity Physics.OverlapSphere, neighbours"
  param center Vector3 "The centre."
  param radius f32 "The radius in metres; a negative one finds nothing."
  returns List.Neighbour
}
```

<sub>Module `world` line 228</sub>

<a id="member-SpatialIndex-nearest"></a>

#### `SpatialIndex.nearest`

The `count` entities nearest `center`, nearest first; fewer when it holds fewer.

```ts
SpatialIndex.nearest(center: Vector3, count: number): readonly Neighbour[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `center` | `Vector3` | no | The centre. |
| `count` | `number` | no | How many. |

**Returns** `readonly Neighbour[]`.

Binding symbol `og_spatial_index_nearest` · publish builds: kept

```ogt
fn nearest {
  doc "The `count` entities nearest `center`, nearest first; fewer when it holds fewer."
  param center Vector3 "The centre."
  param count u32 "How many."
  returns List.Neighbour
}
```

<sub>Module `world` line 235</sub>
