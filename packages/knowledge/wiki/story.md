<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `story` module

Dialogue and quests as data: dialogues in `.ogt` files of `type dialogue` (speakers, nodes of lines, choices with conditions, reasons and checks, variables, weighted picks, timers) and quests in `type quest` files (stages of objectives, rewards, conditions), validated by og cook, which names each mistake's line and the fix. Text is never inline: every line, choice, reason, name and title is an og.i18n key, and og.story resolves it in the active locale when it is read, so a locale switch re-translates the line on screen. og-story §1 has every line of both formats.

og.story.load a dialogue or quest in setup, og.story.start a conversation, read og.story.view in your UI each frame (the line in full, its speaker, its choices; the dialogue box paces its own reveal), and call og.story.advance or og.story.choose. Quests start from a dialogue (`quest start rescue`) or og.story.startQuest, count with og.story.progress and advance their stages themselves; og.story.quests lists them for a quest log.

Barks are ambient lines an entity says on its own (a dialogue's `bark` blocks: a topic, lines, a `when` that may read the world, priority, weight, cooldowns per entity and shared, once): og.story.bark asks for one, og.story.ambient every so often, og.story.barks lists those being said. A line's `voice` is played by og.story through og.audio, and DialogueView.voiceLeft says when it ends. A quest objective with `on <Event>` counts each such event the game or the engine sends, filtered by `where` (which reads `event.<field>`), with no game code. `dump_story` writes the whole story: variables, conversations, quests, barks, cooldowns and voices.

Everything is world state: the story's variables, the nodes seen, the lines read, conversations and quests are in saves, snapshots and the world hash, saved by their ids (dialogue, node, line, quest, stage, objective), so a save survives any edit that keeps those ids. Renaming a node, stage or objective a save stands on is the one breaking edit: og.story warns, naming the id, and falls back (a conversation to its dialogue's start, a quest to its first stage) instead of landing elsewhere. Deterministic: `pick` and active checks draw from the story's own stream (og.Rng's fork "story"), timers count ticks, and arithmetic is plain f64. A world that never loads a dialogue or quest keeps its saves and hash: og.Story and the story's events join it at the first og.story.load.

- **TypeScript**: functions are the namespace `og.story`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-story`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module story {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 13965</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Story`](#record-Story) | resource | The story's counters (written by og.story; the story itself is a state block). Registered at the first og.story.load. |
| [`og.DialogueShown`](#record-DialogueShown) | event | A conversation shows a new line (start the voice, reset the reveal). Sent before the game's systems, the tick after the call that moved it. |
| [`og.DialogueEnded`](#record-DialogueEnded) | event | A conversation ended: its dialogue ran out, reached `end`, took a choice without `goto`, or og.story.stop stopped it. Sent the tick after. |
| [`og.DialogueCheck`](#record-DialogueCheck) | event | An active check was rolled (d20 plus the stat against the target): reveal the roll. Sent the tick after. |
| [`og.QuestUpdated`](#record-QuestUpdated) | event | A quest started, counted, changed stage, completed or failed. Sent the tick after. |
| [`og.BarkSaid`](#record-BarkSaid) | event | An entity barked (og.story.bark or og.story.ambient): show its line over it with og.story.barks. Sent the tick after it was asked for. |
| [`og.StoryEvent`](#record-StoryEvent) | event | A dialogue's `emit name "arg"`: open the shop, give the sword, play the cutscene. Sent the tick after. |
| [`StoryAsset`](#type-StoryAsset) | struct | A loaded dialogue or quest. |
| [`StartDesc`](#type-StartDesc) | desc struct | A conversation to start. |
| [`SpeakerView`](#type-SpeakerView) | struct | Who speaks a line. |
| [`CheckView`](#type-CheckView) | struct | A choice's check. |
| [`ChoiceView`](#type-ChoiceView) | struct | A choice the node offers (hidden ones are left out). |
| [`TimerView`](#type-TimerView) | struct | A node's choice timer. |
| [`RollView`](#type-RollView) | struct | An active check's roll. |
| [`DialogueView`](#type-DialogueView) | struct | What a conversation shows now: the line in full (the dialogue box paces its own reveal), its speaker and the choices, if it waits on them. |
| [`ChosenView`](#type-ChosenView) | struct | The choice made while a line was shown. |
| [`StoryLine`](#type-StoryLine) | struct | A line of a conversation's backlog. |
| [`BarkDesc`](#type-BarkDesc) | desc struct | A bark to ask for. |
| [`AmbientDesc`](#type-AmbientDesc) | desc struct | An entity that barks on its own. |
| [`BarkView`](#type-BarkView) | struct | A bark being said. |
| [`MarkerDesc`](#type-MarkerDesc) | desc struct | Where a quest's objective is, for the map, compass and tracker. |
| [`MarkerView`](#type-MarkerView) | struct | Where a quest points: the marker og.story.mark set, else the current stage's first undone objective's `marker`. |
| [`RewardView`](#type-RewardView) | struct | A quest's reward. |
| [`ObjectiveView`](#type-ObjectiveView) | struct | An objective of a quest's current stage. |
| [`QuestView`](#type-QuestView) | struct | A quest as a quest log shows it. |
| [`og.story.load`](#fn-load) | function | A cooked dialogue or quest (an `.ogt` of `type dialogue` or `type quest`, validated by og cook). Loading again replaces it, so a live edit takes effect at the conversation's next step. The first load registers og.Story and the story's events, so call it in setup: inside a system it is refused until one load has run outside one. |
| [`og.story.parse`](#fn-parse) | function | A dialogue or quest from text, now (tests and generated content; assets go through og cook). Registers og.Story as load does. |
| [`og.story.start`](#fn-start) | function | Start a conversation: it runs to its first line (or its choices) now, and og.DialogueShown is sent the next tick. Several may run at once. |
| [`og.story.view`](#fn-view) | function | What a conversation shows now, resolved in the active locale. |
| [`og.story.history`](#fn-history) | function | A conversation's lines so far, oldest first, with the choice made on each (a VN backlog). |
| [`og.story.conversations`](#fn-conversations) | function | The conversations that exist (started and not stopped), oldest first. |
| [`og.story.advance`](#fn-advance) | function | Move a conversation past its line: it runs to the next line, its choices or its end. |
| [`og.story.choose`](#fn-choose) | function | Take a choice: its `set`, `emit` and `quest` steps run, an active check rolls (og.DialogueCheck), and the conversation goes to its `goto` (or `fail` on a missed roll), or ends without one. |
| [`og.story.stop`](#fn-stop) | function | End a conversation now (og.DialogueEnded if it was running) and forget it. An ended conversation stays viewable until it is stopped. |
| [`og.story.bark`](#fn-bark) | function | Ask an entity to bark a line of a topic: at the og.story system's next run, of that topic's barks whose `when` holds (it may read the world: `self.Health.hp`, `Game.alarm`) and whose cooldowns are over, the highest priority is said, ties drawn by weight on the story's stream; og.BarkSaid follows. Nothing is said when none qualifies, or the entity is saying a bark of equal or higher priority. |
| [`og.story.ambient`](#fn-ambient) | function | Let an entity bark a topic on its own, every `every` ticks (and up to `jitter` more), as og.story.bark would; `every` 0 stops it. In saves; a despawned entity stops. |
| [`og.story.barks`](#fn-barks) | function | The barks being said, by entity: draw each over its entity (a speech bubble) until it ends. |
| [`og.story.get`](#fn-get) | function | A story variable (a `global` of any loaded dialogue or quest): its value, else its declared default. |
| [`og.story.set`](#fn-set) | function | Set a story variable; quests whose conditions now hold advance. |
| [`og.story.reset`](#fn-reset) | function | A new playthrough: the story's variables, seen nodes, conversations and quests are cleared. Lines read stay read (DialogueView.seen), so a second playthrough can skip them. |
| [`og.story.startQuest`](#fn-startQuest) | function | Start a loaded quest at its first stage. |
| [`og.story.progress`](#fn-progress) | function | Count toward an objective of an active quest's current stage (a wolf killed, a herb picked); the stage advances when its objectives are done. |
| [`og.story.setStage`](#fn-setStage) | function | Move an active quest to a stage, its counts cleared. |
| [`og.story.completeQuest`](#fn-completeQuest) | function | Complete an active quest now. |
| [`og.story.failQuest`](#fn-failQuest) | function | Fail an active quest now: its undone objectives are marked failed. |
| [`og.story.quest`](#fn-quest) | function | A started quest as the log shows it. |
| [`og.story.quests`](#fn-quests) | function | Every started quest, in the order they started (a quest log). |
| [`og.story.track`](#fn-track) | function | Show a quest in the HUD tracker, or stop showing it. |
| [`og.story.markSeen`](#fn-markSeen) | function | The player looked at a quest: clear QuestView.isNew and updated. |
| [`og.story.mark`](#fn-mark) | function | Point a quest at an entity or a point (the map, compass and tracker read QuestView.marker), over its objectives' own markers. |

## Functions

<a id="fn-load"></a>

### `og.story.load`

A cooked dialogue or quest (an `.ogt` of `type dialogue` or `type quest`, validated by og cook). Loading again replaces it, so a live edit takes effect at the conversation's next step. The first load registers og.Story and the story's events, so call it in setup: inside a system it is refused until one load has run outside one.

```ts
og.story.load(world: World, path: string): Result<StoryAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The asset, such as `story/smith.ogt`. |

**Returns** `Result<StoryAsset>`: Refused for a path that is not a cooked dialogue or quest.

Binding symbol `og_story_load` · publish builds: kept

```ogt
fn load {
  doc "A cooked dialogue or quest (an `.ogt` of `type dialogue` or `type quest`, validated by og cook). Loading again replaces it, so a live edit takes effect at the conversation's next step. The first load registers og.Story and the story's events, so call it in setup: inside a system it is refused until one load has run outside one."
  param world World "The world."
  param path String "The asset, such as `story/smith.ogt`."
  returns Result.StoryAsset "Refused for a path that is not a cooked dialogue or quest."
}
```

<sub>`api.ogt` line 14224</sub>

<a id="fn-parse"></a>

### `og.story.parse`

A dialogue or quest from text, now (tests and generated content; assets go through og cook). Registers og.Story as load does.

```ts
og.story.parse(world: World, text: string): Result<StoryAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `text` | `string` | no | The `.ogt` text. |

**Returns** `Result<StoryAsset>`: Refused with the first error's line and the fix.

Binding symbol `og_story_parse` · publish builds: kept

```ogt
fn parse {
  doc "A dialogue or quest from text, now (tests and generated content; assets go through og cook). Registers og.Story as load does."
  param world World "The world."
  param text String "The `.ogt` text."
  returns Result.StoryAsset "Refused with the first error's line and the fix."
}
```

<sub>`api.ogt` line 14231</sub>

<a id="fn-start"></a>

### `og.story.start`

Start a conversation: it runs to its first line (or its choices) now, and og.DialogueShown is sent the next tick. Several may run at once.

```ts
og.story.start(world: World, desc?: StartDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `StartDesc` | yes | The dialogue, and optionally the node, cast and starting variables. |

**Returns** `Result<number>`: The conversation, or why not: an unloaded dialogue, an unknown node, speaker or variable.

Binding symbol `og_story_start` · publish builds: kept

```ogt
fn start {
  doc "Start a conversation: it runs to its first line (or its choices) now, and og.DialogueShown is sent the next tick. Several may run at once."
  param world World "The world."
  param desc StartDesc "The dialogue, and optionally the node, cast and starting variables."
  returns Result.u32 "The conversation, or why not: an unloaded dialogue, an unknown node, speaker or variable."
}
```

<sub>`api.ogt` line 14238</sub>

<a id="fn-view"></a>

### `og.story.view`

What a conversation shows now, resolved in the active locale.

```ts
og.story.view(world: World, conversation: number): DialogueView | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |

**Returns** `DialogueView | null`: Null for a conversation that does not exist (never started, or stopped).

Binding symbol `og_story_view` · publish builds: kept

```ogt
fn view {
  doc "What a conversation shows now, resolved in the active locale."
  param world World "The world."
  param conversation u32 "The conversation."
  returns Option.DialogueView "Null for a conversation that does not exist (never started, or stopped)."
}
```

<sub>`api.ogt` line 14245</sub>

<a id="fn-history"></a>

### `og.story.history`

A conversation's lines so far, oldest first, with the choice made on each (a VN backlog).

```ts
og.story.history(world: World, conversation: number): readonly StoryLine[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |

**Returns** `readonly StoryLine[]`: Empty for an unknown conversation.

Binding symbol `og_story_history` · publish builds: kept

```ogt
fn history {
  doc "A conversation's lines so far, oldest first, with the choice made on each (a VN backlog)."
  param world World "The world."
  param conversation u32 "The conversation."
  returns List.StoryLine "Empty for an unknown conversation."
}
```

<sub>`api.ogt` line 14252</sub>

<a id="fn-conversations"></a>

### `og.story.conversations`

The conversations that exist (started and not stopped), oldest first.

```ts
og.story.conversations(world: World): readonly number[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly number[]`.

Binding symbol `og_story_conversations` · publish builds: kept

```ogt
fn conversations {
  doc "The conversations that exist (started and not stopped), oldest first."
  param world World "The world."
  returns List.u32
}
```

<sub>`api.ogt` line 14259</sub>

<a id="fn-advance"></a>

### `og.story.advance`

Move a conversation past its line: it runs to the next line, its choices or its end.

```ts
og.story.advance(world: World, conversation: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |

**Returns** `Result<void>`: Refused while it waits on choices, after it ended, or for an unknown conversation.

Binding symbol `og_story_advance` · publish builds: kept

```ogt
fn advance {
  doc "Move a conversation past its line: it runs to the next line, its choices or its end."
  param world World "The world."
  param conversation u32 "The conversation."
  returns Result.Void "Refused while it waits on choices, after it ended, or for an unknown conversation."
}
```

<sub>`api.ogt` line 14265</sub>

<a id="fn-choose"></a>

### `og.story.choose`

Take a choice: its `set`, `emit` and `quest` steps run, an active check rolls (og.DialogueCheck), and the conversation goes to its `goto` (or `fail` on a missed roll), or ends without one.

```ts
og.story.choose(world: World, conversation: number, index: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |
| `index` | `number` | no | ChoiceView.index. |

**Returns** `Result<void>`: Refused for a hidden or disabled choice, a conversation showing a line, or an unknown one.

Binding symbol `og_story_choose` · publish builds: kept

```ogt
fn choose {
  doc "Take a choice: its `set`, `emit` and `quest` steps run, an active check rolls (og.DialogueCheck), and the conversation goes to its `goto` (or `fail` on a missed roll), or ends without one."
  param world World "The world."
  param conversation u32 "The conversation."
  param index u32 "ChoiceView.index."
  returns Result.Void "Refused for a hidden or disabled choice, a conversation showing a line, or an unknown one."
}
```

<sub>`api.ogt` line 14272</sub>

<a id="fn-stop"></a>

### `og.story.stop`

End a conversation now (og.DialogueEnded if it was running) and forget it. An ended conversation stays viewable until it is stopped.

```ts
og.story.stop(world: World, conversation: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |

**Returns** `boolean`: False for an unknown conversation.

Binding symbol `og_story_stop` · publish builds: kept

```ogt
fn stop {
  doc "End a conversation now (og.DialogueEnded if it was running) and forget it. An ended conversation stays viewable until it is stopped."
  param world World "The world."
  param conversation u32 "The conversation."
  returns bool "False for an unknown conversation."
}
```

<sub>`api.ogt` line 14280</sub>

<a id="fn-bark"></a>

### `og.story.bark`

Ask an entity to bark a line of a topic: at the og.story system's next run, of that topic's barks whose `when` holds (it may read the world: `self.Health.hp`, `Game.alarm`) and whose cooldowns are over, the highest priority is said, ties drawn by weight on the story's stream; og.BarkSaid follows. Nothing is said when none qualifies, or the entity is saying a bark of equal or higher priority.

```ts
og.story.bark(world: World, desc?: BarkDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `BarkDesc` | yes | Who and which topic. |

**Returns** `Result<void>`: Refused for a topic no loaded dialogue has.

Binding symbol `og_story_bark` · publish builds: kept

```ogt
fn bark {
  doc "Ask an entity to bark a line of a topic: at the og.story system's next run, of that topic's barks whose `when` holds (it may read the world: `self.Health.hp`, `Game.alarm`) and whose cooldowns are over, the highest priority is said, ties drawn by weight on the story's stream; og.BarkSaid follows. Nothing is said when none qualifies, or the entity is saying a bark of equal or higher priority."
  param world World "The world."
  param desc BarkDesc "Who and which topic."
  returns Result.Void "Refused for a topic no loaded dialogue has."
}
```

<sub>`api.ogt` line 14287</sub>

<a id="fn-ambient"></a>

### `og.story.ambient`

Let an entity bark a topic on its own, every `every` ticks (and up to `jitter` more), as og.story.bark would; `every` 0 stops it. In saves; a despawned entity stops.

```ts
og.story.ambient(world: World, desc?: AmbientDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `AmbientDesc` | yes | Who, which topic and how often. |

**Returns** `Result<void>`: Refused for a topic no loaded dialogue has.

Binding symbol `og_story_ambient` · publish builds: kept

```ogt
fn ambient {
  doc "Let an entity bark a topic on its own, every `every` ticks (and up to `jitter` more), as og.story.bark would; `every` 0 stops it. In saves; a despawned entity stops."
  param world World "The world."
  param desc AmbientDesc "Who, which topic and how often."
  returns Result.Void "Refused for a topic no loaded dialogue has."
}
```

<sub>`api.ogt` line 14294</sub>

<a id="fn-barks"></a>

### `og.story.barks`

The barks being said, by entity: draw each over its entity (a speech bubble) until it ends.

```ts
og.story.barks(world: World): readonly BarkView[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly BarkView[]`.

Binding symbol `og_story_barks` · publish builds: kept

```ogt
fn barks {
  doc "The barks being said, by entity: draw each over its entity (a speech bubble) until it ends."
  param world World "The world."
  returns List.BarkView
}
```

<sub>`api.ogt` line 14301</sub>

<a id="fn-get"></a>

### `og.story.get`

A story variable (a `global` of any loaded dialogue or quest): its value, else its declared default.

```ts
og.story.get(world: World, name: string): unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The variable. |

**Returns** `unknown`: A number, a boolean, a string, or null for one nothing declares.

Binding symbol `og_story_get` · publish builds: kept

```ogt
fn get {
  doc "A story variable (a `global` of any loaded dialogue or quest): its value, else its declared default."
  param world World "The world."
  param name String "The variable."
  returns Json "A number, a boolean, a string, or null for one nothing declares."
}
```

<sub>`api.ogt` line 14307</sub>

<a id="fn-set"></a>

### `og.story.set`

Set a story variable; quests whose conditions now hold advance.

```ts
og.story.set(world: World, name: string, value: unknown): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The variable. |
| `value` | `unknown` | no | A number, a boolean or a string. |

**Returns** `Result<void>`: Refused for another kind of value or a world without a loaded story.

Binding symbol `og_story_set` · publish builds: kept

```ogt
fn set {
  doc "Set a story variable; quests whose conditions now hold advance."
  param world World "The world."
  param name String "The variable."
  param value Json "A number, a boolean or a string."
  returns Result.Void "Refused for another kind of value or a world without a loaded story."
}
```

<sub>`api.ogt` line 14314</sub>

<a id="fn-reset"></a>

### `og.story.reset`

A new playthrough: the story's variables, seen nodes, conversations and quests are cleared. Lines read stay read (DialogueView.seen), so a second playthrough can skip them.

```ts
og.story.reset(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `void`.

Binding symbol `og_story_reset` · publish builds: kept

```ogt
fn reset {
  doc "A new playthrough: the story's variables, seen nodes, conversations and quests are cleared. Lines read stay read (DialogueView.seen), so a second playthrough can skip them."
  param world World "The world."
}
```

<sub>`api.ogt` line 14322</sub>

<a id="fn-startQuest"></a>

### `og.story.startQuest`

Start a loaded quest at its first stage.

```ts
og.story.startQuest(world: World, id: string): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |

**Returns** `Result<boolean>`: False when it had started before. Refused for a quest not loaded.

Binding symbol `og_story_start_quest` · publish builds: kept

```ogt
fn startQuest {
  doc "Start a loaded quest at its first stage."
  param world World "The world."
  param id String "The quest's id."
  returns Result.bool "False when it had started before. Refused for a quest not loaded."
}
```

<sub>`api.ogt` line 14327</sub>

<a id="fn-progress"></a>

### `og.story.progress`

Count toward an objective of an active quest's current stage (a wolf killed, a herb picked); the stage advances when its objectives are done.

```ts
og.story.progress(world: World, quest: string, objective: string, amount?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `quest` | `string` | no | The quest's id. |
| `objective` | `string` | no | The objective's id. |
| `amount` | `number` | yes | How many (default 1). |

**Returns** `boolean`: False when the quest is not active or its stage has no such objective.

Binding symbol `og_story_progress` · publish builds: kept

```ogt
fn progress {
  doc "Count toward an objective of an active quest's current stage (a wolf killed, a herb picked); the stage advances when its objectives are done."
  param world World "The world."
  param quest String "The quest's id."
  param objective String "The objective's id."
  param amount u32 "How many (default 1)." {
    optional true
  }
  returns bool "False when the quest is not active or its stage has no such objective."
}
```

<sub>`api.ogt` line 14334</sub>

<a id="fn-setStage"></a>

### `og.story.setStage`

Move an active quest to a stage, its counts cleared.

```ts
og.story.setStage(world: World, quest: string, stage: string): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `quest` | `string` | no | The quest's id. |
| `stage` | `string` | no | The stage's id. |

**Returns** `Result<boolean>`: False when it is not active or already there. Refused for an unknown quest or stage.

Binding symbol `og_story_set_stage` · publish builds: kept

```ogt
fn setStage {
  doc "Move an active quest to a stage, its counts cleared."
  param world World "The world."
  param quest String "The quest's id."
  param stage String "The stage's id."
  returns Result.bool "False when it is not active or already there. Refused for an unknown quest or stage."
}
```

<sub>`api.ogt` line 14345</sub>

<a id="fn-completeQuest"></a>

### `og.story.completeQuest`

Complete an active quest now.

```ts
og.story.completeQuest(world: World, quest: string): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `quest` | `string` | no | The quest's id. |

**Returns** `Result<boolean>`: False when it is not active. Refused for a quest not loaded.

Binding symbol `og_story_complete_quest` · publish builds: kept

```ogt
fn completeQuest {
  doc "Complete an active quest now."
  param world World "The world."
  param quest String "The quest's id."
  returns Result.bool "False when it is not active. Refused for a quest not loaded."
}
```

<sub>`api.ogt` line 14353</sub>

<a id="fn-failQuest"></a>

### `og.story.failQuest`

Fail an active quest now: its undone objectives are marked failed.

```ts
og.story.failQuest(world: World, quest: string): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `quest` | `string` | no | The quest's id. |

**Returns** `Result<boolean>`: False when it is not active. Refused for a quest not loaded.

Binding symbol `og_story_fail_quest` · publish builds: kept

```ogt
fn failQuest {
  doc "Fail an active quest now: its undone objectives are marked failed."
  param world World "The world."
  param quest String "The quest's id."
  returns Result.bool "False when it is not active. Refused for a quest not loaded."
}
```

<sub>`api.ogt` line 14360</sub>

<a id="fn-quest"></a>

### `og.story.quest`

A started quest as the log shows it.

```ts
og.story.quest(world: World, id: string): QuestView | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |

**Returns** `QuestView | null`: Null for a quest not started.

Binding symbol `og_story_quest` · publish builds: kept

```ogt
fn quest {
  doc "A started quest as the log shows it."
  param world World "The world."
  param id String "The quest's id."
  returns Option.QuestView "Null for a quest not started."
}
```

<sub>`api.ogt` line 14367</sub>

<a id="fn-quests"></a>

### `og.story.quests`

Every started quest, in the order they started (a quest log).

```ts
og.story.quests(world: World): readonly QuestView[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly QuestView[]`.

Binding symbol `og_story_quests` · publish builds: kept

```ogt
fn quests {
  doc "Every started quest, in the order they started (a quest log)."
  param world World "The world."
  returns List.QuestView
}
```

<sub>`api.ogt` line 14374</sub>

<a id="fn-track"></a>

### `og.story.track`

Show a quest in the HUD tracker, or stop showing it.

```ts
og.story.track(world: World, id: string, on: boolean): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |
| `on` | `boolean` | no | Tracked. |

**Returns** `boolean`: False for a quest not started.

Binding symbol `og_story_track` · publish builds: kept

```ogt
fn track {
  doc "Show a quest in the HUD tracker, or stop showing it."
  param world World "The world."
  param id String "The quest's id."
  param on bool "Tracked."
  returns bool "False for a quest not started."
}
```

<sub>`api.ogt` line 14380</sub>

<a id="fn-markSeen"></a>

### `og.story.markSeen`

The player looked at a quest: clear QuestView.isNew and updated.

```ts
og.story.markSeen(world: World, id: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |

**Returns** `boolean`: False for a quest not started.

Binding symbol `og_story_mark_seen` · publish builds: kept

```ogt
fn markSeen {
  doc "The player looked at a quest: clear QuestView.isNew and updated."
  param world World "The world."
  param id String "The quest's id."
  returns bool "False for a quest not started."
}
```

<sub>`api.ogt` line 14388</sub>

<a id="fn-mark"></a>

### `og.story.mark`

Point a quest at an entity or a point (the map, compass and tracker read QuestView.marker), over its objectives' own markers.

```ts
og.story.mark(world: World, id: string, desc?: MarkerDesc): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |
| `desc` | `MarkerDesc` | yes | The entity or point; both zero clears it. |

**Returns** `boolean`: False for a quest not started.

Binding symbol `og_story_mark` · publish builds: kept

```ogt
fn mark {
  doc "Point a quest at an entity or a point (the map, compass and tracker read QuestView.marker), over its objectives' own markers."
  param world World "The world."
  param id String "The quest's id."
  param desc MarkerDesc "The entity or point; both zero clears it."
  returns bool "False for a quest not started."
}
```

<sub>`api.ogt` line 14395</sub>

## Components, resources and events

<a id="record-Story"></a>

### `og.Story` (resource)

The story's counters (written by og.story; the story itself is a state block). Registered at the first og.story.load.

Token `og.Story`, registered under the schema name `og.Story`; its value type is `Story`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lines` | `number` | `0` | — | — | read only | Lines shown so far (DialogueView.seq of the latest). |
| `conversations` | `number` | `0` | — | — | read only | Conversations not yet ended. |
| `activeQuests` | `number` | `0` | — | — | read only | Quests started and not yet completed or failed. |

```ogt
resource Story {
  doc "The story's counters (written by og.story; the story itself is a state block). Registered at the first og.story.load."
  field lines u32 "Lines shown so far (DialogueView.seq of the latest)." {
    readonly true
  }
  field conversations u32 "Conversations not yet ended." {
    readonly true
  }
  field activeQuests u32 "Quests started and not yet completed or failed." {
    readonly true
  }
}
```

<sub>`api.ogt` line 13972</sub>

<a id="record-DialogueShown"></a>

### `og.DialogueShown` (event)

A conversation shows a new line (start the voice, reset the reveal). Sent before the game's systems, the tick after the call that moved it.

Token `og.DialogueShown`, registered under the schema name `og.DialogueShown`; its value type is `DialogueShown`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation. |
| `seq` | `number` | `0` | — | — | — | The line's DialogueView.seq. |

```ogt
event DialogueShown {
  doc "A conversation shows a new line (start the voice, reset the reveal). Sent before the game's systems, the tick after the call that moved it."
  field conversation u32 "The conversation."
  field seq u32 "The line's DialogueView.seq."
}
```

<sub>`api.ogt` line 13985</sub>

<a id="record-DialogueEnded"></a>

### `og.DialogueEnded` (event)

A conversation ended: its dialogue ran out, reached `end`, took a choice without `goto`, or og.story.stop stopped it. Sent the tick after.

Token `og.DialogueEnded`, registered under the schema name `og.DialogueEnded`; its value type is `DialogueEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation. |

```ogt
event DialogueEnded {
  doc "A conversation ended: its dialogue ran out, reached `end`, took a choice without `goto`, or og.story.stop stopped it. Sent the tick after."
  field conversation u32 "The conversation."
}
```

<sub>`api.ogt` line 13991</sub>

<a id="record-DialogueCheck"></a>

### `og.DialogueCheck` (event)

An active check was rolled (d20 plus the stat against the target): reveal the roll. Sent the tick after.

Token `og.DialogueCheck`, registered under the schema name `og.DialogueCheck`; its value type is `DialogueCheck`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation. |
| `stat` | `string` | `""` | — | — | — | The variable read as the stat. |
| `target` | `number` | `0` | — | — | — | What the roll plus the stat had to reach. |
| `roll` | `number` | `0` | — | — | — | The d20, 1 to 20. |
| `passed` | `boolean` | `false` | — | — | — | Whether roll + stat reached the target. |

```ogt
event DialogueCheck {
  doc "An active check was rolled (d20 plus the stat against the target): reveal the roll. Sent the tick after."
  field conversation u32 "The conversation."
  field stat String "The variable read as the stat."
  field target f32 "What the roll plus the stat had to reach."
  field roll u32 "The d20, 1 to 20."
  field passed bool "Whether roll + stat reached the target."
}
```

<sub>`api.ogt` line 13996</sub>

<a id="record-QuestUpdated"></a>

### `og.QuestUpdated` (event)

A quest started, counted, changed stage, completed or failed. Sent the tick after.

Token `og.QuestUpdated`, registered under the schema name `og.QuestUpdated`; its value type is `QuestUpdated`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `quest` | `string` | `""` | — | — | — | The quest's id. |
| `status` | `string` | `""` | — | — | — | active, completed or failed. |

```ogt
event QuestUpdated {
  doc "A quest started, counted, changed stage, completed or failed. Sent the tick after."
  field quest String "The quest's id."
  field status String "active, completed or failed."
}
```

<sub>`api.ogt` line 14005</sub>

<a id="record-BarkSaid"></a>

### `og.BarkSaid` (event)

An entity barked (og.story.bark or og.story.ambient): show its line over it with og.story.barks. Sent the tick after it was asked for.

Token `og.BarkSaid`, registered under the schema name `og.BarkSaid`; its value type is `BarkSaid`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Who barked. |
| `bark` | `string` | `""` | — | — | — | The bark's id in its dialogue. |
| `key` | `string` | `""` | — | — | — | The line's i18n key. |
| `seq` | `number` | `0` | — | — | — | The line's number (BarkView.seq). |

```ogt
event BarkSaid {
  doc "An entity barked (og.story.bark or og.story.ambient): show its line over it with og.story.barks. Sent the tick after it was asked for."
  field entity Entity "Who barked."
  field bark String "The bark's id in its dialogue."
  field key String "The line's i18n key."
  field seq u32 "The line's number (BarkView.seq)."
}
```

<sub>`api.ogt` line 14011</sub>

<a id="record-StoryEvent"></a>

### `og.StoryEvent` (event)

A dialogue's `emit name "arg"`: open the shop, give the sword, play the cutscene. Sent the tick after.

Token `og.StoryEvent`, registered under the schema name `og.StoryEvent`; its value type is `StoryEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation that emitted it. |
| `name` | `string` | `""` | — | — | — | The name written after emit. |
| `arg` | `string` | `""` | — | — | — | Its argument, evaluated and written as text (empty without one). |

```ogt
event StoryEvent {
  doc "A dialogue's `emit name \"arg\"`: open the shop, give the sword, play the cutscene. Sent the tick after."
  field conversation u32 "The conversation that emitted it."
  field name String "The name written after emit."
  field arg String "Its argument, evaluated and written as text (empty without one)."
}
```

<sub>`api.ogt` line 14019</sub>

## Types

<a id="type-StoryAsset"></a>

### `StoryAsset` (struct)

A loaded dialogue or quest.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | — | dialogue or quest. |
| `name` | `string` | `""` | — | — | — | A dialogue's name or a quest's id: what og.story.start and og.story.startQuest take. |
| `nodes` | `readonly string[]` | `[]` | — | — | — | A dialogue's nodes, in order (a quest: its stages). |
| `speakers` | `readonly string[]` | `[]` | — | — | — | A dialogue's speakers (a quest: none). |
| `variables` | `readonly string[]` | `[]` | — | — | — | The variables it declares (var: the conversation's; global: the story's). |

```ogt
struct StoryAsset {
  doc "A loaded dialogue or quest."
  field kind String "dialogue or quest."
  field name String "A dialogue's name or a quest's id: what og.story.start and og.story.startQuest take."
  field nodes List.String "A dialogue's nodes, in order (a quest: its stages)."
  field speakers List.String "A dialogue's speakers (a quest: none)."
  field variables List.String "The variables it declares (var: the conversation's; global: the story's)."
}
```

<sub>`api.ogt` line 14026</sub>

<a id="type-StartDesc"></a>

### `StartDesc` (desc struct)

A conversation to start.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dialogue` | `string` | `""` | — | — | — | The dialogue's name (StoryAsset.name). |
| `node` | `string` | `""` | — | — | — | The node to start at (empty: the dialogue's `start`). |
| `cast` | `unknown` | `null` | — | — | — | Speaker ids to the entities playing them, `{ smith: entity }`: SpeakerView.entity (for a speech bubble or a camera). |
| `vars` | `unknown` | `null` | — | — | — | Starting values for the dialogue's own `var`s, `{ mood: 2 }`. |

```ogt
struct StartDesc {
  doc "A conversation to start."
  desc true
  field dialogue String "The dialogue's name (StoryAsset.name)."
  field node String "The node to start at (empty: the dialogue's `start`)."
  field cast Json "Speaker ids to the entities playing them, `{ smith: entity }`: SpeakerView.entity (for a speech bubble or a camera)."
  field vars Json "Starting values for the dialogue's own `var`s, `{ mood: 2 }`."
}
```

<sub>`api.ogt` line 14035</sub>

<a id="type-SpeakerView"></a>

### `SpeakerView` (struct)

Who speaks a line.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The speaker's id (`narrator` for the narrator). |
| `name` | `string` | `""` | — | — | — | Its name, resolved in the active locale (empty for the narrator). |
| `entity` | `Entity` | `0` | — | — | — | The entity playing it (StartDesc.cast; 0: none). |
| `portrait` | `string` | `""` | — | — | — | A portrait image's asset path (empty: none). |
| `expression` | `string` | `""` | — | — | — | The portrait's variant: the line's `expression`, else the speaker's. |
| `side` | `string` | `""` | — | — | — | left, right or center (VN staging). |

```ogt
struct SpeakerView {
  doc "Who speaks a line."
  field id String "The speaker's id (`narrator` for the narrator)."
  field name String "Its name, resolved in the active locale (empty for the narrator)."
  field entity Entity "The entity playing it (StartDesc.cast; 0: none)."
  field portrait String "A portrait image's asset path (empty: none)."
  field expression String "The portrait's variant: the line's `expression`, else the speaker's."
  field side String "left, right or center (VN staging)."
}
```

<sub>`api.ogt` line 14044</sub>

<a id="type-CheckView"></a>

### `CheckView` (struct)

A choice's check.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `stat` | `string` | `""` | — | — | — | The variable read as the stat. |
| `target` | `number` | `0` | — | — | — | What the stat (passive) or a d20 plus the stat (active) must reach. |
| `chance` | `number` | `0` | — | — | — | The chance it passes, 0 to 1: an active check's share of the d20's faces that pass; a passive one's 1 or 0. |
| `kind` | `string` | `""` | — | — | — | passive (a threshold that enables the choice) or active (rolled when chosen). |

```ogt
struct CheckView {
  doc "A choice's check."
  field stat String "The variable read as the stat."
  field target f32 "What the stat (passive) or a d20 plus the stat (active) must reach."
  field chance f32 "The chance it passes, 0 to 1: an active check's share of the d20's faces that pass; a passive one's 1 or 0."
  field kind String "passive (a threshold that enables the choice) or active (rolled when chosen)."
}
```

<sub>`api.ogt` line 14054</sub>

<a id="type-ChoiceView"></a>

### `ChoiceView` (struct)

A choice the node offers (hidden ones are left out).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `0` | — | — | — | What og.story.choose takes: its place among the node's choices, hidden ones counted. |
| `id` | `string` | `""` | — | — | — | Its id within the node. |
| `text` | `string` | `""` | — | — | — | Its text, resolved in the active locale. |
| `key` | `string` | `""` | — | — | — | Its i18n key. |
| `args` | `unknown` | `null` | — | — | — | The arguments its text was resolved with. |
| `enabled` | `boolean` | `false` | — | — | — | Whether it can be chosen now (its `if` holds and a passive check passes). |
| `reason` | `string` | `""` | — | — | — | Why it is disabled, resolved in the active locale (empty without a `reason`). |
| `reasonKey` | `string` | `""` | — | — | — | The reason's i18n key. |
| `reasonArgs` | `unknown` | `null` | — | — | — | The arguments the reason was resolved with. |
| `visited` | `boolean` | `false` | — | — | — | Chosen before in this conversation (drawn muted). |
| `kind` | `string` | `""` | — | — | — | The game's word for its icon: leave, trade, attack, lie... |
| `check` | `CheckView \| null` | `null` | — | — | — | Its check, if it has one. |

```ogt
struct ChoiceView {
  doc "A choice the node offers (hidden ones are left out)."
  field index u32 "What og.story.choose takes: its place among the node's choices, hidden ones counted."
  field id String "Its id within the node."
  field text String "Its text, resolved in the active locale."
  field key String "Its i18n key."
  field args Json "The arguments its text was resolved with."
  field enabled bool "Whether it can be chosen now (its `if` holds and a passive check passes)."
  field reason String "Why it is disabled, resolved in the active locale (empty without a `reason`)."
  field reasonKey String "The reason's i18n key."
  field reasonArgs Json "The arguments the reason was resolved with."
  field visited bool "Chosen before in this conversation (drawn muted)."
  field kind String "The game's word for its icon: leave, trade, attack, lie..."
  field check Option.CheckView "Its check, if it has one."
}
```

<sub>`api.ogt` line 14062</sub>

<a id="type-TimerView"></a>

### `TimerView` (struct)

A node's choice timer.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ticksLeft` | `number` | `0` | — | — | — | Ticks until it runs out. |
| `ticksTotal` | `number` | `0` | — | — | — | Its length in ticks. |
| `defaultChoice` | `number` | `0` | — | — | — | The ChoiceView.index taken when it runs out. |

```ogt
struct TimerView {
  doc "A node's choice timer."
  field ticksLeft u32 "Ticks until it runs out."
  field ticksTotal u32 "Its length in ticks."
  field defaultChoice u32 "The ChoiceView.index taken when it runs out."
}
```

<sub>`api.ogt` line 14078</sub>

<a id="type-RollView"></a>

### `RollView` (struct)

An active check's roll.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `stat` | `string` | `""` | — | — | — | The variable read as the stat. |
| `target` | `number` | `0` | — | — | — | What the roll plus the stat had to reach. |
| `roll` | `number` | `0` | — | — | — | The d20, 1 to 20. |
| `passed` | `boolean` | `false` | — | — | — | Whether it passed. |

```ogt
struct RollView {
  doc "An active check's roll."
  field stat String "The variable read as the stat."
  field target f32 "What the roll plus the stat had to reach."
  field roll u32 "The d20, 1 to 20."
  field passed bool "Whether it passed."
}
```

<sub>`api.ogt` line 14085</sub>

<a id="type-DialogueView"></a>

### `DialogueView` (struct)

What a conversation shows now: the line in full (the dialogue box paces its own reveal), its speaker and the choices, if it waits on them.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation. |
| `node` | `string` | `""` | — | — | — | The node it is in. |
| `seq` | `number` | `0` | — | — | — | Counts lines shown across the story: a new number is a new line, even when a node repeats. |
| `seen` | `boolean` | `false` | — | — | — | The line had been read before it was shown (skip-read, a dimmed backlog); og.story.reset keeps what was read. |
| `key` | `string` | `""` | — | — | — | The line's i18n key. |
| `args` | `unknown` | `null` | — | — | — | The arguments it was resolved with (evaluated when it was shown). |
| `text` | `string` | `""` | — | — | — | The line, resolved in the active locale. |
| `voice` | `string` | `""` | — | — | — | Its voice clip's asset path (empty: none). og.story plays it on its own entity (an og.AudioSource on the sfx bus) from the tick after the line is shown, and stops it when the line moves on. |
| `voiceTicks` | `number` | `0` | — | — | — | The voice's length in ticks (0: no voice, or one that did not load). |
| `voiceLeft` | `number` | `0` | — | — | — | Ticks of the voice still to play: voiceTicks until it starts, 0 once it has ended (advance then, for auto-advance). |
| `speaker` | `SpeakerView` | `{ id: "", name: "", entity: 0, portrait: "", expression: "", side: "" }` | — | — | — | Who says it. |
| `shot` | `string` | `""` | — | — | — | The camera shot it is shown in: its `shot`, else its node's (a virtual camera's entity name or a cast role); empty: none. While it shows, og.story makes that og.VirtualCamera og.CameraBrain.solo: a name is the entity of that name, a role the cast entity's own og.VirtualCamera, else the first one whose lookAt is that entity. When no running conversation's line has a shot the game's own solo is handed back. |
| `choices` | `readonly ChoiceView[]` | `[]` | — | — | — | The choices, when it waits on them; empty while a line waits for og.story.advance. |
| `timer` | `TimerView \| null` | `null` | — | — | — | The choice timer, while it runs. |
| `lastCheck` | `RollView \| null` | `null` | — | — | — | The last active check this conversation rolled. |
| `ended` | `boolean` | `false` | — | — | — | The conversation has ended (the view keeps its last line). |

```ogt
struct DialogueView {
  doc "What a conversation shows now: the line in full (the dialogue box paces its own reveal), its speaker and the choices, if it waits on them."
  field conversation u32 "The conversation."
  field node String "The node it is in."
  field seq u32 "Counts lines shown across the story: a new number is a new line, even when a node repeats."
  field seen bool "The line had been read before it was shown (skip-read, a dimmed backlog); og.story.reset keeps what was read."
  field key String "The line's i18n key."
  field args Json "The arguments it was resolved with (evaluated when it was shown)."
  field text String "The line, resolved in the active locale."
  field voice String "Its voice clip's asset path (empty: none). og.story plays it on its own entity (an og.AudioSource on the sfx bus) from the tick after the line is shown, and stops it when the line moves on."
  field voiceTicks u32 "The voice's length in ticks (0: no voice, or one that did not load)."
  field voiceLeft u32 "Ticks of the voice still to play: voiceTicks until it starts, 0 once it has ended (advance then, for auto-advance)."
  field speaker SpeakerView "Who says it."
  field shot String "The camera shot it is shown in: its `shot`, else its node's (a virtual camera's entity name or a cast role); empty: none. While it shows, og.story makes that og.VirtualCamera og.CameraBrain.solo: a name is the entity of that name, a role the cast entity's own og.VirtualCamera, else the first one whose lookAt is that entity. When no running conversation's line has a shot the game's own solo is handed back."
  field choices List.ChoiceView "The choices, when it waits on them; empty while a line waits for og.story.advance."
  field timer Option.TimerView "The choice timer, while it runs."
  field lastCheck Option.RollView "The last active check this conversation rolled."
  field ended bool "The conversation has ended (the view keeps its last line)."
}
```

<sub>`api.ogt` line 14093</sub>

<a id="type-ChosenView"></a>

### `ChosenView` (struct)

The choice made while a line was shown.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `0` | — | — | — | Its ChoiceView.index. |
| `text` | `string` | `""` | — | — | — | Its text, resolved in the active locale. |

```ogt
struct ChosenView {
  doc "The choice made while a line was shown."
  field index u32 "Its ChoiceView.index."
  field text String "Its text, resolved in the active locale."
}
```

<sub>`api.ogt` line 14113</sub>

<a id="type-StoryLine"></a>

### `StoryLine` (struct)

A line of a conversation's backlog.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seq` | `number` | `0` | — | — | — | Its DialogueView.seq. |
| `node` | `string` | `""` | — | — | — | Its node. |
| `speaker` | `SpeakerView` | `{ id: "", name: "", entity: 0, portrait: "", expression: "", side: "" }` | — | — | — | Who said it. |
| `key` | `string` | `""` | — | — | — | Its i18n key. |
| `args` | `unknown` | `null` | — | — | — | Its arguments. |
| `text` | `string` | `""` | — | — | — | The line, resolved in the active locale. |
| `voice` | `string` | `""` | — | — | — | Its voice clip (empty: none). |
| `seen` | `boolean` | `false` | — | — | — | It had been read before it was shown. |
| `chosen` | `ChosenView \| null` | `null` | — | — | — | The choice made on it, if any. |

```ogt
struct StoryLine {
  doc "A line of a conversation's backlog."
  field seq u32 "Its DialogueView.seq."
  field node String "Its node."
  field speaker SpeakerView "Who said it."
  field key String "Its i18n key."
  field args Json "Its arguments."
  field text String "The line, resolved in the active locale."
  field voice String "Its voice clip (empty: none)."
  field seen bool "It had been read before it was shown."
  field chosen Option.ChosenView "The choice made on it, if any."
}
```

<sub>`api.ogt` line 14119</sub>

<a id="type-BarkDesc"></a>

### `BarkDesc` (desc struct)

A bark to ask for.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Who barks: its og.story.barks entry, `self.` in the bark's conditions and arguments. |
| `topic` | `string` | `""` | — | — | — | The topic: one of the bark's `topic`s, such as idle or alarm. |
| `dialogue` | `string` | `""` | — | — | — | Only this dialogue's barks (empty: every loaded dialogue's). |

```ogt
struct BarkDesc {
  doc "A bark to ask for."
  desc true
  field entity Entity "Who barks: its og.story.barks entry, `self.` in the bark's conditions and arguments."
  field topic String "The topic: one of the bark's `topic`s, such as idle or alarm."
  field dialogue String "Only this dialogue's barks (empty: every loaded dialogue's)."
}
```

<sub>`api.ogt` line 14132</sub>

<a id="type-AmbientDesc"></a>

### `AmbientDesc` (desc struct)

An entity that barks on its own.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Who barks. |
| `topic` | `string` | `""` | — | — | — | The topic it asks for. |
| `dialogue` | `string` | `""` | — | — | — | Only this dialogue's barks (empty: any). |
| `every` | `number` | `600` | — | — | — | Ticks between asks (0 stops it). |
| `jitter` | `number` | `0` | — | — | — | Up to this many ticks more each time, drawn on the story's stream. |

```ogt
struct AmbientDesc {
  doc "An entity that barks on its own."
  desc true
  field entity Entity "Who barks."
  field topic String "The topic it asks for."
  field dialogue String "Only this dialogue's barks (empty: any)."
  field every u32 "Ticks between asks (0 stops it)." {
    default 600
  }
  field jitter u32 "Up to this many ticks more each time, drawn on the story's stream."
}
```

<sub>`api.ogt` line 14140</sub>

<a id="type-BarkView"></a>

### `BarkView` (struct)

A bark being said.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Who says it. |
| `dialogue` | `string` | `""` | — | — | — | Its dialogue. |
| `bark` | `string` | `""` | — | — | — | Its id. |
| `seq` | `number` | `0` | — | — | — | The line's number: a new number is a new bark. |
| `key` | `string` | `""` | — | — | — | The line's i18n key. |
| `args` | `unknown` | `null` | — | — | — | The arguments it was resolved with (evaluated when it was said, world reads included). |
| `text` | `string` | `""` | — | — | — | The line, resolved in the active locale. |
| `speaker` | `SpeakerView` | `{ id: "", name: "", entity: 0, portrait: "", expression: "", side: "" }` | — | — | — | The bark's speaker (entity: who says it). |
| `voice` | `string` | `""` | — | — | — | Its voice clip (empty: none), played as a conversation's. |
| `voiceLeft` | `number` | `0` | — | — | — | Ticks of its voice still to play. |
| `ticksLeft` | `number` | `0` | — | — | — | Ticks it still shows. |
| `priority` | `number` | `0` | — | — | — | Its priority: only a higher one interrupts it. |

```ogt
struct BarkView {
  doc "A bark being said."
  field entity Entity "Who says it."
  field dialogue String "Its dialogue."
  field bark String "Its id."
  field seq u32 "The line's number: a new number is a new bark."
  field key String "The line's i18n key."
  field args Json "The arguments it was resolved with (evaluated when it was said, world reads included)."
  field text String "The line, resolved in the active locale."
  field speaker SpeakerView "The bark's speaker (entity: who says it)."
  field voice String "Its voice clip (empty: none), played as a conversation's."
  field voiceLeft u32 "Ticks of its voice still to play."
  field ticksLeft u32 "Ticks it still shows."
  field priority i32 "Its priority: only a higher one interrupts it."
}
```

<sub>`api.ogt` line 14152</sub>

<a id="type-MarkerDesc"></a>

### `MarkerDesc` (desc struct)

Where a quest's objective is, for the map, compass and tracker.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | An entity to point at (0: none). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | A point to point at, when there is no entity. |

```ogt
struct MarkerDesc {
  doc "Where a quest's objective is, for the map, compass and tracker."
  desc true
  field entity Entity "An entity to point at (0: none)."
  field position Vector3 "A point to point at, when there is no entity." {
    unit m
  }
}
```

<sub>`api.ogt` line 14168</sub>

<a id="type-MarkerView"></a>

### `MarkerView` (struct)

Where a quest points: the marker og.story.mark set, else the current stage's first undone objective's `marker`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | An entity (0: none). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | A point. |

```ogt
struct MarkerView {
  doc "Where a quest points: the marker og.story.mark set, else the current stage's first undone objective's `marker`."
  field entity Entity "An entity (0: none)."
  field position Vector3 "A point." {
    unit m
  }
}
```

<sub>`api.ogt` line 14177</sub>

<a id="type-RewardView"></a>

### `RewardView` (struct)

A quest's reward.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name, resolved in the active locale. |
| `amount` | `number` | `0` | — | — | — | How many. |
| `image` | `string` | `""` | — | — | — | Its image's asset path (empty: none). |
| `rarity` | `string` | `""` | — | — | — | The game's rarity word (empty: none). |

```ogt
struct RewardView {
  doc "A quest's reward."
  field name String "Its name, resolved in the active locale."
  field amount f64 "How many."
  field image String "Its image's asset path (empty: none)."
  field rarity String "The game's rarity word (empty: none)."
}
```

<sub>`api.ogt` line 14185</sub>

<a id="type-ObjectiveView"></a>

### `ObjectiveView` (struct)

An objective of a quest's current stage.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id. |
| `text` | `string` | `""` | — | — | — | Its text, resolved in the active locale with `count` and `target` among its arguments. |
| `count` | `number` | `0` | — | — | — | Counted so far. |
| `target` | `number` | `0` | — | — | — | Counts needed. |
| `done` | `boolean` | `false` | — | — | — | count reached target. |
| `failed` | `boolean` | `false` | — | — | — | Left undone when the quest failed. |
| `optional` | `boolean` | `false` | — | — | — | Not needed to finish the stage. |
| `hidden` | `boolean` | `false` | — | — | — | Not revealed yet: declared hidden and not yet counted (the log leaves it out or shows ???). |

```ogt
struct ObjectiveView {
  doc "An objective of a quest's current stage."
  field id String "Its id."
  field text String "Its text, resolved in the active locale with `count` and `target` among its arguments."
  field count u32 "Counted so far."
  field target u32 "Counts needed."
  field done bool "count reached target."
  field failed bool "Left undone when the quest failed."
  field optional bool "Not needed to finish the stage."
  field hidden bool "Not revealed yet: declared hidden and not yet counted (the log leaves it out or shows ???)."
}
```

<sub>`api.ogt` line 14193</sub>

<a id="type-QuestView"></a>

### `QuestView` (struct)

A quest as a quest log shows it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id. |
| `title` | `string` | `""` | — | — | — | Its title, resolved in the active locale. |
| `summary` | `string` | `""` | — | — | — | The current stage's summary, else the quest's, resolved. |
| `category` | `string` | `""` | — | — | — | main, side, daily... (the log's tabs). |
| `stage` | `string` | `""` | — | — | — | The current stage's id. |
| `status` | `string` | `""` | — | — | — | active, completed or failed. |
| `tracked` | `boolean` | `false` | — | — | — | Shown in the HUD tracker (og.story.track; the first quest started is tracked). |
| `isNew` | `boolean` | `false` | — | — | — | Not looked at since it started (og.story.markSeen clears it). |
| `updated` | `boolean` | `false` | — | — | — | Changed since it was last looked at (og.story.markSeen clears it). |
| `updatedTick` | `number` | `0` | — | — | — | The tick it last changed (sort the log by recent). |
| `giver` | `string` | `""` | — | — | — | Who gave it, resolved (empty: nobody). |
| `region` | `string` | `""` | — | — | — | Where it is, resolved (empty: nowhere). |
| `marker` | `MarkerView \| null` | `null` | — | — | — | Where it points now. |
| `rewards` | `readonly RewardView[]` | `[]` | — | — | — | Its rewards. |
| `objectives` | `readonly ObjectiveView[]` | `[]` | — | — | — | The current stage's objectives. |

```ogt
struct QuestView {
  doc "A quest as a quest log shows it."
  field id String "Its id."
  field title String "Its title, resolved in the active locale."
  field summary String "The current stage's summary, else the quest's, resolved."
  field category String "main, side, daily... (the log's tabs)."
  field stage String "The current stage's id."
  field status String "active, completed or failed."
  field tracked bool "Shown in the HUD tracker (og.story.track; the first quest started is tracked)."
  field isNew bool "Not looked at since it started (og.story.markSeen clears it)."
  field updated bool "Changed since it was last looked at (og.story.markSeen clears it)."
  field updatedTick u32 "The tick it last changed (sort the log by recent)."
  field giver String "Who gave it, resolved (empty: nobody)."
  field region String "Where it is, resolved (empty: nowhere)."
  field marker Option.MarkerView "Where it points now."
  field rewards List.RewardView "Its rewards."
  field objectives List.ObjectiveView "The current stage's objectives."
}
```

<sub>`api.ogt` line 14205</sub>
