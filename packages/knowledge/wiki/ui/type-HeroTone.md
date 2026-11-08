<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HeroTone"></a>

### `HeroTone` (enum)

A call to action's material.

A string: one of the variants below. The zero value is `"primary"`.

| Variant | Doc |
|---|---|
| `"primary"` | The style's primary, lit from the top (End turn, Done, Play). |
| `"premium"` | Gold: the purchase a screen leads with (Summon ×10). |
| `"secondary"` | A raised neutral beside a primary or premium one (Summon ×1). |
| `"danger"` | A destructive action, raised in the danger colour. |

```ogt
enum HeroTone {
  doc "A call to action's material."
  variant primary "The style's primary, lit from the top (End turn, Done, Play)."
  variant premium "Gold: the purchase a screen leads with (Summon ×10)."
  variant secondary "A raised neutral beside a primary or premium one (Summon ×1)."
  variant danger "A destructive action, raised in the danger colour."
}
```

<sub>Module `ui` line 4092</sub>
