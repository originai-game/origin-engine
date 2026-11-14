<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="record-Volume"></a>

### `og.Volume` (component)

The shape of a region of the world: a box along the entity's axes, or a sphere about it, with a fade past its surface. A region component on the same entity takes its extent from it: og.ReverbZone (where the room is heard) and og.StreamCell (the box its anchors approach). So every region is placed, turned, scaled and faded the same way, and one entity can be several regions of one shape. Alone it does nothing.

Also known as: zone shape, region bounds, the box or sphere a zone covers, area an effect applies in, size of a zone, soft edge of a zone, area of effect

Token `og.Volume`, registered under the schema name `og.Volume`; its value type is `Volume`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | `m` | — | The box's full size along the entity's axes, centred on it (times og.Transform.scale). |
| `radius` | `number` | `0` | ≥ 0 | `m` | tunable | A sphere of this radius about the entity instead of the box (times the largest of og.Transform.scale); 0 uses the box. |
| `fade` | `number` | `0` | ≥ 0 | `m` | tunable | How far past the surface the region fades out, linearly; 0 is a hard edge. |

```ogt
component Volume {
  aliases "zone shape, region bounds, the box or sphere a zone covers, area an effect applies in, size of a zone, soft edge of a zone, area of effect"
  doc "The shape of a region of the world: a box along the entity's axes, or a sphere about it, with a fade past its surface. A region component on the same entity takes its extent from it: og.ReverbZone (where the room is heard) and og.StreamCell (the box its anchors approach). So every region is placed, turned, scaled and faded the same way, and one entity can be several regions of one shape. Alone it does nothing."
  field size Vector3 "The box's full size along the entity's axes, centred on it (times og.Transform.scale)." {
    default 1 1 1
    unit m
  }
  field radius f32 "A sphere of this radius about the entity instead of the box (times the largest of og.Transform.scale); 0 uses the box." {
    min 0
    unit m
    tunable true
  }
  field fade f32 "How far past the surface the region fades out, linearly; 0 is a hard edge." {
    min 0
    unit m
    tunable true
  }
}
```

<sub>Module `transform` line 29</sub>
