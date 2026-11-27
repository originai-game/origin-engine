<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-graph"></a>

### `og.story.graph`

A loaded dialogue's nodes and where each may lead, read from the dialogue, not a conversation: a flowchart, a check that every ending is reachable, the segments to warm before a choice.

See `yarn §9`.

Also known as: flowchart, story graph, branching map, node graph, route map

```ts
og.story.graph(world: World, dialogue: string): Result<readonly GraphNode[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `dialogue` | `string` | no | The dialogue's name (StoryAsset.name). |

**Returns** `Result<readonly GraphNode[]>`: Refused for a dialogue that is not loaded.

Binding symbol `og_story_graph` · publish builds: kept

```ogt
fn graph {
  doc "A loaded dialogue's nodes and where each may lead, read from the dialogue, not a conversation: a flowchart, a check that every ending is reachable, the segments to warm before a choice."
  aliases "flowchart, story graph, branching map, node graph, route map"
  param world World "The world."
  param dialogue String "The dialogue's name (StoryAsset.name)."
  returns Result.List.GraphNode "Refused for a dialogue that is not loaded."
  guide "yarn §9"
}
```

<sub>Module `story` line 328</sub>
