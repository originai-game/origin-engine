---
name: og-story
description: Dialogue and quests on Origin Engine with og.story — dialogue graphs and quests as .ogt data (speakers, lines as og.i18n keys, choices with conditions, reasons and d20 checks, variables, weighted picks, timers; quest stages, objectives, rewards), a dialogue box and quest log fed by og.story.view and og.story.quests, and saves that survive edits. Use when a game has talking characters, branching conversations, a visual novel, quests or a quest log.
---

A conversation on Origin Engine is data, not code: a dialogue in
`assets/story/<name>.ogt` (`type dialogue`), its text in the string
tables (`assets/strings/<locale>.ogt`, og-i18n), and a few calls. The
game never walks a graph by hand; it starts a conversation, draws what
`og.story.view` answers each frame, and calls `og.story.advance` or
`og.story.choose`. Quests are the same: a `type quest` file, counted with
`og.story.progress`, listed with `og.story.quests`. The API is in
`og api show story`; this skill is how to use it.

## 1. The formats

A dialogue. `og cook` validates it and names each mistake's line and the
fix; every quoted string after `say`, `choice`, `reason` and `name` is an
og.i18n key, never the text itself, and the cook warns about a key the
base table lacks or an argument its message takes that nothing passes.

```text
type dialogue
version 1
---
name smith
start greet
speaker smith {
  name "npc.smith"
  portrait "art/smith.png"
  side right
}
var mood 0
global gold 10
global charm 3
node greet {
  say smith "smith.hello" {
    arg gold "gold"
    voice "vo/smith_hello"
  }
  choice "smith.buy" {
    if "gold >= 10"
    reason "smith.need_gold" {
      arg need "10 - gold"
    }
    kind trade
    set gold "gold - 10"
    goto bought
  }
  choice "smith.haggle" {
    check active charm 12
    goto bought
    fail angry
  }
  choice "smith.leave"
  timer 600 smith.leave
}
node bought {
  say smith "smith.thanks"
  quest progress rescue sword
  emit give_sword
}
node angry {
  pick {
    greet 1
    bought 3
  }
}
```

- A node runs its steps in order. `say` shows a line and waits for
  `advance`; `set`, `emit` and `quest` act at once; `if "cond" node`,
  `goto`, `pick` and `end` jump. A node's choices show with its last line.
  A choice without `goto`, or a node that runs out without choices, ends
  the conversation.
- `var` is the conversation's own; `global` is the story's, shared by every
  dialogue and quest and kept in saves.
- Expressions are in quotes: numbers, `true`/`false`, `'strings'`,
  `+ - * / %`, `== != < <= > >=`, `and or not`, and `seen(node)`
  (`seen(other.node)` for another dialogue's), `status(quest)`,
  `stage(quest)`, `count(quest, objective)`.
- `check passive stat 5` enables a choice only when the stat reaches 5.
  `check active stat 12` rolls d20 + stat against 12 when chosen: `goto` on
  a pass, `fail` on a miss. `hide` hides a choice instead of disabling it.
- `timer <ticks> <choice id>`: the choice taken when time runs out. A
  choice's id is its key unless it has an `id` line.

A quest:

```text
type quest
version 1
---
id rescue
title "quest.rescue"
summary "quest.rescue.summary"
category side
giver "npc.smith"
global wolvesFled false
reward "item.gold" 50 {
  rarity common
}
stage hunt {
  objective wolves "quest.rescue.wolves" {
    target 3
    marker 12 0 4
  }
  objective sword "quest.rescue.sword" {
    optional
  }
  fail "wolvesFled"
  next report
}
stage report {
  objective talk "quest.rescue.talk" {
    when "seen(smith.bought)"
  }
}
```

A stage is done when every objective that is not `optional` reaches its
`target`; the quest moves to `next`, or completes after its last stage.
An objective's message gets `{count}` and `{target}` from the log.

## 2. Run a conversation

```ts
og.story.load(world, "story/smith.ogt");   // in setup: the first load registers og.Story
og.story.load(world, "story/rescue.ogt");
const conv = og.story.start(world, { dialogue: "smith", cast: { smith: smithEntity } });
// each frame, in the UI:
const v = og.story.view(world, conv.value);  // the full line, speaker, choices
// on input:
og.story.advance(world, conv.value);       // or og.story.choose(world, conv.value, choice.index)
```

- The view carries the whole resolved line; the dialogue box paces its own
  reveal. A new `seq` is a new line, even when a node repeats; `seen` marks
  a line read on an earlier playthrough (skip-read).
- A disabled choice's `reason` is resolved from its key and arguments;
  draw `visited` choices muted and an active check's `chance` as a percent.
- Events arrive the tick after: og.DialogueShown (start the voice),
  og.DialogueCheck (reveal the roll), og.StoryEvent (the dialogue's `emit`:
  open the shop, give the item), og.DialogueEnded, og.QuestUpdated.
- `og.story.history` is the VN backlog, with the choice made on each line.
- `og.i18n.setLocale` re-translates the line on screen; nothing else moves.

## 3. Quests and the log

`og.story.startQuest`, `og.story.progress(world, "rescue", "wolves")`,
`og.story.quests` for the log (start order), `og.story.track` for the
HUD tracker, `og.story.markSeen` when the player opens it, and
`og.story.mark` to point the map at an entity.

## 4. Saves and determinism

The story is world state: saves, restores and the world hash carry it,
by ids (dialogue, node, line key, quest, stage, objective). Any edit that
keeps those ids is safe for old saves. Renaming the node, stage or
objective a save stands on is the one breaking edit: og.story logs a
warning naming the id and falls back (a conversation to its dialogue's
`start`, a quest to its first stage). `pick` and active checks draw from
og.Rng's fork "story", and timers count ticks, so a seed replays the same
conversation on every host.
