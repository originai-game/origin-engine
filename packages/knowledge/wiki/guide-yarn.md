<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `yarn` guide: Author dialogue in Yarn

Put dialogue in `assets/story/*.yarn`, then run `og cook`. Load the asset
with `og.story.load(world, "story/smith.yarn")`. The cook compiles Yarn to
engine-owned IR. The runtime, saves, quests, and UI views remain engine-owned.

<a id="guide-yarn-1"></a>
### `yarn §1` Declare the profile and stable ids

Every file declares `#og-profile:yarn-spinner-3.2.2` and a unique `#og-name`.
The language is pinned to released Yarn Spinner **v3.2.2**, not development
main. The Rust compiler and runtime implement that language inside the engine.
Every spoken line, narrator line, option, and line-group member requires an
explicit `#line:` id. Keep these ids when editing or moving text. A missing
or duplicate id is a cook error with its source line and the fix.

```yarn
#og-profile:yarn-spinner-3.2.2
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

Declarations accept `as number`, `as string`, `as bool`, or an enum type.
A nonconstant declaration defines a smart variable, evaluated on each read;
smart variables cannot be assigned and their dependency graph must be acyclic.
Use `<<enum Mode>>`, `<<case Open>>`, and `<<endenum>>` for enums, with optional
constant string or numeric raw values. Refer to `Mode.Open`, or `.Open` where
its type is unambiguous.

Declarations are global. Add `#og-local:name` before the first node for a
conversation-local variable. Assignment accepts `=`, `to`, `+=`, `-=`, `*=`,
`/=`, and `%=`. Expressions support Yarn's arithmetic, comparisons, boolean
operators and aliases, and function calls. Numbers follow Yarn's single-precision
semantics on both hosts. Engine functions `seen`, `status`, `stage`, and `count`
remain available. World reads are limited to bark conditions and arguments.

Dynamic transfers use `<<jump {$destination}>>` and `<<detour {$destination}>>`.
Use `<<once>>` or `<<once if expression>>` blocks with `<<endonce>>`, and
`<<once>>` or `<<once if expression>>` conditions on lines and options.
Optional `<<else>>` content runs after a once block has been consumed.
`<<call function(...)>>` remains supported because it exists in v3.2.2.

Other commands emit `og.StoryEvent` once, with the command name and its
formatted argument text. Any number of `{expression}` substitutions can appear
between literal fragments. The game handles the
event on the next tick. The reserved `quest` command accepts `start`,
`complete`, `fail`, `stage`, and `progress`; it calls the engine quest runtime.

<a id="guide-yarn-2"></a>
### `yarn §2` Keep translations separate from saves

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
Repeated identical commands are allowed in one node. Their occurrence within
that scope disambiguates their instruction ids. Inserting unrelated lines does
not change those ids. Changing command text or inserting another identical
command before a saved return point can change its identity. This version does
not migrate old `type dialogue` saves.

Offered option conditions and arguments are evaluated once and checkpointed.
Reading a view or restoring cannot draw another random number or settle the
same choice again. History retains the selected option's evaluated arguments.

<a id="guide-yarn-3"></a>
### `yarn §3` Select lines and schedule barks

Use `=>` for a line group. Repeat a `title:` with `when:` headers to form a
node group. Each member can have a stable `subtitle:`. Multiple `when:` headers
are alternatives. `when: always`, `when: once`, and `when: once if expression`
are supported. Keep subtitles stable when editing group members used by saves.

Select a strategy with `og.story.setSaliency(world, strategy)`:

- `first`: first eligible member.
- `best`: highest condition complexity, then source order.
- `bestLeastRecentlyViewed`: lowest view count, then highest complexity,
  then source order.
- `randomBestLeastRecentlyViewed`: lowest view count, then highest complexity,
  then a random tie break.
- `conditionalFirstWeighted` (engine default): conditional members before
  unconditional fallbacks, then positive `#weight` values, defaulting to 1.

The first four implement upstream's built-in strategies. Every random choice
uses `og.Rng`'s `story` fork. The strategy, view counts, and RNG are saved.

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

<a id="guide-yarn-4"></a>
### `yarn §4` Keep quests in engine data

Quests remain `.ogt` files of `type quest`. Objectives retain reflected
`on Event` and `where "event.field == 'value'"` filters, progression, rewards,
and saves. QuestView does not change. Dialogue views retain their existing
fields and add typed markup spans.

<a id="guide-yarn-5"></a>
### `yarn §5` Deliver localized text and spans

Dialogue, choices, selected choices, history, and barks carry plain `text` and
`spans`. Each root API `TextSpan` has `name`, `position`, `length`, and
`properties: Readonly<Record<string, string | number | boolean>>`.
Positions and lengths count **Unicode scalar values**, including the complete
`Speaker: ` prefix. They are not UTF-16 offsets. The character span covers that
prefix and stores the character's name in `properties.name`.

The engine resolves `[nomarkup]`, escapes, `[select]`, `[plural]`, and `[ordinal]`
after localization. Plural and ordinal selection uses og.i18n's CLDR rules.
Unknown attributes and typed properties are preserved. UI consumers receive
the spans directly, never markup to parse again. A UI document binds
`<text spans={$spans}>{$body}</text>`; its styles decide how attributes such as
`bold` or `wave` look. This compiler does not implement visual effects.

<a id="guide-yarn-6"></a>
### `yarn §6` Call built-in and registered functions

The built-ins are `visited`, `visited_count`, `has_any_content`, `random`,
`random_range`, `random_range_float`, `dice`, `min`, `max`, `round`,
`round_places`, `floor`, `ceil`, `inc`, `dec`, `decimal`, `int`, `number`,
`string`, `bool`, `format`, and `format_invariant`.

Visit counts increase when source nodes exit, not at each generated instruction.
Literal `visited("Node")` and `visited_count("Node")` references enable tracking.
Set the Yarn `tracking:` header to `always` for dynamic references, or `never`
to suppress tracking.
For compatibility, v3.2.2's `random_range_float` draws an integer displacement
and adds the floating minimum. Numeric `format` uses invariant .NET-compatible
standard and custom numeric formats. It does not depend on the host locale.

Register pure typed functions in `project.ogt` before cooking:

```text
storyFunctions {
	function scaled number {
		param value number
		param factor number
		body "$value * $factor + 2"
	}
}
```

Then use `{scaled(3, 7)}` in Yarn. The cook type-checks calls and stores checked
expression bodies in the IR. Parameters and return values use `number`,
`string`, or `bool`. Functions may call other pure functions, but cannot read
story variables, RNG, quests, clocks, or host callbacks. Recursive calls fail
cooking. The Rust registration entry is `yarn::compile_with_functions`.

<a id="guide-yarn-7"></a>
### `yarn §7` Upgrade the declared release

The conformance inputs and MIT license are under `third_party/yarn-spinner/`.
All 83 upstream `.yarn` files and 33 `.testplan` files come from v3.2.2.
The harness adds mandatory engine line ids only in memory, checks runtime
output and saved-state hashes on native and wasm, and restores at suspensions.
Markup expectations are converted from upstream UTF-16 to engine scalar offsets.

Legacy Yarn 1 `[[option|destination]]` links are not part of the released
[v3.2.2 grammar](https://github.com/YarnSpinnerTool/YarnSpinner/blob/v3.2.2/YarnSpinner.Compiler/Grammars/YarnSpinnerParser.g4).
Use `->` options and `<<jump>>`. No v3.2.2 feature is excluded because a later
development version removes it. Unknown engine profile versions fail cooking;
arbitrary Yarn headers and hashtags remain metadata.

Replace old `type dialogue` sources with `.yarn`; keep their localization
keys as `#line:` ids and move the base text into the script. Re-cook all
dialogue assets after changing the profile or registered functions.

<a id="guide-yarn-8"></a>
### `yarn §8` Run dialogue, barks and quests from the game

```ts
// setup
og.story.load(world, "story/smith.yarn");
// on talking to the smith
const c = og.story.start(world, { dialogue: "smith" });
// each frame: read the view, then answer it
const v = c.ok ? og.story.view(world, c.value) : null;
if (v && v.choices.length === 0) og.story.advance(world, c.value);
else if (v) og.story.choose(world, c.value, v.choices[0].index);
```

- og.story resolves every line through og.i18n on every read, so a
  locale switch re-translates the current line and choices (§2). Dialogue
  text keeps its `Speaker:` prefix and carries typed spans (§5).
- A line's `#voice:` plays through og.audio on the conversation's own
  entity, an og.AudioSource on the sfx bus, from the tick after the line
  is shown; it stops when the line moves on. For auto-advance, advance
  once the view's `voiceTicksLeft` reaches 0.
- A line's `#shot:` (else its node's) names a virtual camera's entity or a
  cast role. While the line shows, og.story makes that og.VirtualCamera
  og.CameraBrain.solo (with `soloBlend` -1): a name is the entity of that
  name, a role the cast entity's own og.VirtualCamera, else the first one
  whose lookAt is that entity. When no running conversation's line has a
  shot, the game's own solo is handed back (`camera §2`).
- Barks: og.story.bark asks for one of a topic, og.story.ambient asks
  periodically, og.story.barks lists those being said, to draw over their
  entities.
- Quests start with `<<quest start rescue>>` in Yarn or og.story.startQuest,
  count with og.story.progress (or `on Event` objectives) and advance their
  stages themselves; og.story.quests lists them for a quest log.
- Choice timers, bark durations and cooldowns count ticks, never wall time.
- A world without loaded stories keeps its existing saves and hash.
