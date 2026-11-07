# Author dialogue in Yarn

Put dialogue in `assets/story/*.yarn`, then run `og cook`. Load the asset
with `og.story.load(world, "story/smith.yarn")`. The cook compiles Yarn to
engine-owned IR. The runtime, saves, quests, and UI views remain engine-owned.

## 1. Declare the profile and stable ids

Every file declares `#og-profile:yarn-3.0-og-1` and a unique `#og-name`.
This is a bounded Yarn Spinner 3.0 syntax profile, not the upstream runtime.
Every spoken line, narrator line, option, and line-group member requires an
explicit `#line:` id. Keep these ids when editing or moving text. A missing
or duplicate id is a cook error with its source line and the fix.

```yarn
#og-profile:yarn-3.0-og-1
#og-name:smith
#og-locale:en
title: Start
---
<<declare $gold = 10>>
Smith: You have {$gold} gold. #line:smith.hello
-> Buy <<if $gold >= 10>> #line:smith.buy
    <<set $gold = $gold - 10>>
    <<give_sword>>
    <<detour Thanks>>
-> Leave #line:smith.leave
    <<stop>>
===
title: Thanks
---
Smith: Mind the edge. #line:smith.thanks
===
```

`title:` names a node; `---` begins its body and `===` ends it. The first
non-bark node is the default start. Indent an option's body beneath `->`.
Consecutive options form one choice group. A preceding line shows those
choices immediately, including any straight-line commands between them.

Use `<<if expression>>`, `<<elseif expression>>`, `<<else>>`, and `<<endif>>`
for branches. `<<jump Node>>` transfers control. `<<detour Node>>` saves a
return continuation; reaching the node's end or `<<return>>` resumes it.
`<<stop>>` ends the conversation. Without a detour caller, returning ends it.

Declare number, boolean, and string defaults with `<<declare $name = value>>`.
Declarations are global. Add `#og-local:name` before the first node for a
conversation-local variable. Assignment accepts `=` or `to`. Expressions
support arithmetic, comparisons, parentheses, `and`, `or`, `not`, `&&`,
`||`, and `!`. Engine functions `seen`, `status`, `stage`, and `count` remain
available. World reads are limited to bark conditions and bark arguments.

Other commands emit `og.StoryEvent` once, with the command name and one
argument: a raw text tail or a single `{expression}`. The game handles the
event on the next tick. The reserved `quest` command accepts `start`,
`complete`, `fail`, `stage`, and `progress`; it calls the engine quest runtime.

## 2. Keep translations separate from saves

The cook extracts base text under each `#line:` id. `#og-locale` defaults to
`en`. `og i18n extract` includes that text when writing missing base entries.
Explicit string-table entries override authored text. Translations use the
same keys, so switching locale re-translates the current line, choices,
barks, and history without advancing the story. `{$gold}` supplies the
argument `gold`; compound expressions supply `arg0`, `arg1`, and so on.

Saves use stable node, line, option, and continuation ids, not bytecode
offsets. They checkpoint detour stacks, timers, choice settlement, and the
story RNG stream. Inserting or moving unrelated lines does not move a saved
conversation. Missing saved ids cause an error instead of restarting and
possibly repeating a command or reward.

Instruction identities derive from the owning node or option and the
command text. Changing a pending detour command changes its return identity.
Keep that command and its owning node or option while saves may stand on it.
Identical commands in the same scope are rejected; put repeated commands in
distinct named nodes. This profile does not migrate old dialogue-format saves.

## 3. Select lines and schedule barks

Use `=>` for a line group. Matching conditional members take precedence over
unconditional fallback members. Ties draw by positive `#weight` (default 1)
from `og.Rng`'s `story` fork. This conditional-first policy is the profile's
saliency rule, not Yarn's pluggable saliency strategies.

```yarn
=> Smith: A loyal customer. <<if $gold > 100>> #line:smith.rich #weight:3
=> Smith: Welcome back. <<if $gold > 100>> #line:smith.welcome
=> Smith: Browse freely. #line:smith.browse
```

A bark is a Yarn node with `og-topic`. Its body contains lines and optional
`set`, event, or quest commands, not dialogue flow control. The engine still
owns bark priority, per-entity and shared cooldowns, once state, and ambient
scheduling. These states survive saves.

```yarn
title: Hurt
og-topic: idle
og-when: self.Health.hp < 30
og-priority: 5
og-weight: 1
og-cooldown: 120
og-shared: 30
og-duration: 180
og-once: false
og-order: random
---
Guard: I need help. #line:guard.hurt
===
```

Durations and cooldowns count ticks. `og-order` is `sequence` (default) or
`random`. Add `og-speaker: id|name-key|portrait-path|left` to a node header
to supply speaker metadata (`left`, `right`, or `center`). Speakers without
metadata are inferred from `Speaker: text`.

Line tags preserve presentation: `#voice:path`, `#expression:variant`, and
`#shot:camera-or-role`. Option tags preserve existing UI features:
`#kind:trade`, `#reason:i18n.key`, `#reason-arg:name=expression`, `#hide`,
`#check:passive:stat:target`, `#check:active:stat:target`, `#fail:Node`, and
`#timer:ticks`. An active check requires a failure target. One option per
group may carry the positive timeout; that option is the default choice.

## 4. Keep quests in engine data

Quests remain `.ogt` files of `type quest`. Objectives retain reflected
`on Event` and `where "event.field == 'value'"` filters, progression, rewards,
and saves. DialogueView and QuestView do not change. No UI migration is needed.

## 5. Stay within the declared profile

Unsupported features fail cooking instead of silently becoming executable
dialogue: node groups, dynamic jump or detour targets, smart variables,
typed declarations, enums, once blocks, markup, legacy `[[options]]`,
compound assignments, custom expression functions, and mixed command
interpolation. Full-line `//` comments are supported. Node `tags` are
metadata only. Unknown headers, line tags, or version profiles are errors.

Replace old `type dialogue` sources with `.yarn`; keep their localization
keys as `#line:` ids and move the base text into the script. Re-cook all
dialogue assets. The compiler's upstream syntax references and MIT license
are recorded under `third_party/yarn-spinner/`.
