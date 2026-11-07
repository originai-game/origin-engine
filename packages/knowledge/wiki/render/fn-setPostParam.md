<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-setPostParam"></a>

### `og.render.setPostParam`

Set a parameter of a custom post pass, by the name its asset declares: a number for float, two to four for vec2, vec3, vec4 and color. Every frame it runs in reads it.

```ts
og.render.setPostParam(world: World, post: Post, name: string, value: readonly number[]): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the pass. |
| `post` | `Post` | no | The pass, from og.load.post. |
| `name` | `string` | no | The parameter's name. |
| `value` | `readonly number[]` | no | Its value; missing components are 0. |

**Returns** `Result<void>`: Refused for a pass not loaded in this world or a parameter it does not declare.

Binding symbol `og_render_set_post_param` · publish builds: kept

```ogt
fn setPostParam {
  doc "Set a parameter of a custom post pass, by the name its asset declares: a number for float, two to four for vec2, vec3, vec4 and color. Every frame it runs in reads it."
  param world World "The world whose renderer owns the pass."
  param post Post "The pass, from og.load.post."
  param name String "The parameter's name."
  param value List.f32 "Its value; missing components are 0."
  returns Result.Void "Refused for a pass not loaded in this world or a parameter it does not declare."
}
```

<sub>Module `render` line 3841</sub>
