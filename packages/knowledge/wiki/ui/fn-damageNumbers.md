<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-damageNumbers"></a>

### `og.ui.damageNumbers`

Damage numbers to the bar of Diablo IV, Genshin Impact and World of Warcraft: each hit thrown from where it landed, rising and drifting to its side as it fades, outlined so it reads on any scene; a critical larger in gold, popping past its size and settling; heals in green with a +; damage over time smaller; misses as their word in grey; the player's own damage in danger; numbers compact past ten thousand. Mass elements: thousands at once, in the snapshot.

Parts: none per number (mass elements); the templates are named <id>.number and <id>.word. Accessibility: none (combat text is decoration; the game announces what matters). Probe: og.ui.massCount.

```ts
og.ui.damageNumbers(id: string, desc?: DamageNumbersDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id (its mass templates' prefix). |
| `desc` | `DamageNumbersDesc` | yes | This tick's hits. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn damageNumbers {
  doc "Damage numbers to the bar of Diablo IV, Genshin Impact and World of Warcraft: each hit thrown from where it landed, rising and drifting to its side as it fades, outlined so it reads on any scene; a critical larger in gold, popping past its size and settling; heals in green with a +; damage over time smaller; misses as their word in grey; the player's own damage in danger; numbers compact past ten thousand. Mass elements: thousands at once, in the snapshot."
  doc "Parts: none per number (mass elements); the templates are named <id>.number and <id>.word. Accessibility: none (combat text is decoration; the game announces what matters). Probe: og.ui.massCount."
  host false
  param id String "Its id (its mass templates' prefix)."
  param desc DamageNumbersDesc "This tick's hits."
  returns Void
}
```

<sub>Module `ui` line 6548</sub>
