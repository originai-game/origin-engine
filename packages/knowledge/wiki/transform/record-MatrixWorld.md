<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="record-MatrixWorld"></a>

### `og.MatrixWorld` (component)

The world matrix of an entity attached to a bone (og.BoneAttachment), written after og.anim poses it, as three.js Object3D.matrixWorld. Other entities have none: read any entity's world matrix with og.transform.matrixWorld.

Token `og.MatrixWorld`, registered under the schema name `og.MatrixWorld`; its value type is `MatrixWorld`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `matrix` | `Matrix4` | `{ elements: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1] }` | — | — | read only | The composed world matrix. |

```ogt
component MatrixWorld {
  doc "The world matrix of an entity attached to a bone (og.BoneAttachment), written after og.anim poses it, as three.js Object3D.matrixWorld. Other entities have none: read any entity's world matrix with og.transform.matrixWorld."
  field matrix Matrix4 "The composed world matrix." {
    readonly true
  }
}
```

<sub>Module `transform` line 22</sub>
