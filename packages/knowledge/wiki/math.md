<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `math` module

Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module math {
  doc "Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host."
  place top
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 26</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Vector2`](#type-Vector2) | struct | A 2D vector, as three.js Vector2. |
| [`Vector3`](#type-Vector3) | struct | A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js. |
| [`Quaternion`](#type-Quaternion) | struct | A unit quaternion rotation, as three.js Quaternion. The zero value is the identity. |
| [`Matrix4`](#type-Matrix4) | struct | A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity. |
| [`Color`](#type-Color) | struct | A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR. |
| [`Ray`](#type-Ray) | struct | A ray, as three.js Ray. |

## Types

<a id="type-Vector2"></a>

### `Vector2` (struct)

A 2D vector, as three.js Vector2.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The x component. |
| `y` | `number` | `0` | — | — | — | The y component. |

```ogt
struct Vector2 {
  doc "A 2D vector, as three.js Vector2."
  field x f32 "The x component."
  field y f32 "The y component."
}
```

<sub>`api.ogt` line 30</sub>

<a id="type-Vector3"></a>

### `Vector3` (struct)

A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The x component. |
| `y` | `number` | `0` | — | — | — | The y component. |
| `z` | `number` | `0` | — | — | — | The z component. |

```ogt
struct Vector3 {
  doc "A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js."
  field x f32 "The x component."
  field y f32 "The y component."
  field z f32 "The z component."
}
```

<sub>`api.ogt` line 36</sub>

<a id="type-Quaternion"></a>

### `Quaternion` (struct)

A unit quaternion rotation, as three.js Quaternion. The zero value is the identity.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The x component. |
| `y` | `number` | `0` | — | — | — | The y component. |
| `z` | `number` | `0` | — | — | — | The z component. |
| `w` | `number` | `1` | — | — | — | The w (scalar) component. |

```ogt
struct Quaternion {
  doc "A unit quaternion rotation, as three.js Quaternion. The zero value is the identity."
  field x f32 "The x component."
  field y f32 "The y component."
  field z f32 "The z component."
  field w f32 "The w (scalar) component." {
    default 1
  }
}
```

<sub>`api.ogt` line 43</sub>

<a id="type-Matrix4"></a>

### `Matrix4` (struct)

A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `elements` | `[number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number]` | `[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]` | — | — | — | The elements, column-major, as three.js Matrix4.elements. |

```ogt
struct Matrix4 {
  doc "A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity."
  field elements Array.f32.16 "The elements, column-major, as three.js Matrix4.elements." {
    default 1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1
  }
}
```

<sub>`api.ogt` line 53</sub>

<a id="type-Color"></a>

### `Color` (struct)

A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `r` | `number` | `0` | — | — | — | The red component. |
| `g` | `number` | `0` | — | — | — | The green component. |
| `b` | `number` | `0` | — | — | — | The blue component. |

```ogt
struct Color {
  doc "A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR."
  field r f32 "The red component."
  field g f32 "The green component."
  field b f32 "The blue component."
}
```

<sub>`api.ogt` line 60</sub>

<a id="type-Ray"></a>

### `Ray` (struct)

A ray, as three.js Ray.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where the ray starts. |
| `direction` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The unit direction the ray points in. |

```ogt
struct Ray {
  doc "A ray, as three.js Ray."
  field origin Vector3 "Where the ray starts."
  field direction Vector3 "The unit direction the ray points in."
}
```

<sub>`api.ogt` line 67</sub>
