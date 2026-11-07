<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `story` module

Dialogue uses released Yarn Spinner 3.2.2 in `.yarn` files, compiled to engine-owned IR by og cook. Declare `#og-profile:yarn-spinner-3.2.2`; see the yarn guide for the language and engine extensions. Every line and option requires an explicit stable `#line:` id. Base text enters the string catalog under that id; og.story resolves through og.i18n on every read, so switching locale re-translates the current line and choices. Quests remain `type quest` data, with stages, reflected event objectives and rewards.

og.story.load a dialogue or quest in setup, og.story.start a conversation, read og.story.view each frame, and call og.story.advance or og.story.choose. Dialogue text retains its Speaker: prefix and carries typed TextSpan attributes in Unicode scalar offsets. Quests start with `<<quest start rescue>>` or og.story.startQuest, count with og.story.progress and advance stages themselves; og.story.quests lists them for a quest log.

Barks use Yarn nodes with `og-topic`, `og-when`, `og-priority`, `og-weight`, `og-cooldown`, `og-shared` and `og-once` headers. The engine scheduler still owns priority, cooldowns and once state. og.story.bark asks for one, og.story.ambient asks periodically, and og.story.barks lists those being said. A line's `#voice:` plays through og.audio. Quest objectives with `on Event` and `where` count reflected events without moving quest data into Yarn.

Saves checkpoint stable continuation, node, line and option ids, detour stacks, timers, variables and the story RNG fork. They never store bytecode offsets. Inserting or moving unrelated lines preserves the current line. A missing saved continuation is an error, never a restart that could repeat a command or reward. `=>` line groups prefer eligible conditional lines over unconditional fallbacks and draw ties by weight; line groups, barks and active checks use og.Rng's story fork on both hosts. Timers count ticks. Worlds without loaded stories keep their existing saves and hash.

- **TypeScript**: functions are the namespace `og.story`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module story {
  doc "Dialogue uses released Yarn Spinner 3.2.2 in `.yarn` files, compiled to engine-owned IR by og cook. Declare `#og-profile:yarn-spinner-3.2.2`; see the yarn guide for the language and engine extensions. Every line and option requires an explicit stable `#line:` id. Base text enters the string catalog under that id; og.story resolves through og.i18n on every read, so switching locale re-translates the current line and choices. Quests remain `type quest` data, with stages, reflected event objectives and rewards."
  doc "og.story.load a dialogue or quest in setup, og.story.start a conversation, read og.story.view each frame, and call og.story.advance or og.story.choose. Dialogue text retains its Speaker: prefix and carries typed TextSpan attributes in Unicode scalar offsets. Quests start with `<<quest start rescue>>` or og.story.startQuest, count with og.story.progress and advance stages themselves; og.story.quests lists them for a quest log."
  doc "Barks use Yarn nodes with `og-topic`, `og-when`, `og-priority`, `og-weight`, `og-cooldown`, `og-shared` and `og-once` headers. The engine scheduler still owns priority, cooldowns and once state. og.story.bark asks for one, og.story.ambient asks periodically, and og.story.barks lists those being said. A line's `#voice:` plays through og.audio. Quest objectives with `on Event` and `where` count reflected events without moving quest data into Yarn."
  doc "Saves checkpoint stable continuation, node, line and option ids, detour stacks, timers, variables and the story RNG fork. They never store bytecode offsets. Inserting or moving unrelated lines preserves the current line. A missing saved continuation is an error, never a restart that could repeat a command or reward. `=>` line groups prefer eligible conditional lines over unconditional fallbacks and draw ties by weight; line groups, barks and active checks use og.Rng's story fork on both hosts. Timers count ticks. Worlds without loaded stories keep their existing saves and hash."
  # declarations follow, each on its own page
```

<sub>Module `story` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`StorySaliency`](story/type-StorySaliency.md) | enum |  | How Yarn node and line groups choose eligible content. |
| [`og.story.setSaliency`](story/fn-setSaliency.md) | function | `(world: World, strategy: StorySaliency): Result<void>` | Select this world's Yarn content strategy after loading its story. |
| [`og.Story`](story/record-Story.md) | resource |  | The story's counters (written by og.story; the story itself is a state block). |
| [`og.DialogueShown`](story/record-DialogueShown.md) | event |  | A conversation shows a new line (start the voice, reset the reveal). |
| [`og.DialogueEnded`](story/record-DialogueEnded.md) | event |  | A conversation ended: its top-level Yarn node returned, reached `<<stop>>`, or og.story.stop stopped it. |
| [`og.DialogueCheck`](story/record-DialogueCheck.md) | event |  | An active check was rolled (d20 plus the stat against the target): reveal the roll. |
| [`og.QuestUpdated`](story/record-QuestUpdated.md) | event |  | A quest started, counted, changed stage, completed or failed. |
| [`og.BarkSaid`](story/record-BarkSaid.md) | event |  | An entity barked (og.story.bark or og.story.ambient): show its line over it with og.story.barks. |
| [`og.StoryEvent`](story/record-StoryEvent.md) | event |  | A Yarn command such as `<<give_sword {$amount}>>`: open the shop, give the sword, play the cutscene. |
| [`StoryAsset`](story/type-StoryAsset.md) | struct |  | A loaded dialogue or quest. |
| [`StartDesc`](story/type-StartDesc.md) | desc struct |  | A conversation to start. |
| [`SpeakerView`](story/type-SpeakerView.md) | struct |  | Who speaks a line. |
| [`CheckView`](story/type-CheckView.md) | struct |  | A choice's check. |
| [`ChoiceView`](story/type-ChoiceView.md) | struct |  | A choice the node offers (hidden ones are left out). |
| [`TimerView`](story/type-TimerView.md) | struct |  | A node's choice timer. |
| [`RollView`](story/type-RollView.md) | struct |  | An active check's roll. |
| [`DialogueView`](story/type-DialogueView.md) | struct |  | What a conversation shows now: the line in full (the dialogue box paces its own reveal), its speaker and the choices, if it waits on them. |
| [`ChosenView`](story/type-ChosenView.md) | struct |  | The choice made while a line was shown. |
| [`StoryLine`](story/type-StoryLine.md) | struct |  | A line of a conversation's backlog. |
| [`BarkDesc`](story/type-BarkDesc.md) | desc struct |  | A bark to ask for. |
| [`AmbientDesc`](story/type-AmbientDesc.md) | desc struct |  | An entity that barks on its own. |
| [`BarkView`](story/type-BarkView.md) | struct |  | A bark being said. |
| [`MarkerDesc`](story/type-MarkerDesc.md) | desc struct |  | Where a quest's objective is, for the map, compass and tracker. |
| [`MarkerView`](story/type-MarkerView.md) | struct |  | Where a quest points: the marker og.story.mark set, else the current stage's first undone objective's `marker`. |
| [`RewardView`](story/type-RewardView.md) | struct |  | A quest's reward. |
| [`ObjectiveView`](story/type-ObjectiveView.md) | struct |  | An objective of a quest's current stage. |
| [`QuestView`](story/type-QuestView.md) | struct |  | A quest as a quest log shows it. |
| [`og.story.load`](story/fn-load.md) | function | `(world: World, path: string): Result<StoryAsset>` | A cooked `.yarn` dialogue or `type quest` asset. |
| [`og.story.parse`](story/fn-parse.md) | function | `(world: World, text: string): Result<StoryAsset>` | A dialogue or quest from text, now (tests and generated content; assets go through og cook). |
| [`og.story.start`](story/fn-start.md) | function | `(world: World, desc?: StartDesc): Result<number>` | Start a conversation: it runs to its first line (or its choices) now, and og.DialogueShown is sent the next tick. |
| [`og.story.view`](story/fn-view.md) | function | `(world: World, conversation: number): DialogueView \| null` | The current dialogue line and choices, with text and markup spans resolved in the active locale. |
| [`og.story.history`](story/fn-history.md) | function | `(world: World, conversation: number): readonly StoryLine[]` | A conversation's lines so far, oldest first, with the choice made on each (a VN backlog). |
| [`og.story.conversations`](story/fn-conversations.md) | function | `(world: World): readonly number[]` | The conversations that exist (started and not stopped), oldest first. |
| [`og.story.advance`](story/fn-advance.md) | function | `(world: World, conversation: number): Result<void>` | Move a conversation past its line: it runs to the next line, its choices or its end. |
| [`og.story.choose`](story/fn-choose.md) | function | `(world: World, conversation: number, index: number): Result<void>` | Take a choice: an active check rolls (og.DialogueCheck), then its indented Yarn body runs, or #fail redirects a missed roll. |
| [`og.story.stop`](story/fn-stop.md) | function | `(world: World, conversation: number): boolean` | End a conversation now (og.DialogueEnded if it was running) and forget it. |
| [`og.story.bark`](story/fn-bark.md) | function | `(world: World, desc?: BarkDesc): Result<void>` | Ask an entity to bark a line of a topic: at the og.story system's next run, of that topic's barks whose `when` holds (it may read the world: `self.Health.hp`, `Game.alarm`) and whose cooldowns are ov… |
| [`og.story.ambient`](story/fn-ambient.md) | function | `(world: World, desc?: AmbientDesc): Result<void>` | Let an entity bark a topic on its own, every `every` ticks (and up to `jitter` more), as og.story.bark would; `every` 0 stops it. |
| [`og.story.barks`](story/fn-barks.md) | function | `(world: World): readonly BarkView[]` | The barks being said, by entity: draw each over its entity (a speech bubble) until it ends. |
| [`og.story.get`](story/fn-get.md) | function | `(world: World, name: string): unknown` | A story variable (a `global` of any loaded dialogue or quest): its value, else its declared default. |
| [`og.story.set`](story/fn-set.md) | function | `(world: World, name: string, value: unknown): Result<void>` | Set a story variable; quests whose conditions now hold advance. |
| [`og.story.reset`](story/fn-reset.md) | function | `(world: World): void` | A new playthrough: the story's variables, seen nodes, conversations and quests are cleared. |
| [`og.story.startQuest`](story/fn-startQuest.md) | function | `(world: World, id: string): Result<boolean>` | Start a loaded quest at its first stage. |
| [`og.story.progress`](story/fn-progress.md) | function | `(world: World, quest: string, objective: string, amount?: number): boolean` | Count toward an objective of an active quest's current stage (a wolf killed, a herb picked); the stage advances when its objectives are done. |
| [`og.story.setStage`](story/fn-setStage.md) | function | `(world: World, quest: string, stage: string): Result<boolean>` | Move an active quest to a stage, its counts cleared. |
| [`og.story.completeQuest`](story/fn-completeQuest.md) | function | `(world: World, quest: string): Result<boolean>` | Complete an active quest now. |
| [`og.story.failQuest`](story/fn-failQuest.md) | function | `(world: World, quest: string): Result<boolean>` | Fail an active quest now: its undone objectives are marked failed. |
| [`og.story.quest`](story/fn-quest.md) | function | `(world: World, id: string): QuestView \| null` | A started quest as the log shows it. |
| [`og.story.quests`](story/fn-quests.md) | function | `(world: World): readonly QuestView[]` | Every started quest, in the order they started (a quest log). |
| [`og.story.track`](story/fn-track.md) | function | `(world: World, id: string, on: boolean): boolean` | Show a quest in the HUD tracker, or stop showing it. |
| [`og.story.markSeen`](story/fn-markSeen.md) | function | `(world: World, id: string): boolean` | The player looked at a quest: clear QuestView.isNew and updated. |
| [`og.story.mark`](story/fn-mark.md) | function | `(world: World, id: string, desc?: MarkerDesc): boolean` | Point a quest at an entity or a point (the map, compass and tracker read QuestView.marker), over its objectives' own markers. |
