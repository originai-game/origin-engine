<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertSite"></a>

### `AssertSite` (struct)

One assertion in the report: by its kind, name and place.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `AssertKind` | `"always"` | — | — | — | Its kind. |
| `origin` | `AssertOrigin` | `"game"` | — | — | — | Who declared it. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `location` | `string` | `""` | — | — | — | Where it is called; for a default invariant, the record and field (`Health.hp`). |
| `ran` | `boolean` | `false` | — | — | — | Whether it was called in this report (a site the build found may never run). |
| `fired` | `boolean` | `false` | — | — | — | Whether a sometimes held, a reachable ran, or an each's bucket held. |
| `firstTick` | `number \| null` | `null` | — | — | — | The first tick it fired. |
| `failures` | `number` | `0` | — | — | — | How many calls violated it. |
| `buckets` | `readonly AssertBucket[]` | `[]` | — | — | optional | An each's buckets, in the order first seen. |
| `bucketsDropped` | `number` | `0` | — | — | optional | An each's buckets past the 1,024 kept. |

```ogt
struct AssertSite {
  doc "One assertion in the report: by its kind, name and place."
  field kind AssertKind "Its kind."
  field origin AssertOrigin "Who declared it."
  field name String "Its name."
  field location String "Where it is called; for a default invariant, the record and field (`Health.hp`)."
  field ran bool "Whether it was called in this report (a site the build found may never run)."
  field fired bool "Whether a sometimes held, a reachable ran, or an each's bucket held."
  field firstTick Option.u32 "The first tick it fired."
  field failures u32 "How many calls violated it."
  field buckets List.AssertBucket "An each's buckets, in the order first seen." {
    optional true
  }
  field bucketsDropped u32 "An each's buckets past the 1,024 kept." {
    optional true
  }
}
```

<sub>Module `assert` line 42</sub>
