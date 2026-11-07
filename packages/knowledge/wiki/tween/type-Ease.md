<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-Ease"></a>

### `Ease` (enum)

An easing curve from the start (0) to the end (1). In starts slow, out ends slow, inOut both. back overshoots, elastic rings, bounce bounces at the end.

A string: one of the variants below. The zero value is `"power1.out"`.

| Variant | Doc |
|---|---|
| `"power1.out"` | Gentle ease out: the default for most motion. |
| `"none"` | Constant speed: scrolling, timers, colour cycles. |
| `"power1.in"` | Gentle ease in. |
| `"power1.inOut"` | Gentle in and out. |
| `"power2.in"` | Ease in. |
| `"power2.out"` | Ease out: UI sliding in, things settling. |
| `"power2.inOut"` | In and out: camera moves, deliberate motion. |
| `"power3.in"` | Strong ease in. |
| `"power3.out"` | Strong ease out. |
| `"power3.inOut"` | Strong in and out. |
| `"expo.in"` | Exponential ease in: things falling away, warp-outs. |
| `"expo.out"` | Exponential ease out: snappy arrivals. |
| `"expo.inOut"` | Exponential in and out. |
| `"back.in"` | Pulls back first, then goes: anticipation. |
| `"back.out"` | Overshoots, then settles: pops, squash recovery, buttons. |
| `"back.inOut"` | Pulls back and overshoots. |
| `"elastic.in"` | Rings before leaving. |
| `"elastic.out"` | Rings after arriving: springy, cartoon impacts. |
| `"elastic.inOut"` | Rings at both ends. |
| `"bounce.in"` | Bounces before leaving. |
| `"bounce.out"` | Bounces at the end: drops, landings. |
| `"bounce.inOut"` | Bounces at both ends. |

```ogt
enum Ease {
  doc "An easing curve from the start (0) to the end (1). In starts slow, out ends slow, inOut both. back overshoots, elastic rings, bounce bounces at the end."
  variant power1.out "Gentle ease out: the default for most motion."
  variant none "Constant speed: scrolling, timers, colour cycles."
  variant power1.in "Gentle ease in."
  variant power1.inOut "Gentle in and out."
  variant power2.in "Ease in."
  variant power2.out "Ease out: UI sliding in, things settling."
  variant power2.inOut "In and out: camera moves, deliberate motion."
  variant power3.in "Strong ease in."
  variant power3.out "Strong ease out."
  variant power3.inOut "Strong in and out."
  variant expo.in "Exponential ease in: things falling away, warp-outs."
  variant expo.out "Exponential ease out: snappy arrivals."
  variant expo.inOut "Exponential in and out."
  variant back.in "Pulls back first, then goes: anticipation."
  variant back.out "Overshoots, then settles: pops, squash recovery, buttons."
  variant back.inOut "Pulls back and overshoots."
  variant elastic.in "Rings before leaving."
  variant elastic.out "Rings after arriving: springy, cartoon impacts."
  variant elastic.inOut "Rings at both ends."
  variant bounce.in "Bounces before leaving."
  variant bounce.out "Bounces at the end: drops, landings."
  variant bounce.inOut "Bounces at both ends."
}
```

<sub>Module `tween` line 15</sub>
