<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `debug` module

Debug draw: lines, boxes, spheres, arrows, axes and labels in the world, each for a number of ticks, drawn over the finished frame (after every post effect, before the UI) so they read in every capture and never hide behind geometry; and the debug views (physics, navigation, bounds, levels of detail, and the lit image's terms: shadow cascades, ambient occlusion, albedo, image-based and direct light) the command channel's `debug:` actions switch.

Development only: not in publish builds, where every member does nothing and answers its default, so a game may leave its debug draws in. Debug draw is rendering: it never feeds back into the simulation, and its draws are not world state (a snapshot or a replay does not carry them).

- **TypeScript**: functions are the namespace `og.debug`; components, resources and events are tokens on `og`.
- **Publish builds**: dropped (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module debug {
  doc "Debug draw: lines, boxes, spheres, arrows, axes and labels in the world, each for a number of ticks, drawn over the finished frame (after every post effect, before the UI) so they read in every capture and never hide behind geometry; and the debug views (physics, navigation, bounds, levels of detail, and the lit image's terms: shadow cascades, ambient occlusion, albedo, image-based and direct light) the command channel's `debug:` actions switch."
  doc "Development only: not in publish builds, where every member does nothing and answers its default, so a game may leave its debug draws in. Debug draw is rendering: it never feeds back into the simulation, and its draws are not world state (a snapshot or a replay does not carry them)."
  publish false
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 25715</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`DebugDrawDesc`](debug/type-DebugDrawDesc.md) | desc struct |  | How a debug shape looks and how long it lasts. |
| [`og.debug.line`](debug/fn-line.md) | function | `(world: World, from: Vector3, to: Vector3, desc?: DebugDrawDesc): void` | A line from one point to another. |
| [`og.debug.box`](debug/fn-box.md) | function | `(world: World, center: Vector3, size: Vector3, desc?: DebugDrawDesc): void` | A box's twelve edges, as three.js Box3Helper (turned by desc.rotation). |
| [`og.debug.sphere`](debug/fn-sphere.md) | function | `(world: World, center: Vector3, radius: number, desc?: DebugDrawDesc): void` | A sphere as its three great circles. |
| [`og.debug.arrow`](debug/fn-arrow.md) | function | `(world: World, from: Vector3, to: Vector3, desc?: DebugDrawDesc): void` | An arrow from one point to another, its head a fifth of its length (three.js ArrowHelper): a velocity, a force, a direction to a target. |
| [`og.debug.axes`](debug/fn-axes.md) | function | `(world: World, position: Vector3, size: number, desc?: DebugDrawDesc): void` | A frame's three axes, as three.js AxesHelper: +x red, +y green, +z blue (desc.color is not read). |
| [`og.debug.text`](debug/fn-text.md) | function | `(world: World, position: Vector3, text: string, desc?: DebugDrawDesc): void` | A label anchored at a world point, drawn in screen space at a constant size in the UI's body font (a world with no UI font draws none), centred above the point: an entity's state, a counter, a name. |
| [`og.debug.clear`](debug/fn-clear.md) | function | `(world: World): void` | Remove every debug draw that is still showing. |
| [`og.debug.setView`](debug/fn-setView.md) | function | `(world: World, view: DebugView): void` | Show a debug view besides og.debug's draws (the command channel's `debug:<view>`); none shows the draws alone. |
| [`og.debug.view`](debug/fn-view.md) | function | `(world: World): DebugView` | The debug view shown. |
| [`og.debug.setEnabled`](debug/fn-setEnabled.md) | function | `(world: World, enabled: boolean): void` | Show or hide every debug draw and view (`debug:on`, `debug:off`); on by default outside publish builds. |
| [`og.debug.enabled`](debug/fn-enabled.md) | function | `(world: World): boolean` | Whether debug draws and views show. |
