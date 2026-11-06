<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-HookRun"></a>

### `HookRun` (callback)

A hook's body: called once per entity the component is added to or removed from, wherever the change came from (a system's structural changes as they apply when it returns, or a call between ticks), in the order hooks are listed in GameDesc.hooks. Its own structural changes apply when it returns and may run hooks in turn (16 deep at most). A restore runs none.

Game code supplies this function.

```ts
type HookRun = (world: World, entity: Entity) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world, as a system's. |
| `entity` | `Entity` | no | The entity. |

```ogt
callback HookRun {
  doc "A hook's body: called once per entity the component is added to or removed from, wherever the change came from (a system's structural changes as they apply when it returns, or a call between ticks), in the order hooks are listed in GameDesc.hooks. Its own structural changes apply when it returns and may run hooks in turn (16 deep at most). A restore runs none."
  param world World "The world, as a system's."
  param entity Entity "The entity."
}
```

<sub>`api.ogt` line 1370</sub>
