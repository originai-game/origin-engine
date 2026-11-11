<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="type-Explosion2dDesc"></a>

### `Explosion2dDesc` (desc struct)

A blast for og.destruction.explode2d.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Its centre, in world space (its xy). |
| `radius` | `number` | `1.5` | ≥ 0 | `m` | — | The hole it blows in every og.PixelTerrain in reach. |
| `debris` | `number` | `0.4` | ≥ 0 | `m` | — | The width of the ring of ground round the hole that it breaks into pieces and throws (0 none): the ground there is cut into Voronoi pieces of about pieceCells cells, each an og.PixelPiece. |
| `pieceCells` | `number` | `24` | ≥ 4 | — | — | About how many cells each thrown piece has. |
| `speed` | `number` | `8` | ≥ 0 | `mps` | — | How fast it throws the pieces it cuts and the islands it cuts loose outwards, at its centre, falling linearly to nothing at radius plus debris; the og.PixelPiece entities already in reach take the same push on their og.Velocity. |

```ogt
struct Explosion2dDesc {
  doc "A blast for og.destruction.explode2d."
  desc true
  field point Vector3 "Its centre, in world space (its xy)." {
    unit m
  }
  field radius f32 "The hole it blows in every og.PixelTerrain in reach." {
    default 1.5
    min 0
    unit m
  }
  field debris f32 "The width of the ring of ground round the hole that it breaks into pieces and throws (0 none): the ground there is cut into Voronoi pieces of about pieceCells cells, each an og.PixelPiece." {
    default 0.4
    min 0
    unit m
  }
  field pieceCells u32 "About how many cells each thrown piece has." {
    default 24
    min 4
  }
  field speed f32 "How fast it throws the pieces it cuts and the islands it cuts loose outwards, at its centre, falling linearly to nothing at radius plus debris; the og.PixelPiece entities already in reach take the same push on their og.Velocity." {
    default 8
    min 0
    unit mps
  }
}
```

<sub>Module `destruction` line 446</sub>
