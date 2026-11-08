<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-countBadge"></a>

### `og.ui.countBadge`

A count badge: a small pill on a corner of the open container, outset by half its height so it straddles the edge, with a ring in the ground colour that cuts it from what it sits on. A count over max reads max+; a rise pops it (emphasis spring) and plays notification once. Hidden at 0.

Parts: CountBadge.base[<kind>][<tone>], CountBadge.text. States: group count (none, some, overflow). Accessibility: status named by name or the count. Probe ids: <id>, <id>/Text. Call it inside the container it decorates, after its content; the container should not clip.

```ts
og.ui.countBadge(id: string, desc?: CountBadgeDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `CountBadgeDesc` | yes | What it shows and where. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn countBadge {
  doc "A count badge: a small pill on a corner of the open container, outset by half its height so it straddles the edge, with a ring in the ground colour that cuts it from what it sits on. A count over max reads max+; a rise pops it (emphasis spring) and plays notification once. Hidden at 0."
  doc "Parts: CountBadge.base[<kind>][<tone>], CountBadge.text. States: group count (none, some, overflow). Accessibility: status named by name or the count. Probe ids: <id>, <id>/Text. Call it inside the container it decorates, after its content; the container should not clip."
  host false
  param id String "Its id."
  param desc CountBadgeDesc "What it shows and where."
  returns Void
}
```

<sub>Module `ui` line 5330</sub>
