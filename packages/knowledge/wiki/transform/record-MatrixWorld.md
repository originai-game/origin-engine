<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="record-MatrixWorld"></a>

### `og.MatrixWorld` (component)

The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld.

Token `og.MatrixWorld`, registered under the schema name `og.MatrixWorld`; its value type is `MatrixWorld`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `matrix` | `Matrix4` | `{ elements: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1] }` | — | — | read only | The composed world matrix. |

```ogt
component MatrixWorld {
  doc "The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld."
  field matrix Matrix4 "The composed world matrix." {
    readonly true
  }
}
```

<sub>Module `transform` line 21</sub>
