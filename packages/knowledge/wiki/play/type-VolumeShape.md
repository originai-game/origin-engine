<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`play`](../play.md) · [index](../index.md)

<a id="type-VolumeShape"></a>

### `VolumeShape` (enum)

A hit or hurt volume's shape: og.HitVolume.shape and og.HurtVolume.shape.

A string: one of the variants below. The zero value is `"sphere"`.

| Variant | Doc |
|---|---|
| `"sphere"` | A sphere of `radius` (a circle in 2D). |
| `"capsule"` | A capsule along its local y of `radius`, with `height` between the cap centres (three.js CapsuleGeometry). |
| `"box"` | A box of `size`, its full extents (three.js BoxGeometry). |

```ogt
enum VolumeShape {
  doc "A hit or hurt volume's shape: og.HitVolume.shape and og.HurtVolume.shape."
  variant sphere "A sphere of `radius` (a circle in 2D)."
  variant capsule "A capsule along its local y of `radius`, with `height` between the cap centres (three.js CapsuleGeometry)."
  variant box "A box of `size`, its full extents (three.js BoxGeometry)."
}
```

<sub>Module `play` line 4</sub>
