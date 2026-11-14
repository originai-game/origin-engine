<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-finishLink"></a>

### `og.nav.finishLink`

End a crossing the game moves (NavLinkDesc.game): the agent lands at the link's end as og.nav next runs and goes on along its path.

```ts
og.nav.finishLink(world: World, entity: Entity): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The agent. |

**Returns** `boolean`: False when it is not crossing such a link.

Binding symbol `og_nav_finish_link` · publish builds: kept

```ogt
fn finishLink {
  doc "End a crossing the game moves (NavLinkDesc.game): the agent lands at the link's end as og.nav next runs and goes on along its path."
  param world World "The world."
  param entity Entity "The agent."
  returns bool "False when it is not crossing such a link."
}
```

<sub>Module `nav` line 639</sub>
