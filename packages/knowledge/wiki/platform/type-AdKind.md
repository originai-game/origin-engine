<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-AdKind"></a>

### `AdKind` (enum)

Which kind of ad.

A string: one of the variants below. The zero value is `"rewarded"`.

| Variant | Doc |
|---|---|
| `"rewarded"` | The player chose to watch it for a reward: the answer says whether they watched it to the end. |
| `"interstitial"` | Shown at a break the game picks (between levels): the player may close it. |

```ogt
enum AdKind {
  doc "Which kind of ad."
  variant rewarded "The player chose to watch it for a reward: the answer says whether they watched it to the end."
  variant interstitial "Shown at a break the game picks (between levels): the player may close it."
}
```

<sub>Module `platform` line 218</sub>
