<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `reflect` module

Reflection: components and resources are declared once with their field layout and metadata, and registered by name.

The inspector, Tune, the probe, scene text and the wiki read this one schema. Each column factory below makes one field.

A field holds one value, or a list of at most a fixed number of one number type or handle: `list(u32(), 8)`, `list(Asset(), 4)` (the list factory below). A list is read as a frozen copy (`push` throws) and written whole (`chain.passes = [...chain.passes, h]`); one over its capacity is refused, never truncated. A few vectors are numbered fields (`p0` to `p7`, as og.CameraRail does); a list of things with fields of their own is an entity each, with its own component, looped over by a query (a ring of skid marks, the laps of a race); a long record (a lap's taped inputs) is text in `str` fields of at most 4,096 bytes each, chunked over entities. Keep none of it in module variables: they are not world state, so a snapshot, a replay and a hot reload lose them.

A handle (an og.load answer, a texture, a sound, a geometry, a material, an entity) is a 64-bit number: keep it in a field of its own kind (`Asset()`, `Texture()`, `Sound()`, `Font()`, `Style()`, `Geometry()`, `Material()`, `Entity()`: every handle type of the API has its factory, named as the type), never cast to another kind, in a component or a resource. A `u32` field refuses one (handles pass 2^32: `4294967298 is not a u32`); an `f64` holds the number but loses its type.

- **TypeScript**: functions are named exports of the package: `import { … } from "@origingame/engine"`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-core`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module reflect {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 660</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`FieldOptions`](#type-FieldOptions) | desc struct | Metadata of one field. Ranges and units are what Tune and the probe's bounds show. Nothing clamps writes: game code's writes (world.set, a resource field, a column) are stored as given, and writes from outside the game (the probe's `set`, `patch` and sweeps, Tune, a look's fields at cook) are refused outside min..max with the range named. |
| [`f32()`](#column-f32) | column kind | A 32-bit float, the default numeric field. |
| [`f64()`](#column-f64) | column kind | A 64-bit float, for accumulated values that must not drift, such as elapsed time. |
| [`u32()`](#column-u32) | column kind | A 32-bit unsigned integer: counters, flags, indices. |
| [`i32()`](#column-i32) | column kind | A 32-bit signed integer. |
| [`bool()`](#column-bool) | column kind | A boolean, stored as one byte: 0 or 1. |
| [`Vector2()`](#column-Vector2) | column kind | A Vector2 field: x, y interleaved. |
| [`Vector3()`](#column-Vector3) | column kind | A Vector3 field: x, y, z interleaved. |
| [`Quaternion()`](#column-Quaternion) | column kind | A Quaternion field: x, y, z, w interleaved. |
| [`Matrix4()`](#column-Matrix4) | column kind | A Matrix4 field: sixteen column-major elements per entity. |
| [`Color()`](#column-Color) | column kind | A Color field: r, g, b interleaved. |
| [`str()`](#column-str) | column kind | Text of at most `capacity` bytes of UTF-8, stored inline and zero-padded, so it is world state like a number: a player's name, a label. `str(48)` holds any 12 characters (four bytes covers every script). The capacity is 1 to 4,096 bytes: a larger one is refused when the game starts (`a string field holds 1 to 4096 bytes`), so longer text goes in chunks, an entity each. |
| [`list()`](#column-list) | column kind | A list of at most `capacity` values of one number column (`f32`, `f64`, `u32`, `i32`) or handle column (`Asset`, `Entity`, `Post`, ...), stored inline: a count, then the elements, zeros past the count, so `[]` and `[0]` differ and the list is world state like a number. The capacity is 1 to 1,024 elements; it starts empty and is never tunable. |
| [`defineComponent`](#fn-defineComponent) | function | Declare a component: plain data stored per entity in engine memory, one column per field. |
| [`defineResource`](#fn-defineResource) | function | Declare a resource: a reflected singleton, such as a tuning table. Its fields start at their defaults. |

## Functions

<a id="fn-defineComponent"></a>

### `defineComponent`

Declare a component: plain data stored per entity in engine memory, one column per field.

Registration is by name, so a hot reload re-registers the same component and migrates its fields by name.

```ts
defineComponent<F extends Fields>(name: string, fields: F): ComponentType<F>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The component's unique name, PascalCase, as it appears in scenes and dumps. |
| `fields` | `F` | no | The layout: field names to column factories, as `{ hp: f32(100) }`. |

**Returns** `ComponentType<F>`: The token systems, queries and the world take.

Binding symbol `og_reflect_define_component` · publish builds: kept

```ogt
fn defineComponent {
  doc "Declare a component: plain data stored per entity in engine memory, one column per field."
  doc "Registration is by name, so a hot reload re-registers the same component and migrates its fields by name."
  generic F Fields
  param name String "The component's unique name, PascalCase, as it appears in scenes and dumps."
  param fields F "The layout: field names to column factories, as `{ hp: f32(100) }`."
  returns ComponentType.F "The token systems, queries and the world take."
}
```

<sub>`api.ogt` line 764</sub>

<a id="fn-defineResource"></a>

### `defineResource`

Declare a resource: a reflected singleton, such as a tuning table. Its fields start at their defaults.

```ts
defineResource<F extends Fields>(name: string, fields: F): ResourceType<F>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The resource's unique name, PascalCase. |
| `fields` | `F` | no | The layout, as for defineComponent. |

**Returns** `ResourceType<F>`: The token `world.resource` and system declarations take.

Binding symbol `og_reflect_define_resource` · publish builds: kept

```ogt
fn defineResource {
  doc "Declare a resource: a reflected singleton, such as a tuning table. Its fields start at their defaults."
  generic F Fields
  param name String "The resource's unique name, PascalCase."
  param fields F "The layout, as for defineComponent."
  returns ResourceType.F "The token `world.resource` and system declarations take."
}
```

<sub>`api.ogt` line 773</sub>

## Types

<a id="type-FieldOptions"></a>

### `FieldOptions` (desc struct)

Metadata of one field. Ranges and units are what Tune and the probe's bounds show. Nothing clamps writes: game code's writes (world.set, a resource field, a column) are stored as given, and writes from outside the game (the probe's `set`, `patch` and sweeps, Tune, a look's fields at cook) are refused outside min..max with the range named.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number \| null` | `null` | — | — | — | The smallest meaningful value. |
| `max` | `number \| null` | `null` | — | — | — | The largest meaningful value. |
| `step` | `number \| null` | `null` | — | — | — | The increment Tune uses. |
| `unit` | `string` | `""` | — | — | — | A unit name from the API's unit table, or empty. |
| `label` | `string` | `""` | — | — | — | The person-facing name; empty uses the field name. |
| `doc` | `string` | `""` | — | — | — | One sentence on what the field means. |
| `tunable` | `boolean` | `false` | — | — | — | Publish the field to Tune and to the probe's bounds and patch. |

```ogt
struct FieldOptions {
  doc "Metadata of one field. Ranges and units are what Tune and the probe's bounds show. Nothing clamps writes: game code's writes (world.set, a resource field, a column) are stored as given, and writes from outside the game (the probe's `set`, `patch` and sweeps, Tune, a look's fields at cook) are refused outside min..max with the range named."
  desc true
  field min Option.f32 "The smallest meaningful value."
  field max Option.f32 "The largest meaningful value."
  field step Option.f32 "The increment Tune uses."
  field unit String "A unit name from the API's unit table, or empty."
  field label String "The person-facing name; empty uses the field name."
  field doc String "One sentence on what the field means."
  field tunable bool "Publish the field to Tune and to the probe's bounds and patch."
}
```

<sub>`api.ogt` line 668</sub>

## Column kinds

Each is a factory, `f32(value?, options?)`, that makes one field of a component or resource layout; `str(capacity, value?, options?)` takes its capacity in bytes first; `list(of, capacity, options?)` takes another factory's field (`u32()`, `Asset()`) and a capacity in elements.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-f32"></a>`f32` | `number` | `Float32Array` | 1 | A 32-bit float, the default numeric field. |
| <a id="column-f64"></a>`f64` | `number` | `Float64Array` | 1 | A 64-bit float, for accumulated values that must not drift, such as elapsed time. |
| <a id="column-u32"></a>`u32` | `number` | `Uint32Array` | 1 | A 32-bit unsigned integer: counters, flags, indices. |
| <a id="column-i32"></a>`i32` | `number` | `Int32Array` | 1 | A 32-bit signed integer. |
| <a id="column-bool"></a>`bool` | `boolean` | `Uint8Array` | 1 | A boolean, stored as one byte: 0 or 1. |
| <a id="column-Vector2"></a>`Vector2` | `Vector2` | `Float32Array` | 2 | A Vector2 field: x, y interleaved. |
| <a id="column-Vector3"></a>`Vector3` | `Vector3` | `Float32Array` | 3 | A Vector3 field: x, y, z interleaved. |
| <a id="column-Quaternion"></a>`Quaternion` | `Quaternion` | `Float32Array` | 4 | A Quaternion field: x, y, z, w interleaved. |
| <a id="column-Matrix4"></a>`Matrix4` | `Matrix4` | `Float32Array` | 16 | A Matrix4 field: sixteen column-major elements per entity. |
| <a id="column-Color"></a>`Color` | `Color` | `Float32Array` | 3 | A Color field: r, g, b interleaved. |
| <a id="column-str"></a>`str` | `string` | `Uint8Array` | the capacity | Text of at most `capacity` bytes of UTF-8, stored inline and zero-padded, so it is world state like a number: a player's name, a label. `str(48)` holds any 12 characters (four bytes covers every script). The capacity is 1 to 4,096 bytes: a larger one is refused when the game starts (`a string field holds 1 to 4096 bytes`), so longer text goes in chunks, an entity each. Get and set it as a JavaScript string; text over the capacity is refused with the fix named (an error value from world.insert/world.set, a throw from a resource's setter), never truncated. Its view is the raw bytes. |
| <a id="column-list"></a>`list` | an array of the element's | the element's | the element's × (capacity + 1) | A list of at most `capacity` values of one number column (`f32`, `f64`, `u32`, `i32`) or handle column (`Asset`, `Entity`, `Post`, ...), stored inline: a count, then the elements, zeros past the count, so `[]` and `[0]` differ and the list is world state like a number. The capacity is 1 to 1,024 elements; it starts empty and is never tunable. Reading it answers a frozen array, a copy: `push`, `splice` and index writes throw, so change it by writing a whole array (`v.passes = [...v.passes, h]`, `v.passes = []`). A list over the capacity is refused with the fix named (an error value from world.insert/world.set, a throw from a resource's setter), never truncated. Scene text and looks give it on one line, its elements in order (`passes 3 5`), the key alone for the empty list. Its view is the element's typed array with a stride of (capacity + 1) elements, the count first. |

```ogt
column f32 {
  doc "A 32-bit float, the default numeric field."
  array Float32Array
  stride 1
  value f32
}
```

<sub>`api.ogt` line 680</sub>

```ogt
column f64 {
  doc "A 64-bit float, for accumulated values that must not drift, such as elapsed time."
  array Float64Array
  stride 1
  value f64
}
```

<sub>`api.ogt` line 687</sub>

```ogt
column u32 {
  doc "A 32-bit unsigned integer: counters, flags, indices."
  array Uint32Array
  stride 1
  value u32
}
```

<sub>`api.ogt` line 694</sub>

```ogt
column i32 {
  doc "A 32-bit signed integer."
  array Int32Array
  stride 1
  value i32
}
```

<sub>`api.ogt` line 701</sub>

```ogt
column bool {
  doc "A boolean, stored as one byte: 0 or 1."
  array Uint8Array
  stride 1
  value bool
}
```

<sub>`api.ogt` line 708</sub>

```ogt
column Vector2 {
  doc "A Vector2 field: x, y interleaved."
  array Float32Array
  stride 2
  value Vector2
}
```

<sub>`api.ogt` line 715</sub>

```ogt
column Vector3 {
  doc "A Vector3 field: x, y, z interleaved."
  array Float32Array
  stride 3
  value Vector3
}
```

<sub>`api.ogt` line 722</sub>

```ogt
column Quaternion {
  doc "A Quaternion field: x, y, z, w interleaved."
  array Float32Array
  stride 4
  value Quaternion
}
```

<sub>`api.ogt` line 729</sub>

```ogt
column Matrix4 {
  doc "A Matrix4 field: sixteen column-major elements per entity."
  array Float32Array
  stride 16
  value Matrix4
}
```

<sub>`api.ogt` line 736</sub>

```ogt
column Color {
  doc "A Color field: r, g, b interleaved."
  array Float32Array
  stride 3
  value Color
}
```

<sub>`api.ogt` line 743</sub>

```ogt
column str {
  doc "Text of at most `capacity` bytes of UTF-8, stored inline and zero-padded, so it is world state like a number: a player's name, a label. `str(48)` holds any 12 characters (four bytes covers every script). The capacity is 1 to 4,096 bytes: a larger one is refused when the game starts (`a string field holds 1 to 4096 bytes`), so longer text goes in chunks, an entity each."
  doc "Get and set it as a JavaScript string; text over the capacity is refused with the fix named (an error value from world.insert/world.set, a throw from a resource's setter), never truncated. Its view is the raw bytes."
  array Uint8Array
  capacity true
  value String
}
```

<sub>`api.ogt` line 750</sub>

```ogt
column list {
  doc "A list of at most `capacity` values of one number column (`f32`, `f64`, `u32`, `i32`) or handle column (`Asset`, `Entity`, `Post`, ...), stored inline: a count, then the elements, zeros past the count, so `[]` and `[0]` differ and the list is world state like a number. The capacity is 1 to 1,024 elements; it starts empty and is never tunable."
  doc "Reading it answers a frozen array, a copy: `push`, `splice` and index writes throw, so change it by writing a whole array (`v.passes = [...v.passes, h]`, `v.passes = []`). A list over the capacity is refused with the fix named (an error value from world.insert/world.set, a throw from a resource's setter), never truncated. Scene text and looks give it on one line, its elements in order (`passes 3 5`), the key alone for the empty list. Its view is the element's typed array with a stride of (capacity + 1) elements, the count first."
  list true
}
```

<sub>`api.ogt` line 758</sub>
