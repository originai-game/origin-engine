<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-ColorMath"></a>

### `ColorMath` (interface)

Pure functions over Color (linear RGB), as three.js Color's methods with colour management on: hex and HSL are sRGB, converted to and from linear as three.js does. The value `color`: `color.fromHex(0xff8800)`. The same bits on every host (no Math.pow).

```ogt
interface ColorMath {
  doc "Pure functions over Color (linear RGB), as three.js Color's methods with colour management on: hex and HSL are sRGB, converted to and from linear as three.js does. The value `color`: `color.fromHex(0xff8800)`. The same bits on every host (no Math.pow)."
  # members follow, each quoted below
```

<sub>Module `math` line 578</sub>

<a id="member-ColorMath-fromHex"></a>

#### `ColorMath.fromHex`

The linear colour of an sRGB hex number, as new three.js Color(0xrrggbb).

```ts
ColorMath.fromHex(hex: number): Color
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `hex` | `number` | no | 0xrrggbb, sRGB. |

**Returns** `Color`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromHex {
  doc "The linear colour of an sRGB hex number, as new three.js Color(0xrrggbb)."
  host false
  param hex u32 "0xrrggbb, sRGB."
  returns Color
}
```

<sub>Module `math` line 580</sub>

<a id="member-ColorMath-toHex"></a>

#### `ColorMath.toHex`

The sRGB hex number of a linear colour, each channel clamped to 0..1 and rounded, as three.js Color.getHex.

```ts
ColorMath.toHex(c: Color): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `c` | `Color` | no | The colour. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn toHex {
  doc "The sRGB hex number of a linear colour, each channel clamped to 0..1 and rounded, as three.js Color.getHex."
  host false
  param c Color "The colour."
  returns u32
}
```

<sub>Module `math` line 586</sub>

<a id="member-ColorMath-lerp"></a>

#### `ColorMath.lerp`

a + (b - a) * t in linear RGB, as three.js Color.lerpColors.

```ts
ColorMath.lerp(a: Color, b: Color, t: number): Color
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Color` | no | The value at t = 0. |
| `b` | `Color` | no | The value at t = 1. |
| `t` | `number` | no | The fraction. |

**Returns** `Color`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn lerp {
  doc "a + (b - a) * t in linear RGB, as three.js Color.lerpColors."
  host false
  param a Color "The value at t = 0."
  param b Color "The value at t = 1."
  param t f32 "The fraction."
  returns Color
}
```

<sub>Module `math` line 592</sub>

<a id="member-ColorMath-fromHsl"></a>

#### `ColorMath.fromHsl`

The linear colour of sRGB hue, saturation and lightness, as three.js Color.setHSL (hue wraps, saturation and lightness clamp to 0..1).

```ts
ColorMath.fromHsl(h: number, s: number, l: number): Color
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `h` | `number` | no | The hue, 0 to 1 around the wheel (0 red, 1/3 green, 2/3 blue). |
| `s` | `number` | no | The saturation, 0 to 1. |
| `l` | `number` | no | The lightness, 0 to 1. |

**Returns** `Color`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromHsl {
  doc "The linear colour of sRGB hue, saturation and lightness, as three.js Color.setHSL (hue wraps, saturation and lightness clamp to 0..1)."
  host false
  param h f32 "The hue, 0 to 1 around the wheel (0 red, 1/3 green, 2/3 blue)."
  param s f32 "The saturation, 0 to 1."
  param l f32 "The lightness, 0 to 1."
  returns Color
}
```

<sub>Module `math` line 600</sub>
