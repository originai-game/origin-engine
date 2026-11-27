<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="record-NavFilter"></a>

### `og.NavFilter` (component)

Which navmesh areas an agent crosses and what a metre in each costs: beside an og.NavAgent, its plans weigh every polygon's length by its area's cost and never enter an excluded area. Areas are marked at cook time (`area` in a navmesh asset) or at run time (og.nav.setArea); 0 is ground. Without one every area costs 1.

Also known as: area cost, terrain cost, Unity NavMesh area mask, NavMeshQueryFilter, Recast dtQueryFilter, Godot navigation layers

Token `og.NavFilter`, registered under the schema name `og.NavFilter`; its value type is `NavFilter`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `areaCosts` | `readonly number[]` | `[]` | — | — | a list of at most 64 `f32` (`List.f32.64`): reads a frozen copy, writes a whole array | Per area id, the cost of a metre in it (past the list's length 1, and at least 0.01): `{ areaCosts: [1, 1, 1, 8] }` makes area 3 (water) eight times dearer, so a path wades only when going round is longer than that. |
| `excluded` | `readonly number[]` | `[]` | — | — | a list of at most 64 `u32` (`List.u32.64`): reads a frozen copy, writes a whole array | Area ids the agent never enters (0 to 63): `{ excluded: [4] }` keeps it out of area 4 (lava). |

```ogt
component NavFilter {
  doc "Which navmesh areas an agent crosses and what a metre in each costs: beside an og.NavAgent, its plans weigh every polygon's length by its area's cost and never enter an excluded area. Areas are marked at cook time (`area` in a navmesh asset) or at run time (og.nav.setArea); 0 is ground. Without one every area costs 1."
  aliases "area cost, terrain cost, Unity NavMesh area mask, NavMeshQueryFilter, Recast dtQueryFilter, Godot navigation layers"
  field areaCosts List.f32.64 "Per area id, the cost of a metre in it (past the list's length 1, and at least 0.01): `{ areaCosts: [1, 1, 1, 8] }` makes area 3 (water) eight times dearer, so a path wades only when going round is longer than that."
  field excluded List.u32.64 "Area ids the agent never enters (0 to 63): `{ excluded: [4] }` keeps it out of area 4 (lava)."
}
```

<sub>Module `nav` line 212</sub>
