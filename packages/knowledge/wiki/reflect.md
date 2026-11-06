<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `reflect` module

Reflection: components and resources are declared once with their field layout and metadata, and registered by name.

The inspector, Tune, the probe, scene text and the wiki read this one schema. Each column factory below makes one field.

A field holds one value, or a list of at most a fixed number of one number type or handle: `list(u32(), 8)`, `list(Asset(), 4)` (the list factory below). A list is read as a frozen copy (`push` throws) and written whole (`chain.passes = [...chain.passes, h]`); one over its capacity is refused, never truncated. A few vectors are numbered fields (`p0` to `p7`, as og.CameraRail does); a list of things with fields of their own is an entity each, with its own component, looped over by a query (a ring of skid marks, the laps of a race); a long record (a lap's taped inputs) is text in `str` fields of at most 4,096 bytes each, chunked over entities. Keep none of it in module variables: they are not world state, so a snapshot, a replay and a hot reload lose them.

A handle (an og.load answer, a texture, a sound, a geometry, a material, an entity) is a 64-bit number: keep it in a field of its own kind (`Asset()`, `Texture()`, `Sound()`, `Font()`, `Style()`, `Geometry()`, `Material()`, `Entity()`: every handle type of the API has its factory, named as the type), never cast to another kind, in a component or a resource. A `u32` field refuses one (handles pass 2^32: `4294967298 is not a u32`); an `f64` holds the number but loses its type.

- **TypeScript**: functions are named exports of the package: `import { … } from "@origingame/engine"`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module reflect {
  doc "Reflection: components and resources are declared once with their field layout and metadata, and registered by name."
  doc "The inspector, Tune, the probe, scene text and the wiki read this one schema. Each column factory below makes one field."
  doc "A field holds one value, or a list of at most a fixed number of one number type or handle: `list(u32(), 8)`, `list(Asset(), 4)` (the list factory below). A list is read as a frozen copy (`push` throws) and written whole (`chain.passes = [...chain.passes, h]`); one over its capacity is refused, never truncated. A few vectors are numbered fields (`p0` to `p7`, as og.CameraRail does); a list of things with fields of their own is an entity each, with its own component, looped over by a query (a ring of skid marks, the laps of a race); a long record (a lap's taped inputs) is text in `str` fields of at most 4,096 bytes each, chunked over entities. Keep none of it in module variables: they are not world state, so a snapshot, a replay and a hot reload lose them."
  doc "A handle (an og.load answer, a texture, a sound, a geometry, a material, an entity) is a 64-bit number: keep it in a field of its own kind (`Asset()`, `Texture()`, `Sound()`, `Font()`, `Style()`, `Geometry()`, `Material()`, `Entity()`: every handle type of the API has its factory, named as the type), never cast to another kind, in a component or a resource. A `u32` field refuses one (handles pass 2^32: `4294967298 is not a u32`); an `f64` holds the number but loses its type."
  place top
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 733</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`FieldOptions`](reflect/type-FieldOptions.md) | desc struct |  | Metadata of one field. |
| [`f32()`](#column-f32) | column kind |  | A 32-bit float, the default numeric field. |
| [`f64()`](#column-f64) | column kind |  | A 64-bit float, for accumulated values that must not drift, such as elapsed time. |
| [`u32()`](#column-u32) | column kind |  | A 32-bit unsigned integer: counters, flags, indices. |
| [`i32()`](#column-i32) | column kind |  | A 32-bit signed integer. |
| [`bool()`](#column-bool) | column kind |  | A boolean, stored as one byte: 0 or 1. |
| [`Vector2()`](#column-Vector2) | column kind |  | A Vector2 field: x, y interleaved. |
| [`Vector3()`](#column-Vector3) | column kind |  | A Vector3 field: x, y, z interleaved. |
| [`Quaternion()`](#column-Quaternion) | column kind |  | A Quaternion field: x, y, z, w interleaved. |
| [`Matrix4()`](#column-Matrix4) | column kind |  | A Matrix4 field: sixteen column-major elements per entity. |
| [`Color()`](#column-Color) | column kind |  | A Color field: r, g, b interleaved. |
| [`str()`](#column-str) | column kind |  | Text of at most `capacity` bytes of UTF-8, stored inline and zero-padded, so it is world state like a number: a player's name, a label. |
| [`list()`](#column-list) | column kind |  | A list of at most `capacity` values of one number column (`f32`, `f64`, `u32`, `i32`) or handle column (`Asset`, `Entity`, `Post`, ...), stored inline: a count, then the elements, zeros past the coun… |
| [`defineComponent`](reflect/fn-defineComponent.md) | function | `<F extends Fields>(name: string, fields: F): ComponentType<F>` | Declare a component: plain data stored per entity in engine memory, one column per field. |
| [`defineResource`](reflect/fn-defineResource.md) | function | `<F extends Fields>(name: string, fields: F): ResourceType<F>` | Declare a resource: a reflected singleton, such as a tuning table. |

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

<sub>`api.ogt` line 753</sub>

```ogt
column f64 {
  doc "A 64-bit float, for accumulated values that must not drift, such as elapsed time."
  array Float64Array
  stride 1
  value f64
}
```

<sub>`api.ogt` line 760</sub>

```ogt
column u32 {
  doc "A 32-bit unsigned integer: counters, flags, indices."
  array Uint32Array
  stride 1
  value u32
}
```

<sub>`api.ogt` line 767</sub>

```ogt
column i32 {
  doc "A 32-bit signed integer."
  array Int32Array
  stride 1
  value i32
}
```

<sub>`api.ogt` line 774</sub>

```ogt
column bool {
  doc "A boolean, stored as one byte: 0 or 1."
  array Uint8Array
  stride 1
  value bool
}
```

<sub>`api.ogt` line 781</sub>

```ogt
column Vector2 {
  doc "A Vector2 field: x, y interleaved."
  array Float32Array
  stride 2
  value Vector2
}
```

<sub>`api.ogt` line 788</sub>

```ogt
column Vector3 {
  doc "A Vector3 field: x, y, z interleaved."
  array Float32Array
  stride 3
  value Vector3
}
```

<sub>`api.ogt` line 795</sub>

```ogt
column Quaternion {
  doc "A Quaternion field: x, y, z, w interleaved."
  array Float32Array
  stride 4
  value Quaternion
}
```

<sub>`api.ogt` line 802</sub>

```ogt
column Matrix4 {
  doc "A Matrix4 field: sixteen column-major elements per entity."
  array Float32Array
  stride 16
  value Matrix4
}
```

<sub>`api.ogt` line 809</sub>

```ogt
column Color {
  doc "A Color field: r, g, b interleaved."
  array Float32Array
  stride 3
  value Color
}
```

<sub>`api.ogt` line 816</sub>

```ogt
column str {
  doc "Text of at most `capacity` bytes of UTF-8, stored inline and zero-padded, so it is world state like a number: a player's name, a label. `str(48)` holds any 12 characters (four bytes covers every script). The capacity is 1 to 4,096 bytes: a larger one is refused when the game starts (`a string field holds 1 to 4096 bytes`), so longer text goes in chunks, an entity each."
  doc "Get and set it as a JavaScript string; text over the capacity is refused with the fix named (an error value from world.insert/world.set, a throw from a resource's setter), never truncated. Its view is the raw bytes."
  array Uint8Array
  capacity true
  value String
}
```

<sub>`api.ogt` line 823</sub>

```ogt
column list {
  doc "A list of at most `capacity` values of one number column (`f32`, `f64`, `u32`, `i32`) or handle column (`Asset`, `Entity`, `Post`, ...), stored inline: a count, then the elements, zeros past the count, so `[]` and `[0]` differ and the list is world state like a number. The capacity is 1 to 1,024 elements; it starts empty and is never tunable."
  doc "Reading it answers a frozen array, a copy: `push`, `splice` and index writes throw, so change it by writing a whole array (`v.passes = [...v.passes, h]`, `v.passes = []`). A list over the capacity is refused with the fix named (an error value from world.insert/world.set, a throw from a resource's setter), never truncated. Scene text and looks give it on one line, its elements in order (`passes 3 5`), the key alone for the empty list. Its view is the element's typed array with a stride of (capacity + 1) elements, the count first."
  list true
}
```

<sub>`api.ogt` line 831</sub>
