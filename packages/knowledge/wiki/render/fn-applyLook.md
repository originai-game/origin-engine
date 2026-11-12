<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-applyLook"></a>

### `og.render.applyLook`

Apply a look: every resource the look names is replaced whole (the fields it sets take its values, the rest their defaults; PostEffects.flash and PostEffects.shakeTrauma keep theirs), and the UI is configured with its style, keeping the rest of the last UiDesc. Resources it does not name are left as they are. Inside a system, the system must declare those resources among its writes, except the art contract's, which are left as they are when undeclared. A look that switches the stylisation starts TAA's history over.

See `looks §16`.

```ts
og.render.applyLook(world: World, look: Look): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose look changes. |
| `look` | `Look` | no | The look, from og.load.look. |

**Returns** `Result<void>`: Refused for a look not loaded in this world, one in a streaming group that is not resident (the reason names it), or a resource the system did not declare.

Binding symbol `og_render_apply_look` · publish builds: kept

```ogt
fn applyLook {
  doc "Apply a look: every resource the look names is replaced whole (the fields it sets take its values, the rest their defaults; PostEffects.flash and PostEffects.shakeTrauma keep theirs), and the UI is configured with its style, keeping the rest of the last UiDesc. Resources it does not name are left as they are. Inside a system, the system must declare those resources among its writes, except the art contract's, which are left as they are when undeclared. A look that switches the stylisation starts TAA's history over."
  guide "looks §16"
  param world World "The world whose look changes."
  param look Look "The look, from og.load.look."
  returns Result.Void "Refused for a look not loaded in this world, one in a streaming group that is not resident (the reason names it), or a resource the system did not declare."
}
```

<sub>Module `render` line 2956</sub>
