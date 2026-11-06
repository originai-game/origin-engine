<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SaveEdit"></a>

### `SaveEdit` (interface)

A save opened for editing (world.openSave), as a migration sees it: the engine's own migrations already applied, every type with its saved fields. Each edit changes the save, not the world; world.restore takes the result. An edit refused changes nothing. Entity ids are the saved world's.

```ogt
interface SaveEdit {
  doc "A save opened for editing (world.openSave), as a migration sees it: the engine's own migrations already applied, every type with its saved fields. Each edit changes the save, not the world; world.restore takes the result. An edit refused changes nothing. Entity ids are the saved world's."
  # members follow, each quoted below
```

<sub>Module `world` line 104</sub>

<a id="member-SaveEdit-version"></a>

#### `SaveEdit.version` (property)

The game's save version the save was written at (1 for a save of a game that never declared one).

```ts
readonly version: number
```

Binding symbol `og_save_edit_get_version` · publish builds: kept

```ogt
prop version u32 "The game's save version the save was written at (1 for a save of a game that never declared one)."
```

<sub>Module `world` line 106</sub>

<a id="member-SaveEdit-types"></a>

#### `SaveEdit.types`

The types the save holds, in its table's order.

```ts
SaveEdit.types(): readonly SaveType[]
```

**Returns** `readonly SaveType[]`.

Binding symbol `og_save_edit_types` · publish builds: kept

```ogt
fn types {
  doc "The types the save holds, in its table's order."
  returns List.SaveType
}
```

<sub>Module `world` line 107</sub>

<a id="member-SaveEdit-has"></a>

#### `SaveEdit.has`

Whether the save holds a type.

```ts
SaveEdit.has(name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The type's name. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn has {
  doc "Whether the save holds a type."
  host false
  param name String "The type's name."
  returns bool
}
```

<sub>Module `world` line 111</sub>

<a id="member-SaveEdit-records"></a>

#### `SaveEdit.records`

A type's saved values: its components by entity in storage order, then its resource.

```ts
SaveEdit.records(name: string): Result<readonly SaveRecord[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The type's name. |

**Returns** `Result<readonly SaveRecord[]>`: Refused for a type the save does not hold.

Binding symbol `og_save_edit_records` · publish builds: kept

```ogt
fn records {
  doc "A type's saved values: its components by entity in storage order, then its resource."
  param name String "The type's name."
  returns Result.List.SaveRecord "Refused for a type the save does not hold."
}
```

<sub>Module `world` line 117</sub>

<a id="member-SaveEdit-setRecords"></a>

#### `SaveEdit.setRecords`

Replace a type's values (its components, its resource) with records written as this build's type `into`: the old type's values and queued events go, an entity keeps its other components and its place. A type the save does not hold is added (a component a new version gives saved entities).

```ts
SaveEdit.setRecords(name: string, desc?: SetRecordsDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type to replace. |
| `desc` | `SetRecordsDesc` | yes | The type to write and the records. |

**Returns** `Result<void>`: Refused, with nothing changed, for a type this build does not declare, a record that does not fit it, or an entity the save does not hold.

Binding symbol `og_save_edit_set_records` · publish builds: kept

```ogt
fn setRecords {
  doc "Replace a type's values (its components, its resource) with records written as this build's type `into`: the old type's values and queued events go, an entity keeps its other components and its place. A type the save does not hold is added (a component a new version gives saved entities)."
  param name String "The saved type to replace."
  param desc SetRecordsDesc "The type to write and the records."
  returns Result.Void "Refused, with nothing changed, for a type this build does not declare, a record that does not fit it, or an entity the save does not hold."
}
```

<sub>Module `world` line 122</sub>

<a id="member-SaveEdit-transform"></a>

#### `SaveEdit.transform`

Rewrite every value of a type through `map`, which sees the old record: records(name), then setRecords with what map returned. With `into`, the type becomes that one (a rename that reshapes).

```ts
SaveEdit.transform(name: string, map: RecordFn, desc?: TransformDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type. |
| `map` | `RecordFn` | no | The old record to the new one. |
| `desc` | `TransformDesc` | yes | The type the results are written as. |

**Returns** `Result<void>`: Refused as setRecords is, or when map throws.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn transform {
  doc "Rewrite every value of a type through `map`, which sees the old record: records(name), then setRecords with what map returned. With `into`, the type becomes that one (a rename that reshapes)."
  host false
  param name String "The saved type."
  param map RecordFn "The old record to the new one."
  param desc TransformDesc "The type the results are written as."
  returns Result.Void "Refused as setRecords is, or when map throws."
}
```

<sub>Module `world` line 128</sub>

<a id="member-SaveEdit-split"></a>

#### `SaveEdit.split`

Split a type into several: each saved value is handed to `map`, whose answer names the new types' records; the old type goes. The migrated world holds the records a world built at the new version holds, and every peer and every restore of the same save migrates to the same world and hash; its hash differs from a built one's only by the archetypes the built one passed through (inserting the new types one at a time leaves an empty archetype of the first alone, which the split never makes), so compare a split's result record by record, not by hash. renameField, renameType, transform, dropType and spawn keep which types sit together: those migrate to a built world's hash exactly.

```ts
SaveEdit.split(name: string, map: SplitFn): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type. |
| `map` | `SplitFn` | no | The old record to the new types' records. |

**Returns** `Result<void>`: Refused as setRecords is, or when map throws.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn split {
  doc "Split a type into several: each saved value is handed to `map`, whose answer names the new types' records; the old type goes. The migrated world holds the records a world built at the new version holds, and every peer and every restore of the same save migrates to the same world and hash; its hash differs from a built one's only by the archetypes the built one passed through (inserting the new types one at a time leaves an empty archetype of the first alone, which the split never makes), so compare a split's result record by record, not by hash. renameField, renameType, transform, dropType and spawn keep which types sit together: those migrate to a built world's hash exactly."
  host false
  param name String "The saved type."
  param map SplitFn "The old record to the new types' records."
  returns Result.Void "Refused as setRecords is, or when map throws."
}
```

<sub>Module `world` line 136</sub>

<a id="member-SaveEdit-renameType"></a>

#### `SaveEdit.renameType`

Give a saved type another name, its fields and values as they are (a later restore maps them field by field against the type of the new name).

```ts
SaveEdit.renameType(from: string, to: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `from` | `string` | no | The saved name. |
| `to` | `string` | no | The new name. |

**Returns** `Result<void>`: Refused for a name the save lacks, or one it already has.

Binding symbol `og_save_edit_rename_type` · publish builds: kept

```ogt
fn renameType {
  doc "Give a saved type another name, its fields and values as they are (a later restore maps them field by field against the type of the new name)."
  param from String "The saved name."
  param to String "The new name."
  returns Result.Void "Refused for a name the save lacks, or one it already has."
}
```

<sub>Module `world` line 143</sub>

<a id="member-SaveEdit-renameField"></a>

#### `SaveEdit.renameField`

Give a saved type's field another name, its values as they are: `gold` saved, `coins` declared now, and every balance arrives.

```ts
SaveEdit.renameField(name: string, from: string, to: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type. |
| `from` | `string` | no | The saved field. |
| `to` | `string` | no | The new name. |

**Returns** `Result<void>`: Refused for a type or field the save lacks, or a name the type already has.

Binding symbol `og_save_edit_rename_field` · publish builds: kept

```ogt
fn renameField {
  doc "Give a saved type's field another name, its values as they are: `gold` saved, `coins` declared now, and every balance arrives."
  param name String "The saved type."
  param from String "The saved field."
  param to String "The new name."
  returns Result.Void "Refused for a type or field the save lacks, or a name the type already has."
}
```

<sub>Module `world` line 149</sub>

<a id="member-SaveEdit-dropType"></a>

#### `SaveEdit.dropType`

Remove a type from the save: its components, resource and queued events.

```ts
SaveEdit.dropType(name: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type. |

**Returns** `Result<void>`: Refused for a type the save does not hold.

Binding symbol `og_save_edit_drop_type` · publish builds: kept

```ogt
fn dropType {
  doc "Remove a type from the save: its components, resource and queued events."
  param name String "The saved type."
  returns Result.Void "Refused for a type the save does not hold."
}
```

<sub>Module `world` line 156</sub>

<a id="member-SaveEdit-spawn"></a>

#### `SaveEdit.spawn`

Add an entity to the save, with components of this build's types: a whole save gives it the id the next spawn would take, a selection the next free one.

```ts
SaveEdit.spawn(desc?: SaveSpawnDesc): Result<Entity>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `SaveSpawnDesc` | yes | Its name and components. |

**Returns** `Result<Entity>`: Refused for a taken name, a type this build does not declare or a value that does not fit it.

Binding symbol `og_save_edit_spawn` · publish builds: kept

```ogt
fn spawn {
  doc "Add an entity to the save, with components of this build's types: a whole save gives it the id the next spawn would take, a selection the next free one."
  param desc SaveSpawnDesc "Its name and components."
  returns Result.Entity "Refused for a taken name, a type this build does not declare or a value that does not fit it."
}
```

<sub>Module `world` line 161</sub>

<a id="member-SaveEdit-text"></a>

#### `SaveEdit.text`

The save as text, at GameDesc.saveVersion: what world.restore takes.

```ts
SaveEdit.text(): Result<string>
```

**Returns** `Result<string>`.

Binding symbol `og_save_edit_text` · publish builds: kept

```ogt
fn text {
  doc "The save as text, at GameDesc.saveVersion: what world.restore takes."
  returns Result.String
}
```

<sub>Module `world` line 166</sub>

<a id="member-SaveEdit-close"></a>

#### `SaveEdit.close`

Let go of the open save. world.restore closes the ones it opens.

```ts
SaveEdit.close(): void
```

**Returns** `void`.

Binding symbol `og_save_edit_close` · publish builds: kept

```ogt
fn close {
  doc "Let go of the open save. world.restore closes the ones it opens."
  returns Void
}
```

<sub>Module `world` line 170</sub>
