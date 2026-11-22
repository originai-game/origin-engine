<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-ModelShape"></a>

### `ModelShape` (struct)

A cooked model's shape in its own space (og.Model's frame before its og.Transform): the axes a principal-axis fit of its triangles finds, its length and its top surface along that length. Place and turn things on a model from these, never from hand measurements (`physics §6.1`).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `forward` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Its long axis, a unit vector, signed towards +Z (else +X): a whale's or a ship's nose-to-tail line, either way round. |
| `up` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Of the other two axes the one nearest +Y, signed upward. |
| `centre` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The middle of its length on the area-weighted centre line: where `top` heights are measured from. |
| `length` | `number` | `0` | — | `m` | — | Its extent along forward. |
| `top` | `readonly number[]` | `[]` | — | — | — | Its top surface's height above `centre` along up at 16 evenly spaced stations along forward, from the back (forward's least) to the front, each at the middle of its sixteenth: where a line down the centre line first meets the surface, else the highest point of that slice. |

```ogt
struct ModelShape {
  doc "A cooked model's shape in its own space (og.Model's frame before its og.Transform): the axes a principal-axis fit of its triangles finds, its length and its top surface along that length. Place and turn things on a model from these, never from hand measurements (`physics §6.1`)."
  field forward Vector3 "Its long axis, a unit vector, signed towards +Z (else +X): a whale's or a ship's nose-to-tail line, either way round."
  field up Vector3 "Of the other two axes the one nearest +Y, signed upward."
  field centre Vector3 "The middle of its length on the area-weighted centre line: where `top` heights are measured from."
  field length f32 "Its extent along forward." {
    unit m
  }
  field top List.f32 "Its top surface's height above `centre` along up at 16 evenly spaced stations along forward, from the back (forward's least) to the front, each at the middle of its sixteenth: where a line down the centre line first meets the surface, else the highest point of that slice."
}
```

<sub>Module `load` line 76</sub>
