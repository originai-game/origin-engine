<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="record-PlatformGrants"></a>

### `og.PlatformGrants` (resource)

The purchases this world granted (og.platform.grant), so a consumable is granted once however its answer or notice comes back: a replay, a relaunch, a restored save. World state: saved, restored and hashed. Registered when a game first names it or calls og.platform.grant. Write it only through og.platform.grant.

Token `og.PlatformGrants`, registered under the schema name `og.PlatformGrants`; its value type is `PlatformGrants`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ids` | `readonly number[]` | `[]` | — | — | a list of at most 512 `u32` (`List.u32.512`): reads a frozen copy, writes a whole array | The 64-bit hashes of the granted purchase ids, each as two u32 (low, high), oldest first: the last 256 grants. |

```ogt
resource PlatformGrants {
  doc "The purchases this world granted (og.platform.grant), so a consumable is granted once however its answer or notice comes back: a replay, a relaunch, a restored save. World state: saved, restored and hashed. Registered when a game first names it or calls og.platform.grant. Write it only through og.platform.grant."
  field ids List.u32.512 "The 64-bit hashes of the granted purchase ids, each as two u32 (low, high), oldest first: the last 256 grants."
}
```

<sub>Module `platform` line 240</sub>
