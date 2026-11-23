<!-- Generated from the built-in components' source (packages/engine/src/ui/components/index.tsx and the files it exports) by `og api gen`. Do not edit: `og api check` fails on any difference. -->
# The built-in UI components

Every component, control and screen the engine ships as TSX, one section each: what it draws, its props (from its `<Name>Props` interface, each with its type and what it means; a `?` prop may be left out and has its default in the component), the row types those props name, the pack slots it dresses, its fixtures and the input actions it answers.

Import one by name from `@origingame/engine/ui/components`, or by its path (`@origingame/engine/ui/components/hud/stat-bar`), and mount it or render it inside your own component. Each draws what the game hands it and reports intents through its `on*` props; it never decides costs, rules or rewards. A style pack restyles a component through the slots it names (`packClass("Plate", "frame")` in its class) and the theme's roles; to change one beyond that, copy its file (and its string table) into the game and import the copy. The `screens` guide holds the rules a screen keeps (layout, contrast, grounds, input).

A component marked as a piece of a panel is drawn on a panel's ground (inside a `Panel.frame`), never straight over the scene.

By kit (the directory a component is filed in):

- `arena`: `ArenaDraft`, `ArenaLobby`, `ArenaReady`, `ArenaRoster`, `ArenaRoundHud`, `ArenaScoreboard`, `ArenaSpectator`, `ArenaVersus`, `ArenaVote`
- `base`: `Avatar`, `Currency`, `EmptyState`, `ErrorState`, `ItemSlot`, `ListItem`, `Pips`, `Ribbon`, `SkeletonGroup`, `Spinner`, `Steps`
- `card`: `CardCollection`, `CardDeckList`, `CardFace`, `CardHand`, `CardPiles`, `CardReward`
- `hud`: `AbilityBar`, `AbilitySlot`, `ActionButton`, `ActionHints`, `AmmoCounter`, `HudBar`, `BossBar`, `Celebration`, `RpgCombatLog`, `Combo`, `Cooldown`, `Crosshair`, `DamageDirection`, `DamageNumbers`, `HudEffect`, `HeroButton`, `HudHint`, `HoldButton`, `HudTimer`, `Killfeed`, `HudLootEntry`, `OrbMeter`, `OverlayBar`, `PerfReadout`, `PickupFeed`, `HudPortrait`, `ProgressBar`, `ProgressRing`, `QtePrompt`, `RpgQuestTracker`, `RadialMenu`, `RadialMeter`, `HudSlot`, `StarBar`, `StatBar`, `StatusGrid`, `TouchControls`
- `idle`: `IdleGenerators`, `IdleOffline`, `IdlePrestige`, `IdlePurchases`, `IdleResources`
- `meta`: `Achievements`, `Chat`, `DailyQuests`, `DailyRewards`, `Friends`, `Inbox`, `News`, `PassTrack`, `Profile`, `Reveal`, `ShopConfirm`, `Shop`
- `panels`: `RpgBackpack`, `Breadcrumbs`, `Carousel`, `RpgConversationBacklog`, `RpgConversation`, `RpgCrafting`, `DataGrid`, `RpgItemCard`, `RpgLoot`, `Menu`, `RpgQuestLog`, `SplitView`, `TooltipCard`, `RpgTrade`
- `puzzle`: `PuzzleBoosters`, `PuzzleGoals`, `PuzzleLevelPath`, `PuzzleSummary`
- `racing`: `RacingBanner`, `RacingBonus`, `RacingBoost`, `RacingCar`, `RacingGarage`, `RacingLapHud`, `RacingPedals`, `RacingResults`, `RacingStandings`, `RacingTelemetry`, `RacingTrackMap`
- `rhythm`: `RhythmChain`, `RhythmDifficulty`, `RhythmGauge`, `RhythmJudgement`, `RhythmLaneKeys`, `RhythmResults`, `RhythmScoreHud`, `RhythmSongPlate`, `RhythmSongProgress`, `RhythmSongSelect`, `RhythmTiming`
- `rpg`: `RpgCharacter`, `RpgCombo`, `RpgCommands`, `RpgCompanions`, `RpgEffects`, `RpgEnemyInfo`, `RpgExperience`, `RpgHints`, `RpgHits`, `RpgLeader`, `RpgLockOn`, `RpgLootFeed`, `RpgMenuBar`, `RpgMinimap`, `RpgParty`, `RpgPlaceCard`, `RpgPrompt`, `RpgQuestDetail`, `RpgSkills`, `RpgStats`, `RpgTurnOrder`, `RpgVitals`, `RpgWorldMap`
- `shooter`: `ShooterAbilities`, `CombatHud`, `ShooterDamage`, `ShooterEffects`, `Loadout`, `ShooterObjectives`, `ShooterPickups`, `ShooterPrompt`, `ShooterRadar`, `ShooterRound`, `ShooterSquad`, `ShooterVitals`
- `strategy`: `StrategyAlerts`, `StrategyClock`, `StrategyCommands`, `StrategyGroups`, `StrategyMinimap`, `StrategyQueue`, `StrategyResources`, `StrategySelection`, `StrategyTower`, `StrategyWave`
- `world`: `Compass`, `InteractionPrompt`, `LootLabel`, `MinimapView`, `Minimap`, `Nameplate`, `OverheadBars`, `SpeechBubble`, `Waypoint`
- `controls`: `Art`, `Button`, `Counter`, `Meter`, `Progress`
- `screens`: `Banner`, `Calendar`, `CalibrationScreen`, `Card`, `Confirm`, `CreatorToolsScreen`, `CreditsScreen`, `DataList`, `FlowChaptersScreen`, `FlowChoices`, `FlowChartScreen`, `FlowGalleryScreen`, `FlowPicture`, `FlowScreen`, `GameOverScreen`, `HudEditorScreen`, `KeyValues`, `LeaderboardScreen`, `LoadingScreen`, `MainMenuScreen`, `ModeSelectScreen`, `NotificationsScreen`, `ObjectiveTracker`, `Pagination`, `PauseScreen`, `ResultsScreen`, `SaveLoadScreen`, `SettingsScreen`, `SplashScreen`, `StarRating`, `Subtitles`, `Table`, `TitleScreen`, `Tree`, `TutorialScreen`

## 1. ArenaDraft

A ban-and-pick draft: both teams' ban and pick slots in columns, the team choosing now marked current, the phase named in a band, the turn's timer and Lock in. The game owns the order, the turn and when the local player may lock in; it reports onConfirm.

`import { ArenaDraft } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/draft.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The draft's heading; empty takes the kit's (Draft). |
| `phase?` | `string` | The phase the game is in: ban, pick or done (each named in words in the band). |
| `active?` | `string` | The team choosing now: a, b, or empty for neither (its column is marked current and named in the band). |
| `nameA?` | `string` | The first team's name; empty takes the kit's (Blue team). |
| `nameB?` | `string` | The second team's name; empty takes the kit's (Red team). |
| `slots?` | `ReadonlyArray<DraftSlot>` | Both teams' ban and pick slots in the game's order. |
| `timeLeft?` | `number` | Seconds left in the turn as the game counts them (a HudTimer); negative hides it. Default -1. |
| `timeTotal?` | `number` | The turn's whole time, for the timer's ring (0: no ring). |
| `canConfirm?` | `boolean` | Lock in is offered (false: disabled; the game decides when the local player may lock in). |
| `onConfirm?` | `(e: UiEvent) => void` | Lock in. |

Each `DraftSlot`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The slot's key. |
| `team?` | `string` | a or b. |
| `kind?` | `string` | ban or pick. |
| `name?` | `string` | The character placed (empty: the slot is open). |
| `portrait?` | `Texture \| 0` | The character's portrait texture (0 shows the icon). |
| `icon?` | `string` | An icon name, shown while no portrait is set. |
| `player?` | `string` | Who the pick belongs to (empty hides). |
| `active?` | `boolean` | The slot being filled now (Banning, Picking). |

Pack slots: `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `ban`, `done`, `overflow`.

## 2. ArenaLobby

A match lobby: its heading and detail line, the seats in the game's order (open seats offer Invite), the local player's Ready up or Cancel ready, and Start match for the host. The game decides who is ready and when the match can start.

`import { ArenaLobby } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/lobby.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The lobby's heading (the match's name); empty takes the kit's (Lobby). |
| `detail?` | `string` | A line beside the heading as the game words it (Ranked · 2 v 2, Waiting for the host); empty hides it. |
| `slots?` | `ReadonlyArray<LobbySlot>` | The lobby's seats in the game's order. |
| `canInvite?` | `boolean` | Open seats offer Invite (false: they read Open slot only). |
| `isReady?` | `boolean` | The local player is ready: the button reads Cancel ready (unready) instead of Ready up (ready). |
| `canReady?` | `boolean` | The ready button is offered (false: disabled). |
| `isHost?` | `boolean` | The local player hosts: Start match is shown. |
| `canStart?` | `boolean` | Start match is offered (false: disabled; the game decides when the match can start). |
| `onInvite?` | `(e: UiEvent, id: string) => void` | Invite to an open seat. |
| `onReady?` | `(e: UiEvent) => void` | Ready up. |
| `onUnready?` | `(e: UiEvent) => void` | Cancel ready. |
| `onStart?` | `(e: UiEvent) => void` | The host starts the match. |

Each `LobbySlot`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The seat's key, reported back with invite. |
| `name?` | `string` | The seat's player; empty when open. |
| `portrait?` | `Texture \| 0` | The seat's portrait texture (0 shows the icon). |
| `icon?` | `string` | The seat's player's icon, shown while no portrait is set. |
| `open?` | `boolean` | The seat is free. |
| `host?` | `boolean` | The seat's player hosts. |
| `ready?` | `boolean` | The seat's player is ready, in words (Ready or Not ready). |
| `local?` | `boolean` | The seat is the local player's. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.muted`, `Row.frame.selected`.

Fixtures: `default`, `full`, `guest`, `disabled`, `overflow`.

## 3. ArenaReady

A ready check: the players asked and their answers, the time left as the game words it, and Accept and Decline for the local player. It never starts or expires a timer; the game closes the check.

`import { ArenaReady } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/ready.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Ready check. |
| `detail?` | `string` | Match or reason line; empty hides it. |
| `timeText?` | `string` | Game-owned time remaining; empty hides it. This component never starts or expires a timer. |
| `players?` | `ReadonlyArray<ReadyPlayer>` | The players asked. |
| `response?` | `string` | Local response: ready, declined or pending (empty). Both buttons remain available to change a response while canRespond. |
| `canRespond?` | `boolean` | The game allows a local response. False disables both actions. Default true. |
| `closed?` | `boolean` | The check has ended; no more responses. timeText is hidden. |
| `loading?` | `boolean` | Waiting for participants; stale rows are hidden and responses disabled. |
| `onAccept?` | `(e: UiEvent) => void` | Accept. |
| `onDecline?` | `(e: UiEvent) => void` | Decline. |

Each `ReadyPlayer`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls them. |
| `state?` | `string` | ready, declined, offline, or pending (empty). Order belongs to the game. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Row.frame`.

Fixtures: `default`, `selected`, `disabled`, `closed`, `overflow`.

## 4. ArenaRoster

A character select: the roster as a grid of tiles, the local player's choice selected and named under it, and who is choosing. It reports onChoose; the game decides what may be chosen.

`import { ArenaRoster } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/roster.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The screen's heading; empty takes the kit's (Choose your champion). |
| `characters?` | `ReadonlyArray<RosterCharacter>` | The roster in the game's order. |
| `selected?` | `string` | The local player's character (selected, named under the grid); empty is none. |
| `players?` | `ReadonlyArray<RosterPlayer>` | Who is choosing. |
| `compact?` | `boolean` | Its place is narrow (under about 380 px): the tiles, padding and chosen name tighten. The place decides, not the screen. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | The local player chooses a character. |

Each `RosterCharacter`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The character's key, reported back with choose. |
| `name` | `string` | What a reader calls them. |
| `portrait?` | `Texture \| 0` | The character's portrait texture (og.load.texture, 0 shows the icon instead). |
| `icon?` | `string` | An icon name, shown while no portrait is set. |
| `role?` | `string` | A key for the pack (tank, damage, support…). |
| `roleLabel?` | `string` | The role's word. |
| `locked?` | `boolean` | The player cannot pick it yet. |
| `takenBy?` | `string` | Who already holds it (empty: free). |

Each `RosterPlayer`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The player's key. |
| `slot?` | `number` | 1 to 4, the mark P1…P4. |
| `at?` | `string` | The character id the player is on (empty: none). |
| `ready?` | `boolean` | The player has locked in (a check beside the mark). |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `none`, `full`, `disabled`, `overflow`.

## 5. ArenaRoundHud

The round's HUD for two teams: each team's name and score on a plate (the local team's marked You), the round and its total, the round's timer, and a banner when the game decides the round's outcome. Read only.

`import { ArenaRoundHud } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/round-hud.tsx`).

| Prop | Type | What |
|---|---|---|
| `round?` | `number` | The round being played, as the game counts it. |
| `rounds?` | `number` | How many rounds the match has (Round 3 of 7); 0 hides the total. |
| `nameA?` | `string` | The first team's name; empty takes the kit's (Blue team). |
| `nameB?` | `string` | The second team's name; empty takes the kit's (Red team). |
| `scoreA?` | `number` | The first team's score (rounds won) as the game counts it. |
| `scoreB?` | `number` | The second team's score as the game counts it. |
| `localTeam?` | `string` | The local player's team, a or b (its plate is marked You); empty is a spectator. |
| `timeLeft?` | `number` | Seconds left in the round as the game counts them (a HudTimer); negative hides it. Default -1. |
| `timeTotal?` | `number` | The round's whole time, for the timer's ring (0: no ring). |
| `outcome?` | `string` | The round's end as the game decided it, for the local side: won, lost, draw, or none (no banner). Default none. |
| `at?` | `number` | The banner's id: a new value replays its entrance. |
| `detail?` | `string` | A line under the banner as the game words it (Flawless, Match point); empty hides it. |

Pack slots: `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `won`, `lost`, `draw`, `critical`, `overflow`.

## 6. ArenaScoreboard

A two-team scoreboard: each team's name and score and its players' rows, kills, deaths, assists and score or score alone, side by side or one above the other. Read only; the game orders the rows.

`import { ArenaScoreboard } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/scoreboard.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The board's heading; empty takes the kit's (Scoreboard). |
| `columns?` | `string` | What each row shows: kda (kills, deaths, assists and score) or score (score only). |
| `nameA?` | `string` | The first team's name; empty takes the kit's (Blue team). |
| `nameB?` | `string` | The second team's name; empty takes the kit's (Red team). |
| `scoreA?` | `string` | The first team's score as the game words it (rounds, points); empty hides it. |
| `scoreB?` | `string` | The second team's score as the game words it; empty hides it. |
| `rows?` | `ReadonlyArray<ScoreboardRow>` | Every player in the game's order. |
| `split?` | `boolean` | The two teams stand side by side; otherwise one above the other. |

Each `ScoreboardRow`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The player's key. |
| `team?` | `string` | a or b. |
| `name?` | `string` | What a reader calls them. |
| `kills?` | `number` | Kills as the game counts them. |
| `deaths?` | `number` | Deaths as the game counts them. |
| `assists?` | `number` | Assists as the game counts them. |
| `score?` | `number` | The score as the game counts it. |
| `local?` | `boolean` | The local player's row (You, the row is current). |
| `disconnected?` | `boolean` | The row is dimmed and named Disconnected. |

Pack slots: `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.muted`, `Row.frame.selected`.

Fixtures: `default`, `score`, `disconnected`, `overflow`.

## 7. ArenaSpectator

A spectator's controls: the subjects the game offers to follow, the one followed, previous and next, free camera and Leave. It reports intents; the game moves the camera.

`import { ArenaSpectator } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/spectator.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Spectating. |
| `detail?` | `string` | Match status, broadcast delay or disabled reason, as the game words it. |
| `targets?` | `ReadonlyArray<SpectatorTarget>` | The subjects the game offers. follow reports their stable id. |
| `selected?` | `string` | The followed subject id. Empty means no chosen subject; freeCamera takes precedence over its selected state. |
| `freeCamera?` | `boolean` | Free camera active; choosing a subject only reports follow, leaving the game to change camera mode. |
| `canFree?` | `boolean` | Show and enable the free camera action; false omits it. Default true. |
| `canCycle?` | `boolean` | Enable previous and next. The game owns cycling order and wrapping. Default true. |
| `canLeave?` | `boolean` | Enable Leave; false keeps the exit visible but disabled. Default true. |
| `loading?` | `boolean` | Waiting for subjects; hides stale rows and disables camera actions, but not Leave. |
| `onFollow?` | `(e: UiEvent, id: string) => void` | Follow a subject. |
| `onPrevious?` | `(e: UiEvent) => void` | Previous subject. |
| `onNext?` | `(e: UiEvent) => void` | Next subject. |
| `onFree?` | `(e: UiEvent) => void` | Free camera. |
| `onLeave?` | `(e: UiEvent) => void` | Leave. |

Each `SpectatorTarget`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls them. |
| `team?` | `string` | Their side as the game words it; empty hides it. |
| `detail?` | `string` | A line about them (Alive, Respawning · 4s, Disconnected). |
| `disabled?` | `boolean` | The game decides which subjects can be followed. |

Pack slots: `Button.plate`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `free`, `disabled`, `loading`, `empty`, `overflow`.

## 8. ArenaVersus

A versus reveal: each side's team name and fighters facing each other, replayed when its id changes. An empty side reads Waiting for opponents. Read only.

`import { ArenaVersus } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/versus.tsx`).

| Prop | Type | What |
|---|---|---|
| `nameA?` | `string` | The first side's team name; empty takes the kit's (Blue team). |
| `nameB?` | `string` | The second side's team name; empty takes the kit's (Red team). |
| `sideA?` | `ReadonlyArray<VersusFighter>` | The first side's fighters in the game's order. |
| `sideB?` | `ReadonlyArray<VersusFighter>` | The second side's fighters, keyed as sideA. An empty side reads Waiting for opponents. |
| `at?` | `number` | The reveal's id: a new value replays the sides' entrance. |
| `compact?` | `boolean` | Its place is narrow (under about 300 px): the cards and portraits tighten. The place decides, not the screen. |

Each `VersusFighter`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The fighter's key. |
| `name` | `string` | What a reader calls them. |
| `portrait?` | `Texture \| 0` | The fighter's portrait texture (0 shows the icon). |
| `icon?` | `string` | An icon name, shown while no portrait is set. |
| `character?` | `string` | The champion's name (empty hides). |
| `rank?` | `string` | The rank as the game words it (Gold II, 1840 SR; empty hides). |
| `local?` | `boolean` | Marks the local player (You). |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `teams`, `overflow`.

## 9. ArenaVote

A map or mode vote: the options, the local vote, the time left and, once closed, the winner the game names. It reports onChoose with the option's id; the game counts the votes.

`import { ArenaVote } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/vote.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Map vote. |
| `detail?` | `string` | Voting rules or a disabled reason, supplied by the game. |
| `timeText?` | `string` | Game-owned time remaining; empty hides it. Hidden once closed. |
| `options?` | `ReadonlyArray<VoteOption>` | The choices. choose reports the stable id, never the index. |
| `selected?` | `string` | The local vote's option id; empty means no vote. A click reports choose but does not change it locally. |
| `winner?` | `string` | The winning option id, supplied by the game. Shown only when closed; empty means no announced result. |
| `canVote?` | `boolean` | Whether the local player can vote or change a vote. Default true. |
| `closed?` | `boolean` | Voting is over; options remain readable but disabled. |
| `loading?` | `boolean` | Waiting for options; hides stale options and disables voting. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose an option. |

Each `VoteOption`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What the map or choice is called. |
| `icon?` | `string` | An icon of the style's atlas; empty is none. |
| `detail?` | `string` | A line about it. |
| `votes?` | `number` | The votes so far; the game counts them. |
| `disabled?` | `boolean` | It cannot be chosen. |
| `reason?` | `string` | Why it cannot be chosen (Not installed). |

Pack slots: `Panel.frame`.

Fixtures: `default`, `closed`, `disabled`, `loading`, `empty`, `overflow`.

## 10. Avatar

A portrait in a round well: presence is a dot on its rim (and words under the name, so it never rests on colour), the level a badge across its foot.

`import { Avatar } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/avatar.tsx`).

| Prop | Type | What |
|---|---|---|
| `name?` | `string` | The displayed and accessible name; empty uses the localized unnamed-player label. |
| `showName?` | `boolean` | Shows the name caption; false keeps the accessible name for composition beside a heading. Default true. |
| `portrait?` | `Texture \| 0 \| undefined` | A loaded portrait texture; without one the fallback icon shows. |
| `icon?` | `string` | Fallback atlas icon when there is no portrait. Default "person". |
| `level?` | `number` | Level badge; 0 hides it. Default 0. |
| `status?` | `string` | online, away, busy or offline; empty hides the presence row. It is shown in words as well as colour. |
| `statusText?` | `string` | The game's wording of presence; empty uses the standard status label. |
| `frame?` | `string` | A frame key a pack selects through data-frame; the component imposes no decorative frame. |
| `loading?` | `boolean` | Portrait data is loading; shows a loading label and keeps the name and layout. Default false. |

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `away`, `busy`, `offline`, `empty`, `loading`, `captionless`.

## 11. Currency

A readout, not a button. A game can put it beside a separately named purchase control; Currency never invents an economy or sends an intent. A chip: the resource's icon in a socket, its name over the amount, and beside the amount its capacity, its rate and Full. It is as wide as what it holds.

`import { Currency } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/currency.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | Resource name; empty uses Currency. |
| `icon` | `string` | Atlas icon; empty omits it. |
| `amount` | `number` | Amount held; zero remains visible. Ignored when valueText is supplied. |
| `valueText` | `string` | Game-formatted amount; empty uses the compact amount. |
| `limit` | `string` | Game-formatted capacity; empty omits it. The game decides whether the store is full. |
| `rate` | `string` | Optional income or drain, including its sign and unit. |
| `full?` | `boolean` | At capacity; reads Full in words. |
| `loading?` | `boolean` | Awaiting the amount; hides stale figures and reads Loading. |
| `unavailable?` | `boolean` | The balance could not be read; hides stale figures and reads Unavailable. |

Fixtures: `default`, `empty`, `full`, `loading`, `unavailable`, `overflow`.

## 12. EmptyState

Where content would be, there is a recess: a mark on a disc, why the area is empty, and the one thing to do about it.

`import { EmptyState } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/empty-state.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Nothing here yet. |
| `body?` | `string` | Why this area is empty. |
| `hint?` | `string` | An optional next step, in quieter text. |
| `icon?` | `string` | An icon above the heading; empty hides it. Default "bag". |
| `actionLabel?` | `string` | The optional action's words; empty hides the button. |
| `disabled?` | `boolean` | The action is unavailable. Default false. |
| `onActivate?` | `(e: UiEvent, key?: string) => void` | The activate event, with the row's key where it comes from a row. |

Fixtures: `default`, `minimal`, `disabled`, `overflow`.

Actions: `activate` (Take the suggested action).

## 13. ErrorState

EmptyState's recess with the failure on a disc tinted and ringed in danger (quieter than the action it leads to): what went wrong, the support reference set apart as a code, then retry (the primary action) and the way out.

`import { ErrorState } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/error-state.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Something went wrong. |
| `body?` | `string` | The game's explanation and recovery advice. |
| `code?` | `string` | Optional support reference; never a raw exception or credential. |
| `retryLabel?` | `string` | Retry wording; empty uses Try again. |
| `canRetry?` | `boolean` | Offer the retry action. Default true. |
| `busy?` | `boolean` | Retry is in progress; both actions are disabled and a wait is announced. Default false. |
| `dismissLabel?` | `string` | Optional secondary action; empty hides it. |
| `onDismiss?` | `(e: UiEvent, key?: string) => void` | The dismiss event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |

Fixtures: `default`, `busy`, `terminal`, `overflow`.

Actions: `retry` (Retry the failed operation), `dismiss` (Leave the error state).

## 14. ItemSlot

Reports choose. In a keyed collection the intent keeps its enclosing row key; otherwise the game's stable component instance identifies it.

`import { ItemSlot } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/item-slot.tsx`).

| Prop | Type | What |
|---|---|---|
| `name?` | `string` | The item name, also its accessible name and tooltip. Empty slots use the localized empty-slot name. |
| `icon?` | `string` | An atlas icon. Empty is no item art. |
| `count?` | `number` | The quantity; a badge appears above 1. Default 1. |
| `rarity?` | `string` | The item's rarity: common, uncommon, rare, epic, legendary or mythic; a pack selects data-rarity. Default "common". |
| `cooldown?` | `number` | Seconds remaining, supplied by the game. A cooling item can still be selected for inspection. Default 0. |
| `cooldownMax?` | `number` | The cooldown's full duration; the overlay shows the remaining fraction. Default 1. |
| `keyLabel?` | `string` | A shortcut label; empty hides it. |
| `selected?` | `boolean` | The game-selected slot, exposed as aria-pressed. Default false. |
| `disabled?` | `boolean` | Unavailable; selection is disabled. Default false. |
| `locked?` | `boolean` | Locked; shown in words and selection is disabled. Default false. |
| `loading?` | `boolean` | Waiting for item data; selection is disabled. Default false. |
| `empty?` | `boolean` | An empty slot. It can be selected so the game can offer an equip action. Default false. |
| `showName?` | `boolean` | Show the wrapping item name below the slot. Default false. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Slot.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `selected`, `locked`, `disabled`, `loading`, `empty`.

## 15. ListItem

One line of a list: what it is, its name over a supporting line, then what the game says about it at the trailing end (a value over a badge), then its state (chosen or refreshing). The trailing end stays on the row while the words keep a measure beside it; a long value on a narrow row drops under the words instead, with its state, so the name is not squeezed into a column.

`import { ListItem } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/list-item.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The row's primary text. |
| `subtitle?` | `string` | Optional secondary text, wrapping below the title. |
| `icon?` | `string` | Optional leading icon, used when image is zero. |
| `image?` | `Texture \| 0 \| undefined` | Optional loaded portrait or item texture. |
| `value?` | `string` | The game's formatted trailing value. |
| `badge?` | `string` | Optional trailing badge, such as New or 3. |
| `selected?` | `boolean` | Chosen by the game; marked with a check and aria-pressed. Default false. |
| `disabled?` | `boolean` | Unavailable; cannot choose. Default false. |
| `loading?` | `boolean` | Refreshing this row; prevents choosing and shows a spinner without hiding its content. Default false. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `disabled`, `loading`, `overflow`.

Actions: `choose` (Choose this row (the enclosing collection's stable key)).

## 16. Pips

A counted resource on one line: its name, then one icon per unit. An empty unit is its socket; a full one is cut to its fraction and wears the tone. Unavailable, the units lose their colour and dim, full ones still brighter than their sockets.

`import { Pips } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/pips.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The resource's name; empty uses Resource. |
| `icon?` | `string` | The atlas icon for a unit: heart, bullet, bolt or a game's own icon. Default "heart". |
| `items?` | `ReadonlyArray<{ id: string; value: number; label?: string }>` | Keyed units { id, value, label }: value is the filled fraction, 0 to 1; label names the unit for accessibility. The game supplies capacity, ordering and fractional units, with no fixed maximum. |
| `loading?` | `boolean` | Waiting for resource data; units are replaced by Loading. Default false. |
| `disabled?` | `boolean` | The resource is unavailable; its values remain readable. Default false. |
| `tone?` | `string` | The full units' colour: empty is the style's pip colour (a life's red); success, warn, danger, info or primary names another. |

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `full`, `empty`, `depleted`, `loading`, `disabled`.

## 17. Ribbon

A strip across a corner of whatever it lies over (or a stamp across its middle). Its box only holds and cuts the strip: it draws nothing of its own, so the card under it shows. A game puts the box in the corner it names of a positioned card that clips (`overflow: hidden`), and the card's own corner cuts the strip's ends.

`import { Ribbon } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/ribbon.tsx`).

| Prop | Type | What |
|---|---|---|
| `text` | `string` | The ribbon's words (NEW, SALE). |
| `corner` | `string` | Where it sits: topRight (empty), topLeft, bottomRight, bottomLeft or center (a stamp). |
| `tone` | `string` | Its tone: primary (empty), success, warn, danger or info. |

Fixtures: `default`, `topLeft`, `bottomRight`, `bottomLeft`, `stamp`, `overflow`.

## 18. SkeletonGroup

One band of light crosses the whole group: every bone's sheen is the same width (a share of the group's) and starts from the group's edge, so the bones light together as it passes.

`import { SkeletonGroup } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/skeleton-group.tsx`).

| Prop | Type | What |
|---|---|---|
| `shape` | `string` | What it stands in for: lines (empty), avatar, block, row or card. |
| `items` | `ReadonlyArray<{ id: string }>` | One entry per repeat (rows of a list), each with a stable id. |
| `name` | `string` | What a screen reader says while it loads; empty is Loading. |

Fixtures: `default`.

## 19. Spinner

A ring that is always there and an arc that runs round it. `size` is its width; a host that holds nothing else while it waits (a portrait, a slot) sets `--spinner-size` on the box round it instead, so the ring is in proportion to it.

`import { Spinner } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/spinner.tsx`).

| Prop | Type | What |
|---|---|---|
| `size?` | `number` | Its width and height, CSS px. Default 28. |
| `name` | `string` | What a screen reader calls it; empty is Loading. |

Fixtures: `default`.

## 20. Steps

A rail of numbered marks: each step is a mark, the reach to the next one (full once the step is done) and its words. It lies along a row and stands up as a column where `column` asks or the container is narrow.

`import { Steps } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/steps.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Optional visible heading; the accessible fallback is Progress steps. |
| `steps?` | `ReadonlyArray<{ id: string; label: string; detail?: string; number: string; state: string; disabled?: boolean }>` | Keyed { id, label, detail, number, state, disabled } rows. number is the game's formatted marker, state upcoming/current/done/error. The game chooses order and allowed navigation. Default []. |
| `column?` | `boolean` | Stack steps vertically instead of a wrapping horizontal sequence. Default false. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `empty`, `column`, `error`, `overflow`.

Actions: `choose` (Choose a step, by its stable key).

## 21. CardCollection

The card library of a deck builder: a heading, filter tabs and a grid of cards that fills the height it is given and scrolls inside it. The game filters and sorts what it passes; the grid reports add (a card's key: tap, Enter, pad A), inspect (X or pad Y on the focused card) and choosefilter (a filter's key), tabNext and tabPrev (E/Q, bumpers). A card the game will not add never reports add: it stays in reach, so it can be focused and read, reports inspect from a tap too, and says why in the game's words, with a lock when it is not owned and a cross when it is. The core DataGrid is a table of text rows, so the cards are a grid of their own.

`import { CardCollection } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/collection.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The collection's heading (Collection, Library); empty hides the band. |
| `filters` | `ReadonlyArray<{ id: string; label: string; count?: number }>` | The filters the game offers (classes, costs, types), keyed rows of { id, label, count } (count above 0 shows beside the label); empty hides the row. |
| `filter` | `string` | The id of the filter applied; the game filters the cards it passes. |
| `cards` | `ReadonlyArray<{ id: string; name: string; cost: string; icon: string; art: Texture \| 0; type: string; typeLabel: string; rarity: string; text: string; owned: number; inDeck: number; canAdd: boolean; locked: boolean; reason: string }>` | The cards shown, already filtered and sorted by the game: keyed rows of { id, name, cost, icon, art, type, typeLabel, rarity, text, owned, inDeck, canAdd, locked, reason }. owned and inDeck are the game's counts (owned above 1 is shown, inDeck above 0); canAdd is the game's answer (limit, rarity rule, not owned); locked marks a card not owned yet; reason is why the game will not add it, in the game's words (spoken and marked when canAdd is false; empty says Not owned for a locked card, Cannot be added otherwise). |
| `selected` | `string` | The card shown in detail; empty is none. |
| `empty` | `string` | What an empty result says; empty is No cards match. |
| `onAdd?` | `(e: UiEvent, key?: string) => void` | The add event, with the row's key where it comes from a row. |
| `onChoosefilter?` | `(e: UiEvent, key?: string) => void` | The choosefilter event, with the row's key where it comes from a row. |
| `onInspect?` | `(e: UiEvent, key?: string) => void` | The inspect event, with the row's key where it comes from a row. |
| `onTabNext?` | `(e: UiEvent, key?: string) => void` | The tabNext event, with the row's key where it comes from a row. |
| `onTabPrev?` | `(e: UiEvent, key?: string) => void` | The tabPrev event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `empty`.

Actions: `tabPrev` (Previous filter), `tabNext` (Next filter), `details` (Inspect).

## 22. CardDeckList

The deck being built: its name, the size against the game's limit (a meter and the figures, data-over and data-under as well as the ink), one row per card with its cost, name and copies, a minus and a plus per row, and the main button. The game counts, sorts and rules; the list reports choose (a row's key), add and remove (a row's plus or minus, its key; X or pad Y anywhere in the focused row is its minus, so it is silent where the game refuses the removal) and confirm. The rows fill the height given and scroll inside it; the size and the button keep the foot.

`import { CardDeckList } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/deck-list.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The deck's name as the player named it. |
| `entries` | `ReadonlyArray<{ id: string; name: string; cost: string; type: string; rarity: string; copies: number; canAdd: boolean; canRemove: boolean }>` | The deck's cards in the game's order (by cost, then name), keyed rows of { id, name, cost, type, rarity, copies, canAdd, canRemove }: copies the game's count; canAdd and canRemove the game's answers (its copy limit, its deck limit, a card the deck must keep). |
| `total` | `number` | How many cards the deck holds (the game's count). |
| `limit` | `number` | The size the game requires or allows; 0 hides the size line. total above limit reaches the size as data-over, below as data-under. |
| `selected` | `string` | The entry shown in detail; empty is none. |
| `confirmLabel` | `string` | The main button's text (Save deck, Done); empty hides it. |
| `canConfirm?` | `boolean` | The main button is enabled (the game's verdict on the deck). |
| `onAdd?` | `(e: UiEvent, key?: string) => void` | The add event, with the row's key where it comes from a row. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onConfirm?` | `(e: UiEvent, key?: string) => void` | The confirm event, with the row's key where it comes from a row. |
| `onRemove?` | `(e: UiEvent, key?: string) => void` | The remove event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `empty`, `full`.

Actions: `details` (Remove one).

## 23. CardFace

One card's face: the cost gem, the art window over its rarity's glow, the name on its plate, the type line and the rules text. It draws what the game words and reports nothing; the hand, the collection and the reward wrap it in their buttons. It fills the box it is given (give the box the card's aspect ratio). The frame wears its type's colour; the name takes two lines and the rules text four, each ending in an ellipsis at a whole line, and the art window yields its height to them.

`import { CardFace } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/face.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The card's name as the game words it (an upgraded card's name included). |
| `cost` | `string` | Its cost as the game words it (2, X, 0); empty hides the cost gem (a relic, an unplayable curse). |
| `icon` | `string` | Its art as an icon in the style's or the game's atlas, used when art is 0. |
| `art` | `Texture \| 0` | A painted picture (og.load.texture) filling the art window; 0 shows the icon. |
| `type` | `string` | The card's type for the pack's frame (attack, skill, power, curse, status, or the game's own word); it reaches the frame as data-type. |
| `typeLabel` | `string` | The type line as the game words it (Attack, Skill · Rare); empty hides it. |
| `rarity` | `string` | common, uncommon, rare, epic, legendary or mythic; rare and above draw the ornate card frame. |
| `text` | `string` | The rules text as the game words it, every number already computed; it wraps, up to four lines and then an ellipsis, and the art window gives it the room. |
| `dim?` | `boolean` | Dims the art and the frame (a card that cannot be played or is not owned); the words keep their ink. |
| `size?` | `string` | How short its place is: empty is full height; medium, small and tiny clamp the rules text to 3, 2 and 1 lines (tiny also drops the type line). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `relic`, `overflow`, `dim`.

## 24. CardHand

The player's hand: cards side by side, overlapping as the hand grows so every card keeps a visible edge, turned along a fan when the game asks for one. The selected card rises; when the game says it is playable a Play plate sits on it. A card the game says cannot be played is not disabled (it can still be picked up and read) but carries the unplayable mark and data-playable=false, and never reports play. Reports select (a card's key: tap, Enter, pad A) and play (the Play plate, or X / pad Y on a focused playable card, its key). The game decides legality, cost and targets.

`import { CardHand } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/hand.tsx`).

| Prop | Type | What |
|---|---|---|
| `cards` | `ReadonlyArray<{ id: string; name: string; cost: string; icon: string; art: Texture \| 0; type: string; typeLabel: string; rarity: string; text: string; playable: boolean; at: number; reason?: string }>` | The hand in the game's order, keyed rows of { id, name, cost, icon, art, type, typeLabel, rarity, text, playable, reason, at }: playable is the game's answer (cost, targets, rules), never the hand's; reason the game's words for an unplayable card (No energy), empty Cannot be played; at is the card's place across a fanned hand from -1 (first) to 1 (last), 0 the middle, which the game derives from the order. |
| `selected` | `string` | The id of the card the player picked up (it rises and offers Play when playable); empty is none. |
| `fan` | `number` | How far a fanned hand turns its end cards, in degrees (the game's choice, 0 a flat row). |
| `label` | `string` | What a screen reader calls the hand; empty is Hand. |
| `onPlay?` | `(e: UiEvent, key?: string) => void` | The play event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `Button.plate.primary`, `Plate.frame`.

Fixtures: `default`, `empty`, `single`, `disabled`, `overflow`.

Actions: `details` (Play).

## 25. CardPiles

The draw, discard and exhaust piles as buttons: a stack of card backs (one when it is empty) with the pile's icon on top, the game's count on a tag at its corner and a label under it. A pile at 0 carries data-empty; one the game will not open is disabled. Reports open (a pile's key). The piles sit in a row spread across the width they are given; a page that wants the draw pile in one corner and the discard in the other mounts two.

`import { CardPiles } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/piles.tsx`).

| Prop | Type | What |
|---|---|---|
| `piles?` | `ReadonlyArray<{ id: string; kind: string; label: string; icon: string; count: number; enabled: boolean }>` | The piles in the game's order, keyed rows of { id, kind, label, icon, count, enabled }: kind is draw, discard, exhaust or the game's own word (it reaches the pile as data-kind); count the game's number; enabled false when the game will not open it. |
| `label` | `string` | What a screen reader calls the group; empty is Piles. |
| `onOpen?` | `(e: UiEvent, key?: string) => void` | The open event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `empty`, `overflow`.

## 26. CardReward

Pick one of the game's offers: cards drawn as cards, relics as a gold plate with their mark, kind, name and text, and a skip button. The game rolls the offers and applies the pick; the screen reports pick (an offer's key: tap, Enter, pad A) and skip (the button, Escape, pad B). An offer the game refuses is disabled and dimmed; the one the player is looking at wears brackets. The offers share one row while it fits; a phone shows one at nearly full width, so its text stays whole, with the next one peeking at the faded edge, and scrolls sideways.

`import { CardReward } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/reward.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The heading as the game words it (Choose a reward, Treasure). |
| `kicker?` | `string` | A small line over the title naming the moment (Victory · Floor 7); empty hides it. |
| `choices` | `ReadonlyArray<{ id: string; relic: boolean; name: string; cost: string; icon: string; art: Texture \| 0; type: string; typeLabel: string; rarity: string; text: string; enabled: boolean }>` | The offers the game rolled, keyed rows of { id, relic, name, cost, icon, art, type, typeLabel, rarity, text, enabled }: relic true draws a relic (its icon, typeLabel or Relic, name and text) instead of a card; enabled false when the game will not let it be taken. |
| `selected` | `string` | The offer the player is looking at (focused or tapped once); empty is none. |
| `skipLabel` | `string` | The skip button's text (Skip, Take nothing); empty hides it (Back still reports skip; a game that offers none ignores it). |
| `onPick?` | `(e: UiEvent, key?: string) => void` | The pick event, with the row's key where it comes from a row. |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Plate.frame`.

Fixtures: `default`, `relics`, `empty`.

Actions: `back` (Skip).

## 27. AbilityBar

A strip of AbilitySlot blocks, and nothing else: the slot draws itself, the strip places them. Each place is --ability-bar-slot-size wide and, where the strip is given less, shrinks alike down to --ability-bar-slot-min; under that the strip breaks into lines, so it never overflows. A game that wants another arrangement (two rows, groups, an ultimate at the end) lays AbilitySlot out itself in the same way.

`import { AbilityBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/ability-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `slots` | `ReadonlyArray<{ id: string; icon: string; name: string; action: string; hotkey: string; cooldown: number; cooldownMax: number; cost: string; unaffordable: boolean; locked: boolean; active: boolean; count?: number }>` | The slots, in order: each { id, icon, name, action, hotkey, cooldown, cooldownMax, cost, unaffordable, locked, active, count }, cooldown in seconds (0: ready); count above 0 shows how many are left (a quick item: potions, scrolls). A slot's full set of states is the AbilitySlot component. |
| `column?` | `boolean` | The slots run top to bottom; false is a row. |
| `onUse?` | `(e: UiEvent, id: string) => void` | Use. |

Fixtures: `default`.

## 28. AbilitySlot

One ability or quick item: its icon on a gradient, the hotkey's cap, a cooldown sweep, charges, a count, its cost, ranks as pips and its states (unaffordable, out of range, active, locked, disabled, an ultimate's charge ring). It reports onUse; the game decides whether it fires.

`import { AbilitySlot } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/ability-slot.tsx`).

| Prop | Type | What |
|---|---|---|
| `icon` | `string` | An icon of the atlas, drawn large on a gradient. |
| `name` | `string` | What a screen reader calls it (Fireball). |
| `action` | `string` | The action whose glyph the hotkey cap shows on the device used last (it follows a rebinding); empty is none. |
| `hotkey` | `string` | A label for the cap instead of the action's glyph (Q, LB); empty shows the action's. |
| `cooldown` | `number` | Seconds of cooldown left; 0 is ready. |
| `cooldownMax?` | `number` | The cooldown's length; the sweep shows cooldown over it. Default 1. |
| `charges` | `number` | Charges ready, in a corner badge when maxCharges is over 1. |
| `maxCharges` | `number` | Charges it holds; 0 or 1 is none. |
| `count` | `number` | How many are left (a quick item: potions, scrolls), at the bottom's end; 0 shows none. |
| `cost` | `string` | Its cost (40) at the bottom left while ready; empty is none. |
| `unaffordable?` | `boolean` | Not enough resource: the art tinted red and the cost in danger. |
| `outOfRange?` | `boolean` | The target is out of range: the art tinted red. |
| `active?` | `boolean` | Toggled on or channelling: a breathing glow and a primary edge. |
| `locked?` | `boolean` | Not learnt yet: dark art under a lock. |
| `disabled?` | `boolean` | Cannot be used now: the art washed out under a red slash. |
| `ultimate?` | `number` | An ultimate's charge 0 to 1: larger, a gold ring filling round it and a percent; negative is a plain slot. Default -1. |
| `level` | `number` | Ranks learnt (pips right under it). |
| `fit?` | `boolean` | It fills the place it is given (a strip's), rather than standing --ability-slot-size wide. |
| `maxLevel` | `number` | Ranks it has; 0 draws no pips. |
| `onUse?` | `(e: UiEvent) => void` | Use. |

Pack slots: `Slot.frame`.

Fixtures: `default`.

## 29. ActionButton

A touch disc: an action a finger holds, the primary one larger, an optional charge ring round its rim. The disc's box is a control zone of its og-input action (touchButton), so the action is 1 while a touch that started on it is held; the disc has no press handler of its own. The face's radial sheen is a linear one.

`import { ActionButton } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/action-button.tsx`).

| Prop | Type | What |
|---|---|---|
| `action` | `string` | The og-input action it feeds while a touch holds it (jump). |
| `label` | `string` | Its words, its accessible name; shown when it has no icon. |
| `icon?` | `string` | An icon of the atlas on its face; empty shows the label. |
| `primary?` | `boolean` | The main action: the primary fill and a larger disc. |
| `charge?` | `number` | A charge round its rim, 0 to 1 (an ultimate); negative is none. Default -1. |
| `disabled?` | `boolean` | It feeds nothing and shows faded. |

Fixtures: `default`.

## 30. ActionHints

A bar of input hints: each a key cap showing its action's binding glyph and what it does, one that fires on a hold keeping the hold and running its fill as it holds.

`import { ActionHints } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/action-hints.tsx`).

| Prop | Type | What |
|---|---|---|
| `hints?` | `ReadonlyArray<{ action: string; hint: string; hold?: boolean }>` | The hints, a keyed collection of { action, hint, hold }: action the og-input action whose glyph its cap shows on the device used last, hint what it does (Buy), hold true when it fires on a hold (its fill grows while the action or a press holds it). |
| `label?` | `string` | What a screen reader calls the bar; empty is Actions. |
| `onPress?` | `(e: UiEvent, action: string) => void` | Do. |
| `onHold?` | `(e: UiEvent, action: string) => void` | Hold, completing after 0.8 s; the hold hint's fill runs while it holds. |

Fixtures: `default`.

## 31. AmmoCounter

Ammunition at the screen's bottom corner: the reserve weapon on a small plate, the magazine as the HUD's largest figure with the reserve beside it, a tick per round under the count (lit while loaded), and the fire mode as pips beside the weapon's name; the labels sit on HUD plates. Low and empty change the count's ink and say so; a reload shows its progress. The game owns every count; read only. The ticks' mask and the lean (skewX) have no TSX surface: the ticks are separate views, the lean is dropped.

`import { AmmoCounter } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/ammo-counter.tsx`).

| Prop | Type | What |
|---|---|---|
| `weapon` | `string` | The weapon's name. |
| `mode?` | `string` | Its fire mode: auto, burst or single (beside the name in words, and as three leaning pips: all lit for auto, two for burst, one for single). Default auto. |
| `magazine` | `number` | Rounds in the magazine. |
| `capacity` | `number` | Rounds a full magazine holds: a leaning tick per round under the count, lit for each round left (a continuous strip past 60). |
| `reserve` | `number` | Rounds carried beyond the magazine. |
| `reloading?` | `number` | The reload's progress, 0 to 1; negative while not reloading. Default -1. |
| `low?` | `number` | The fraction of capacity at or under which the count turns low (0.25). Default 0.25. |
| `secondary?` | `string` | The weapon held in reserve, on a small plate over the readout; empty hides it. |
| `secondaryAmmo?` | `number` | The reserve weapon's rounds; negative hides the figure. Default -1. |
| `secondaryKey?` | `string` | The input that swaps to it as the game names it (2), on a key cap; empty hides it. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `low`, `empty`, `reloading`, `belt`, `overflow`.

## 32. HudBar

One bar of a HUD, a block a plate is built from: it fills to value over max in its kind's colour over a dark well with a lit rim; a loss leaves a warm ghost that drains after the fill, a low bar turns to danger, segment cuts it into cells, and showValue stands its figure over it. It is as wide as its place; a plate sizes the place. Read only. The old style's lean (skewX) and the cells' mask have no TSX surface: the cells are drawn as notch marks, the lean is dropped.

`import { HudBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The value now. |
| `max?` | `number` | Its most; 0 draws the bar empty. Default 1. |
| `kind?` | `string` | What it measures, which is its fill: health, mana, stamina, shield or xp. Default health. |
| `ghost?` | `number` | The value before the latest loss: a warm ghost behind the fill that drains after it (at or under value: none). |
| `segment?` | `number` | Value per cell (one dodge's cost): the bar is cut into cells; 0 draws it whole. |
| `low?` | `boolean` | Low: the fill turns to danger and the rim glows. |
| `showValue?` | `boolean` | Its figure, value over max, upright over the bar; for a bar thick enough to hold a line of text. |
| `name?` | `string` | What a screen reader calls it; empty is its kind (Health, Mana, Stamina, Experience). |

Fixtures: `default`, `mana`, `cells`, `shield`, `figure`, `low`, `empty`, `overflow`.

## 33. BossBar

A boss's bar across the top of the screen, on a HUD plate: a Boss tag, the name and its epithet; a long health bar with a warm ghost behind the fill and a diamond at each phase boundary; under it the stagger gauge cut into cells, the elements the boss is weak to and its health's share. The game owns every value; read only. The style's lean (skewX), the gauge's cut mask (notch marks stand in) and the weakness tiles' cut corners (rounded ones stand in) have no TSX surface; the invulnerable fill is greyed in kind (a grey fill, since the greying filter is dropped).

`import { BossBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/boss-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `name?` | `string` | The boss name; empty uses Boss. |
| `epithet?` | `string` | The boss's title as the game words it (Warden of the Ninefold Gate), beside the name; empty hides it. |
| `value?` | `number` | Health, clamped to the supplied range for display; its share of max stands in figures at the bar's end. |
| `max?` | `number` | Full health; a nonpositive capacity shows no health instead of dividing by zero. |
| `ghost?` | `number` | Health before the latest loss: a warm ghost behind the fill that drains after it (at or under value: none). |
| `phases?` | `ReadonlyArray<BossBarPhase>` | Keyed phases in the fight's order; the game owns phase progression. A diamond on the bar marks each boundary between phases (the first phase is the bar's right end), dimmed once health has passed it. |
| `phase?` | `string` | The current phase's stable id; read out with the health. |
| `stagger?` | `number` | The stagger gauge's fill, 0 to 1 (the boss breaks at 1); negative hides the gauge. |
| `staggerCells?` | `number` | Cells the stagger gauge is cut into (10). |
| `weaknesses?` | `ReadonlyArray<BossBarWeakness>` | The elements the boss is weak to, keyed rows. |
| `invulnerable?` | `boolean` | Health remains visible, greyed, and a worded invulnerability state is shown. |
| `defeated?` | `boolean` | The game has confirmed defeat; this does not follow automatically from zero health. |
| `loading?` | `boolean` | Waiting for boss data; hides stale health and phases. |

Each `BossBarPhase`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `label?` | `string` |  |
| `complete?` | `boolean` |  |

Each `BossBarWeakness`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name?` | `string` | For a reader. |
| `icon?` | `Texture \| 0` | A texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon name, in art's place. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `critical`, `invulnerable`, `defeated`, `loading`, `overflow`.

## 34. Celebration

A reward over the HUD: confetti thrown from the top edge, a burst's ring and sparks, a level-up's ring of light with its word in gold, its new level and embers rising under it, or fireworks' five shells. Each play runs once per shot, with the pack's reward sound. The particles are the engine's emitters drawn by `Particles` (./fx/*.ogt), one node per ink, so a pack's roles colour them; reduced motion draws none. The level-up's rays wait for pack art.

`import { Celebration } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/celebration.tsx`).

| Prop | Type | What |
|---|---|---|
| `kind` | `string` | Which one: confetti (falling over its box from the top edge), burst (a ring and sparks thrown from its centre), levelUp (a ring of light behind the title and the new level, embers rising under them) or fireworks (shells bursting across its top). |
| `shots?` | `ReadonlyArray<{ id: string }>` | Its plays, each { id } (the tick it fired, as text): a new id plays it again; a kept one plays once. |
| `title?` | `string` | levelUp: the big word; empty is Level up. |
| `value?` | `string` | levelUp: the new level (12); empty is none. |

Fixtures: `default`, `confetti`, `burst`, `fireworks`.

## 35. RpgCombatLog

The combat log: a small band over a scroll of lines, the newest at the foot, each inked by its tone (damage red, heal green, crit warn; status reads plain, as the style's tone table words it). It scrolls inside the height it is given and fades at its foot. Read only.

`import { RpgCombatLog } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/combat-log.tsx`).

| Prop | Type | What |
|---|---|---|
| `lines?` | `ReadonlyArray<CombatLogLine>` | The log, oldest first. |

Each `CombatLogLine`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `text` | `string` | The line as the game words it (Sir Aldric uses Shield Bash on Cultist.). |
| `tone?` | `string` | damage, heal, status, crit or plain inks the line. |
| `detail?` | `string` | A quieter second line (→ 42 damage), empty for none. |

Fixtures: `default`.

## 36. Combo

The combo readout on its HUD plate: the heading, the count and its multiplier as figures, what state the chain is in, and the window's progress while it runs. A status: its changes announce. Read only; the game owns the count and the window.

`import { Combo } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/combo.tsx`).

| Prop | Type | What |
|---|---|---|
| `count?` | `number` | The game's combo count, shown at zero too. Default 0. |
| `multiplier?` | `number` | The game's multiplier; it is not derived from count. Default 1. |
| `remaining?` | `number` | Remaining window, 0 to 1; negative hides the timer. The game advances it. Default -1. |
| `state?` | `string` | idle, active or broken; zero remaining does not determine gameplay results. Default idle. |
| `label?` | `string` | A custom heading; empty uses Combo. Default "". |

Fixtures: `default`, `idle`, `broken`, `overflow`.

## 37. Cooldown

A scene cell a game lays in a strip: its icon, the share still to run darkened over it and the seconds reading on their halo, popping when it comes ready. The sweep is a nine-disc wipe running counterclockwise from the top, as the .svelte's conic mask did.

`import { Cooldown } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/cooldown.tsx`).

| Prop | Type | What |
|---|---|---|
| `remaining` | `number` | Seconds left; 0 is ready. |
| `total?` | `number` | The whole cooldown in seconds. Default 1. |
| `icon` | `string` | An icon of the atlas under the sweep. |
| `label` | `string` | Its accessible name (Dash). |
| `onUse?` | `(e: UiEvent) => void` | Use. |

Pack slots: `Slot.frame`.

Fixtures: `default`.

## 38. Crosshair

The reticle: four arms and a dot (or a ring and a dot), opening with the weapon's spread, the target's ink when the aim rests on an enemy, and four diagonal ticks confirming the last hit (red with a glow for a headshot and a kill, the shield's blue for a broken one). Geometry is resolved into styles; the ring's mask is a border, and a new hitAt does not replay the mark while it shows (no keyed surface): the ticks pop in when the mark appears.

`import { Crosshair } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/crosshair.tsx`).

| Prop | Type | What |
|---|---|---|
| `spread` | `number` | The weapon's current spread, 0 (settled) to 1 (widest): the arms move out from the centre. |
| `hit?` | `string` | The last hit to confirm: none, hit, headshot, kill or shieldBreak. The game clears it to none when its mark should go. Default none. |
| `hitAt?` | `number` | The id of that hit (its tick): a new id plays the mark again. |
| `aimed?` | `boolean` | The aim rests on an enemy: the reticle takes the target ink. |
| `shape?` | `string` | cross (four arms and a dot), dot, or circle (a ring and a dot). Default cross. |

Fixtures: `default`, `headshot`, `kill`, `circle`.

## 39. DamageDirection

Where recent hits came from: a mark at the screen's edge toward each hit. The game adds a hit and removes it when its mark should go. Read only.

`import { DamageDirection } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/damage-direction.tsx`).

| Prop | Type | What |
|---|---|---|
| `hits?` | `ReadonlyArray<DamageDirectionHit>` | Recent hits. The game removes a hit when its mark should go. |

Each `DamageDirectionHit`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `angle` | `number` | In radians from straight ahead, clockwise, relative to the view. |
| `strength?` | `number` | 0 to 1: the arc's opacity and length. |
| `kind?` | `string` | normal, shield (the shield took it) or critical. |

Fixtures: `default`, `crowded`.

## 40. DamageNumbers

Damage numbers rising from where each hit landed: one number per hit, anchored at the unit (or the world point) in the hit's colour, held in the tag halo so it reads over the scene. Each new row plays the float (rise and fade) once. Read only. The old anchor's pooled GPU mass figures (one instanced draw for a battle's numbers, the rise eased on the GPU) have no TSX surface: each number is a text node and the float runs on the built-in animate-float.

`import { DamageNumbers } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/damage-numbers.tsx`).

| Prop | Type | What |
|---|---|---|
| `hits?` | `ReadonlyArray<DamageHit>` | The hits: each { id, entity, at, amount, color } kept for the rise's length, id unique per hit, entity the unit by its bits (0: at, a world point x y z), color #rrggbb (empty is white). Each new row spawns one number on the GPU that rises and fades. |

Each `DamageHit`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `entity` | `number` | The unit hit, by its bits (0: at, a world point x y z). |
| `at` | `string` | A world point x y z without an entity. |
| `amount` | `number` | The damage, shown as the number. |
| `color?` | `string` | The number's ink as #rrggbb (empty is white). |

Fixtures: `default`.

## 41. HudEffect

One buff or debuff of a HUD, a block an effects row is built from: a small tile whose edge is its fill (green for a boon, red for a harm), its art inset inside it, the time already run darkened round the art and its time or stacks on a tag under it. The game owns every timer; read only. The tile's cut corner has no TSX surface: it is a rounded one.

`import { HudEffect } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/effect.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | What a reader calls it (Haste). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `remaining?` | `number` | The share of its time still to run, 0 to 1: a dark radial sweep over the art; negative is no timer. |
| `label?` | `string` | Its time or stacks as the game words it (0:12, ×3), on a tag under the tile; empty is none. |
| `harmful?` | `boolean` | A debuff (on data-harmful, never colour alone: the edge and the tag's ink both change). |

Pack slots: `Chip.frame`.

Fixtures: `default`, `harmful`, `untimed`, `empty`, `overflow`.

## 42. HeroButton

A hero action's button: its words in capitals and a keycap beside them, on a face that stands on its shadow (primary's primary, premium's gold, secondary's glass, danger's danger); it lifts under a pointer and sinks while held, and shows inert when disabled. The disabled face's greying filter has no TSX surface: it shows dimmed instead.

`import { HeroButton } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hero-button.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | Its words (End turn). |
| `tone?` | `string` | primary (empty), premium, secondary or danger. |
| `disabled?` | `boolean` | Shown but inert (Waiting). |
| `key?` | `string` | A keycap beside the words (E); empty is none. |
| `onPress?` | `(e: UiEvent) => void` | Press. |

Fixtures: `default`, `premium`, `secondary`, `danger`, `disabled`.

## 43. HudHint

One input hint of a HUD, a block a hint row is built from: a key cap and what it does in small tracked capitals. Read only; hidden under a touch screen, where the controls on screen say it. No strings of its own; the svelte file carries none either.

`import { HudHint } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hint.tsx`).

| Prop | Type | What |
|---|---|---|
| `key?` | `string` | The input as the game names it (Tab, Space), on a key cap; empty is none. |
| `label?` | `string` | What it does (Target), in small tracked capitals. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `empty`, `overflow`.

## 44. HoldButton

A confirm that must be held (Delete save): its icon and words, what it says while it is held, and a line under it saying what to do. Holding it for 1.2 s confirms, as does the input action it names; the hold's progress runs along the button's foot and the words swap while held.

`import { HoldButton } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hold-button.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | Its words and accessible name (Delete save). |
| `holding?` | `string` | What it says while it is held (Keep holding…); empty keeps its label. |
| `hint?` | `string` | A line under it, what to do (Hold to delete); empty for none. |
| `icon?` | `string` | An icon of the atlas before the label; empty for none. |
| `action?` | `string` | An og-input action whose hold holds it too (confirm); empty for none. |
| `variant?` | `string` | Its role: danger (the default, a destructive confirm), primary or secondary. Default danger. |
| `disabled?` | `boolean` | Shown but inert. |
| `onConfirm?` | `(e: UiEvent) => void` | Confirmed, by holding it 1.2 s or the input action it names. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `label-only`, `disabled`, `overflow`.

## 45. HudTimer

A HUD clock on its plate: a caption, the seconds in the HUD's large figure, and for a countdown with a total a ring of what is left, which warns and then turns red near zero; paused, the time dims and breathes. Read only.

`import { HudTimer } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hud-timer.tsx`).

| Prop | Type | What |
|---|---|---|
| `seconds` | `number` | Seconds on the clock. |
| `total?` | `number` | A countdown's whole time: a ring of what is left beside it (0: none). |
| `countdown?` | `boolean` | Counts down (warns and turns red near zero); false is a stopwatch. |
| `warnAt?` | `number` | Seconds left when it turns warn; a third of it is critical. |
| `paused?` | `boolean` | Paused: the time dims and breathes. |
| `label?` | `string` | A caption over it (Round 2); empty is none. |

Pack slots: `Plate.frame`.

Fixtures: `default`.

## 46. Killfeed

Eliminations at the screen's top corner, newest first: each row the killer, the weapon's name and the victim, a headshot tag, names in their side's colour on a dark glass that fades toward the middle, an edge at the screen's side lit for the player's own rows. Read only. The weapon glyphs (svg silhouettes) have no TSX surface: every weapon shows in words. The rows' slide and fade transitions have no TSX surface and are dropped; a phone's compact trimming is the game's to do in the prop.

`import { Killfeed } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/killfeed.tsx`).

| Prop | Type | What |
|---|---|---|
| `rows?` | `ReadonlyArray<KillfeedRow>` | The eliminations to show, newest first. The game trims old rows (a phone shows the two newest upright, three on its side); a new row slides in and an old one stays whole until it fades out over its last 300 ms. |

Each `KillfeedRow`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `killer` | `string` | The killer's name. |
| `victim` | `string` | The victim's name. |
| `weapon?` | `string` | The weapon's name. |
| `kind?` | `string` | The weapon's class (rifle, smg, pistol, shotgun, sniper, launcher or melee); its glyph is drawn from it, empty for the weapon's name in words. |
| `headshot?` | `boolean` |  |
| `self?` | `boolean` | The player took part (the row's edge is lit and the player's name takes the HUD's accent). |
| `team?` | `string` | Ally or enemy, the killer's side (the victim takes the other). |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `words`, `long`.

## 47. HudLootEntry

One row of a HUD's loot feed, a block a feed is built from: an icon, the item's name, its rarity on a tag and the count gained, on a dark glass that fades toward the middle with its edge at the screen's side tinted by rarity; a fading row dims. Read only. The art's cut corner (corner-shape) has no TSX surface: a rounded one stands in.

`import { HudLootEntry } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/loot-entry.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The item's name (Ember Blade). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `rarity?` | `string` | common, uncommon, rare, epic or legendary: the edge's colour (on data-rarity); an epic or legendary item's name is inked too. |
| `label?` | `string` | The rarity as the game words it (Epic), on a tag; empty is none. |
| `quantity?` | `number` | The count gained, written ×3 (or +1,240 when currency is true); 0 hides it. |
| `currency?` | `boolean` | The quantity is money (gold): written as a gain. |
| `fading?` | `boolean` | An old row on its way out: dimmed. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `gold`, `fading`, `empty`, `overflow`.

## 48. OrbMeter

A glass vessel in a bezel, over the scene: a bezel with its light and shade, a socket, a dark well in its liquid's hue with the liquid standing to the value, a surface line, a gleam blob riding the surface and the figure in the middle. The style's SVG waves, glare arcs and ornament have no TSX surface: the liquid is a flat fill with a surface line, the glare a white blob, the ornament dropped (the default pack draws none). The packs' --orb-meter-* tokens are resolved to the dark scheme's values here.

`import { OrbMeter } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/orb-meter.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The value. |
| `max?` | `number` | Its maximum. Default 1. |
| `kind?` | `string` | Its liquid's colour: health or mana. |
| `showValue?` | `boolean` | The value in the middle. |
| `label?` | `string` | A caption under it; empty is none. |
| `name?` | `string` | What a screen reader calls it; empty is the label, else Health. |

Fixtures: `default`, `mana`, `empty`, `full`.

## 49. OverlayBar

A map's overlays over the scene: a legend for the selected overlay (its name, its ramp low to high and the words at its ends) over a bar of buttons, one per overlay, its picture or icon beside its name; the bar scrolls along when the buttons run over. Read only; the buttons report their choice. The buttons' pressed state is worded, as the style words it; a pack's picture fills the button's face.

`import { OverlayBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/overlay-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `overlays?` | `ReadonlyArray<OverlayBarOverlay>` | The map overlays, keyed rows. |
| `selected?` | `string` | The id of the overlay shown (its legend over the bar, its button lit and named); empty is none. |
| `label?` | `string` | What a screen reader calls the bar; empty is Overlays. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Show an overlay. |

Each `OverlayBarOverlay`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | Its name (Heat). |
| `icon?` | `string` | An icon name shown when there is no image. |
| `image?` | `Texture \| 0` | A texture handle (0: none). |
| `ramp?` | `string` | The legend's gradient stops low to high (#2b6cb0, #f6e05e, #e53e3e). |
| `low?` | `string` | The word at the ramp's low end. |
| `high?` | `string` | The word at the ramp's high end. |

Fixtures: `default`, `none`.

## 50. PerfReadout

A HUD plate of how the game runs: the connection state in its tone's colour, then four metrics — frames, frame time, ping and packet loss — each its caption over its figure, Unavailable where the game has no reading. The game supplies every value and threshold; the document samples no clocks or network. Read only.

`import { PerfReadout } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/perf-readout.tsx`).

| Prop | Type | What |
|---|---|---|
| `fps?` | `number` | Frames per second; negative means unavailable, zero is a real reading. |
| `frameMs?` | `number` | Frame time in milliseconds; negative means unavailable. |
| `ping?` | `number` | Round-trip milliseconds; negative means unavailable. |
| `loss?` | `number` | Packet loss as a fraction, 0 to 1; negative means unavailable. |
| `status?` | `string` | offline, connecting, online or degraded. The game supplies connection state and thresholds; the document samples no clocks or network. |

Fixtures: `default`, `degraded`, `offline`, `connecting`, `unknown`, `zero`.

## 51. PickupFeed

A HUD plate of what the player just picked up: its heading, then its entries, each its icon, name and caption with the signed change in its trend's colour; the game owns ordering, expiry and retention, and the empty state says so. Read only.

`import { PickupFeed } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/pickup-feed.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The feed heading; empty uses Pickups. |
| `items?` | `ReadonlyArray<PickupFeedItem>` | Keyed entries. The game owns ordering, expiry and retention; quantity is a signed change. No timers run in this document. |
| `loading?` | `boolean` | Waiting for pickup data; stale entries are hidden. |

Each `PickupFeedItem`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | Its name (Iron ore). |
| `icon?` | `string` | An icon of the atlas; empty is none. |
| `quantity?` | `number` | A signed change. |
| `detail?` | `string` | Its caption (Crafting material); empty is none. |

Fixtures: `default`, `loading`, `overflow`.

## 52. HudPortrait

A portrait of a HUD, a block a plate is built from: the art (or the person glyph) inset in a lit edge. Its side is the HUD portrait size; a plate sets it on the place it gives the portrait. Read only. The style's octagon and its greying filter have no TSX surface: the edge is a rounded one and the dimmed art shows dimmed.

`import { HudPortrait } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/portrait.tsx`).

| Prop | Type | What |
|---|---|---|
| `portrait?` | `Texture \| 0` | A head-and-shoulders texture (og.load.texture); 0 draws the person glyph. |
| `dimmed?` | `boolean` | Out of the fight: the art greyed. |
| `class?` | `string` | Extra classes for the root, to size it (NSH style). |

Fixtures: `default`, `dimmed`.

## 53. ProgressBar

A bar fills the width its container gives it: a caption over it at the start and its value in words at the end, a paler buffer ahead of the fill in a dark well with one lit rim, thinner than a vital's; with no known value a band sweeps the track instead. It sits on a HUD plate, so its words read over any scene. The style's lean (skewX) and the fill's sheen have no TSX surface: the lean is dropped and the sweeping band carries the shimmer.

`import { ProgressBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/progress-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | Done, 0 to 1. |
| `buffer?` | `number` | A paler fill ahead of value (downloaded ahead of installed), 0 to 1. |
| `label?` | `string` | A caption over it at the start; empty is none. |
| `valueText?` | `string` | The value in words at the end; empty is the percent. |
| `showValue?` | `boolean` | Show the value at the end. |
| `indeterminate?` | `boolean` | Working with no known value: a band sweeps along the track. |
| `tone?` | `string` | The fill's tone: primary (empty), success, warn, danger or info. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `indeterminate`, `success`, `danger`.

## 54. ProgressRing

A ring that fills with its value, its percent (or the label's words) in the middle; with no known value the ring turns instead. The style's SVG arc is the pack's ring art here, and the spinning ring turns as a lit rim. Read only.

`import { ProgressRing } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/progress-ring.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | Done, 0 to 1. |
| `spinning?` | `boolean` | Working with no known value: the arc turns (indeterminate). |
| `label?` | `string` | Text in the middle; empty is the percent (none while spinning). |
| `name?` | `string` | What a screen reader calls it; empty is Progress, or Loading while spinning. |
| `tone?` | `string` | The fill's tone: primary (empty), success, warn, danger or info. |

Fixtures: `default`, `spinning`, `labelled`, `danger`.

## 55. QtePrompt

A keyed prompt over the scene: what to attempt, a big trigger (press, hold or mash — the hold runs holdMs with its fill, and confirms when it completes), its state in words, and the game-supplied time left (and, for a mash, the response progress) as thin bars. Only the active state accepts input; the game owns resolution and expiry. Read only beyond the respond.

`import { QtePrompt } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/qte-prompt.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The action to attempt; empty uses Respond. |
| `keyLabel?` | `string` | The displayed input hint, not an input binding; empty hides it. |
| `action?` | `string` | An og-input action bound by the game. Pointer, focused keyboard/pad and this action activate the same control. |
| `mode?` | `string` | press, hold or mash. Hold emits once after holdMs; mash emits one respond per activation. The game counts responses and judges success. |
| `holdMs?` | `number` | The minimum hold duration in milliseconds, at least 1. Releasing early cancels without an intent. |
| `remaining?` | `number` | Time left as a fraction, 0 to 1. The game advances it; no clock runs here and zero does not choose a result. |
| `progress?` | `number` | Game-supplied completion fraction, 0 to 1; displayed for mash prompts. |
| `state?` | `string` | active, waiting, success, failure or expired. Only active accepts input; the game owns resolution and expiry. |
| `disabled?` | `boolean` | Temporarily unavailable even while active. |
| `onRespond?` | `(e: UiEvent) => void` | Respond; the game owns resolution. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `hold`, `mash`, `success`, `failure`, `expired`, `waiting`, `disabled`, `overflow`.

## 56. RpgQuestTracker

The tracked quest on the HUD: a heading in gold, the quest's name, then its objectives, the done ones struck through, the one in play lit with its progress on a tag and the upcoming ones dim. Read only; the journal owns choosing what to track.

`import { RpgQuestTracker } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/quest-tracker.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | The tracker's heading as the game words it (Side quest); empty takes the component's Main quest. |
| `name` | `string` | The tracked quest's name; empty hides the tracker. |
| `steps` | `ReadonlyArray<{ id: string; text: string; done: boolean; tag?: string }>` | Its objectives in order, keyed rows of { id, text, done, tag }: done ones are struck and dimmed with a filled diamond; the first not done is the step in play (lit, on data-current); the ones after it are upcoming (dim); tag its progress as the game words it (1 / 3) on a tag, empty for none. A phone shows the step in play alone. |
| `distance` | `string` | How far its marker is, as the game words it (312 m), on a tag beside the name; empty hides it. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `done`, `overflow`.

## 57. RadialMenu

A radial menu: slices clockwise from the top, the one the pointer or stick aims at lit, and its detail in the centre. It reports onAim and onChoose; the game decides what a slice does.

`import { RadialMenu } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/radial-menu.tsx`).

| Prop | Type | What |
|---|---|---|
| `items?` | `ReadonlyArray<RadialMenuItem>` | The slices, clockwise from the top. |
| `span?` | `number` | One slice's share of the turn (1 / n). |
| `chosen?` | `string` | The slice the pointer or stick points at (its id); empty is none. |
| `detail?` | `string` | The centre's line for the chosen slice; empty shows its label. |
| `onAim?` | `(e: UiEvent) => void` | Aim: the pointer moved. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a slice. |

Each `RadialMenuItem`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `label` | `string` | Its label (Heal). |
| `icon?` | `string` | An icon of the atlas. |
| `at` | `number` | Its centre as a turn fraction (slice i of n: i / n). |
| `cost?` | `string` | Its cost as the game words it (300); empty is none. |
| `unaffordable?` | `boolean` | It cannot be paid for. |
| `disabled?` | `boolean` |  |

Fixtures: `default`.

## 58. RadialMeter

A stamina wheel: up to three rings that drain and fill, red once exhausted until full again, faded out when the game hides it. Read only.

`import { RadialMeter } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/radial-meter.tsx`).

| Prop | Type | What |
|---|---|---|
| `value?` | `number` | The value in wheels: 1 is one full wheel, up to 3. |
| `wheels?` | `number` | Wheels (an extended stamina's second and third ring), 1 to 3. Default 1. |
| `exhausted?` | `boolean` | Ran out: red until full again. |
| `hidden?` | `boolean` | Faded out (the game hides it once it has been full a while). |
| `name?` | `string` | What a screen reader calls it; empty is Stamina. |

Fixtures: `default`.

## 59. HudSlot

One slot of a HUD's skill bar, a block a bar is built from: the art in an octagon with a lit edge, its cooldown a dark sweep with a lit leading edge and the time left, a slot that cannot be paid for greyed with its reason on it, charge pips over a slot that holds charges, a consumable's count on a tag, and the ultimate in a hexagon inside a gold ring that fills with its charge; a key cap under it. Its side is --hud-slot-size. The game owns every cooldown, cost and charge; read only. The style cuts the cell with clip-path; here the cell is rounded and the sweep and the ring are pack art (a nine-disc wipe and a nine-ring wipe).

`import { HudSlot } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/slot.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | What a reader calls it (Ember slash). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `key?` | `string` | The input as the game names it (1, Q), on a key cap under the slot; empty is none. |
| `kind?` | `string` | skill, item (a consumable, its count on a tag) or ultimate (a hexagon inside a ring, gold by default, that fills with its charge). |
| `cooldown?` | `number` | The share of its cooldown still to run, 0 to 1: a dark sweep with a lit leading edge; 0 is none. |
| `time?` | `string` | The cooldown left as the game words it (12), over the sweep; empty is none. |
| `count?` | `number` | How many are left, on a tag at the slot's corner; 0 shows none. |
| `ready?` | `boolean` | Ready and worth pressing now: a lit edge and a glow (an ultimate at full charge has them without it). |
| `unaffordable?` | `boolean` | It cannot be paid for: the art greyed under a veil. |
| `reason?` | `string` | Why it cannot be used (No mana), on the slot while it is unaffordable and not cooling; empty is none. |
| `charges?` | `number` | Charges held, the lit pips over the slot. |
| `chargesMax?` | `number` | Charges it holds; 0 draws no pips. |
| `charge?` | `number` | An ultimate's charge, 0 to 1: its ring's lit arc and, under 1, the percent on it. |
| `class?` | `string` | Extra classes for the root, to size it (NSH style). |

Pack slots: `Slot.frame`.

Fixtures: `default`, `cooling`, `item`, `ultimate`, `unaffordable`, `empty`, `overflow`.

## 60. StarBar

A score against its thresholds: the figure over a track that fills in the style's gold, each threshold a star set on the track where it falls, lit once the score reaches it. The stars are the style's rating stars, the same as a level's or a results screen's. The style draws SVG stars; here each is the atlas star icon in a seat, which keeps their shape and colour but not the style's glow.

`import { StarBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/star-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `value?` | `number` | The score. |
| `max?` | `number` | The bar's full score (its last threshold). Default 1. |
| `stars?` | `ReadonlyArray<StarBarStar>` | The thresholds: lit once the score reaches each. |
| `label?` | `string` | A caption over the score; empty is none. |
| `valueText?` | `string` | The score in words instead of the grouped number. |
| `compact?` | `boolean` | One row: the score beside a shorter bar. |

Each `StarBarStar`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `at` | `number` | Where it sits on the track, a fraction of max (0 to 1). |

Pack slots: `Plate.frame`.

Fixtures: `default`.

## 61. StatBar

A stat with its bar: the icon and label before a leaning well that fills to value in its kind's colour, the heal and recoverable shares past it, the shield along its top, a notch at each block's end, and the figure at its end (inside the track on valueInside). Health turns warn under low and danger under critical. Read only. The old style's lean (skewX) and the blocks' mask have no TSX surface: the blocks are drawn as notch marks, the lean and the figure's font-stretch are dropped, and the ghost holds the fill's width instead of draining behind it.

`import { StatBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/stat-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The value. |
| `max?` | `number` | Its maximum. Default 1. |
| `kind?` | `string` | What it measures (its colour): health, mana, stamina or xp; health turns warn and danger as it falls. Default health. |
| `segment?` | `number` | Value per block: a notch at each block's end (0: one bar). |
| `shield?` | `number` | Shield or armour on top of the value, a strip over the bar. |
| `shieldMax?` | `number` | The most shield; 0 hides the strip. |
| `heal?` | `number` | Healing on its way, a pale preview past the fill. |
| `recoverable?` | `number` | Damage that can still come back, drawn grey between the value and this. |
| `label?` | `string` | A short name at its start (HP); empty is none. |
| `icon?` | `string` | An icon of the atlas before it; empty is none. |
| `showValue?` | `boolean` | The value as text at its end. |
| `valueInside?` | `boolean` | The value inside the track at its end, so the bar keeps its length. |
| `valueText?` | `string` | The value as text instead of value / max. |
| `low?` | `number` | The fraction under which it is low (0.3). Default 0.3. |
| `critical?` | `number` | The fraction under which it is critical (0.15). Default 0.15. |
| `bare?` | `boolean` | No HUD plate behind it. |
| `name?` | `string` | What a screen reader calls it; empty is the label, else Health. |

Fixtures: `default`.

## 62. StatusGrid

Buffs and debuffs as a grid of tiles, the debuffs in their own row bordered by type, and the effects past the shown ones folded into a +n tile. Read only.

`import { StatusGrid } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/status-grid.tsx`).

| Prop | Type | What |
|---|---|---|
| `buffs?` | `ReadonlyArray<StatusGridEffect>` | The buffs, in the game's order. |
| `debuffs?` | `ReadonlyArray<StatusGridEffect>` | The debuffs, in their own row and bordered by type. |
| `more?` | `number` | Effects past the shown ones, folded into a +n tile (0: none). |

Each `StatusGridEffect`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `icon` | `string` | An icon of the atlas. |
| `name` | `string` | What a reader calls it (Haste). |
| `stacks?` | `number` | How many are stacked; 1 is plain. |
| `remaining?` | `number` | Seconds left (0 lasts until removed). |
| `duration?` | `number` | Seconds it runs (0 lasts until removed; past it, an extended effect, sweeps nothing). |
| `expiring?` | `boolean` | In its last five seconds. |
| `type?` | `string` | magic, curse, poison, disease or bleed (debuffs only). |

Fixtures: `default`.

## 63. TouchControls

On-screen controls for touch: a stick (or a d-pad) at the start side feeding a move action, an optional look stick and the action buttons at the end side. They feed og.input's actions and show only where touch is the last input, unless always is set.

`import { TouchControls } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/touch-controls.tsx`).

| Prop | Type | What |
|---|---|---|
| `move?` | `string` | The two-axis og-input action the stick at the start side feeds (move); empty leaves the stick out. |
| `pad?` | `boolean` | A d-pad in the stick's place: eight ways, each axis -1, 0 or 1. |
| `look?` | `string` | A second stick at the end side and the two-axis action it feeds (look); empty leaves it out. |
| `deadZone?` | `number` | The sticks' dead zone as a part of their radius, up to 0.95; 0 is the engine's 0.15. |
| `actions?` | `ReadonlyArray<TouchAction>` | The buttons at the end side, the first nearest the corner: keyed rows of { id, action, label, icon, primary, charge, disabled }, each as ActionButton takes them. |
| `always?` | `boolean` | Shown under a mouse too (a touch laptop, a preview). Otherwise the controls show only where the primary pointer is a finger. |
| `disabled?` | `boolean` | Every control feeds nothing and shows faded (a cutscene, a menu over the HUD). |

Each `TouchAction`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `action` | `string` |  |
| `label` | `string` |  |
| `icon?` | `string` |  |
| `primary?` | `boolean` |  |
| `charge?` | `number` | A charge round its rim, 0 to 1; negative is none. Default -1. |
| `disabled?` | `boolean` |  |

Fixtures: `default`, `pad`, `twin`, `many`, `disabled`.

## 64. IdleGenerators

The generator list of an idle game: one cut row per producer with its art in a socket and its level on a hexagon at the socket's foot, its name and how many are owned, its yield in green, its running cycle as a leaning bar with the time left, and its Auto and boost tags. The game runs the cycles and owns every number; the list reports choose (a row's key) and the chosen row is washed in the accent. A locked row is disabled and greyed, and says Locked with what unlocks it. It fills its height and scrolls, its foot fading while there is more.

`import { IdleGenerators } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/generators.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The list's heading; empty takes the group's (Generators). |
| `generators?` | `ReadonlyArray<{ id: string; icon: string; name: string; owned: number; rate: string; cycle: number; cycleText: string; level: number; boost: string; auto: boolean; locked: boolean }>` | The producers in the game's order, keyed rows of { id, icon, name, owned, rate, cycle, cycleText, level, boost, auto, locked }: owned how many the player has (a number, compacted; 0 hides it); rate what it yields as the game words it (+1.2K/s; on a locked row what unlocks it); cycle the running cycle 0 to 1 (negative hides the bar); cycleText the time left as the game words it (empty shows the percent); level its level (0 hides it); boost a multiplier as the game words it (×2; empty hides it); auto it runs by itself (named Auto); locked not yet available (named Locked, disabled). |
| `selected` | `string` | The generator the player last chose, selected; empty is none. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.selected`, `Slot.frame`.

Fixtures: `default`, `empty`, `disabled`, `selected`, `overflow`.

## 65. IdleOffline

The welcome-back panel of an idle game: how long the player was away, large (the game's words, or its seconds as a duration), then what was gathered as sockets glowing gold from their foot, each with its art, the amount and its name. Collect reports collect; Double, the game's gold offer and shown only when it sets canDouble, reports double with what it asks under its word. The game computes the haul and applies it. Many currencies wrap and the haul scrolls.

`import { IdleOffline } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/offline.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading; empty takes the group's (Welcome back). |
| `away` | `string` | How long the player was away, as the game words it (it wins; empty shows seconds as a duration: 1h 2m). |
| `seconds` | `number` | How long the player was away, in seconds, when away is empty. |
| `earned?` | `ReadonlyArray<{ id: string; icon: string; name: string; amount: string }>` | What was gathered while away, keyed rows of { id, icon, name, amount }: amount as the game formats it (+12.4K). |
| `canDouble?` | `boolean` | The game offers to double the haul (an ad, a gem); false hides Double. |
| `doubleText` | `string` | What doubling costs or asks, as the game words it (Watch an ad); empty is none. |
| `onCollect?` | `(e: UiEvent, key?: string) => void` | The collect event, with the row's key where it comes from a row. |
| `onDouble?` | `(e: UiEvent, key?: string) => void` | The double event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`.

Fixtures: `default`, `disabled`, `empty`, `overflow`.

## 66. IdlePrestige

The prestige panel of an idle game: the currency in a socket glowing in the epic purple, what the player holds and, in that purple, what a reset would give; each changed bonus now and after the reset; and the requirement with its leaning bar, green once met, and Ready or Not yet in words. Prestige reports request; the game then sets confirming and the panel asks inline, Reset (confirm) and Cancel (cancel, or Escape and pad B while asking). Cancel comes first so no key resets by accident. The game decides whether a reset is allowed and what it gives.

`import { IdlePrestige } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/prestige.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading; empty takes the group's (Prestige). |
| `currency` | `string` | The prestige currency's name (Souls). |
| `icon` | `string` | The prestige currency's icon. |
| `held` | `string` | How much of it the player holds now, as the game formats it. |
| `gain` | `string` | How much a reset now would give, as the game formats it. |
| `rows?` | `ReadonlyArray<{ id: string; icon: string; label: string; now: string; after: string }>` | What changes, keyed rows of { id, icon, label, now, after }: now the value before the reset and after the value it would be after, both as the game words them. |
| `requirement` | `string` | What a reset needs, as the game words it (Reach 1M gold); empty hides it. |
| `progress?` | `number` | Toward the requirement, 0 to 1, for its bar; negative hides the bar. Default -1. |
| `progressText` | `string` | That progress in words (640K / 1M); empty shows the percent. |
| `met?` | `boolean` | The requirement is met: Prestige is enabled and the requirement reads Ready (else Not yet). |
| `confirming?` | `boolean` | The game is asking the player to confirm (set after request): Reset and Cancel stand in for Prestige. |
| `onCancel?` | `(e: UiEvent, key?: string) => void` | The cancel event, with the row's key where it comes from a row. |
| `onConfirm?` | `(e: UiEvent, key?: string) => void` | The confirm event, with the row's key where it comes from a row. |
| `onRequest?` | `(e: UiEvent, key?: string) => void` | The request event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Row.frame`.

Fixtures: `default`, `disabled`, `confirming`, `empty`, `overflow`.

Actions: `back` (Cancel).

## 67. IdlePurchases

The upgrade shop of an idle game: a ×1 / ×10 / ×100 / Max choice of how many to buy (chooseamount, the amount's key), then one row per upgrade with its icon, name, effect, level, how many the chosen amount buys and what that costs, and Buy (buy, the upgrade's key). The game prices every amount and decides what can be bought: Buy is enabled only when the row is affordable and neither maxed nor locked, and each refused state is a word (Not enough, Maxed, Locked). It fills its height and scrolls.

`import { IdlePurchases } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/purchases.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The list's heading; empty takes the group's (Upgrades). |
| `amounts?` | `ReadonlyArray<{ id: string; label: string }>` | The buy amounts offered, keyed rows of { id, label }: label the amount as the game words it (it wins; empty reads the key max as Max and any other key as ×key: ×1, ×10, ×100). An empty list hides the choice. |
| `amount` | `string` | The buy amount chosen, one of the amounts' keys. |
| `upgrades?` | `ReadonlyArray<{ id: string; icon: string; name: string; effect: string; level: number; count: string; cost: string; costIcon: string; affordable: boolean; maxed: boolean; locked: boolean }>` | The purchases in the game's order, keyed rows of { id, icon, name, effect, level, count, cost, costIcon, affordable, maxed, locked }: effect what it does as the game words it; level its level (0 hides it); count how many the chosen amount buys, as the game words it (×37; empty hides it); cost the price of that many, formatted; costIcon the currency's icon; affordable the player can pay it (else named Not enough); maxed nothing more to buy (named Maxed); locked not yet available (named Locked). Buy is offered only when affordable and neither maxed nor locked. |
| `onBuy?` | `(e: UiEvent, key?: string) => void` | The buy event, with the row's key where it comes from a row. |
| `onChooseamount?` | `(e: UiEvent, key?: string) => void` | The chooseamount event, with the row's key where it comes from a row. |

Pack slots: `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Row.frame`, `Slot.frame`.

Fixtures: `default`, `max`, `disabled`, `maxed`, `empty`, `overflow`.

## 68. IdleResources

The resource bar of an idle game: one cut chip per currency with its art in a socket, its name, the amount large and the income under it, green when it rises. A store at its cap gets a gold rim and a Full tag. The game owns every number: a formatted value or rate it passes wins, else the bare number is compacted (12.3K) and counts to each new amount. While loading each chip keeps its art and name and shows a placeholder, which a pack may animate; the row's name says it is loading. It reports no intents. The chips wrap onto more lines when they do not fit.

`import { IdleResources } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/resources.tsx`).

| Prop | Type | What |
|---|---|---|
| `currencies?` | `ReadonlyArray<{ id: string; icon: string; name: string; amount: number; value: string; rate: string; perSecond: number; full: boolean }>` | The currencies in the game's order, keyed rows of { id, icon, name, amount, value, rate, perSecond, full }: value the amount as the game formats it (it wins; empty shows amount, a number, compacted: 12.3K, counting to each new amount); rate the income as the game words it (+1.2K/s; it wins; empty shows perSecond, a number per second, compacted; 0 hides it); perSecond also sets data-trend (up, down, flat) whether or not rate is given, so pass it with a worded rate too; full the store is at its cap, named Full. |
| `loading?` | `boolean` | The amounts are on their way: each row keeps its icon and name and shows a placeholder for its figures. |

Pack slots: `Chip.frame`, `Slot.frame`.

Fixtures: `default`, `loading`, `full`, `draining`, `empty`, `overflow`.

## 69. Achievements

The achievement list: a band with the title, then the overall count as a large figure over a leaning gold bar; then one entry per achievement, a card with two corners cut: its medal (a hexagon rimmed in its tier's colour), its name, description, its state in words (locked, a bar for one in progress, the date for one unlocked) and its tier on a chip; at the end the share of players who have it, large, and a gold Claim while a reward waits (Claimed once taken). A hidden achievement shows as one, hatched, with neither name nor description, until it is unlocked. On a narrow list the rarity takes a line under the words and Claim takes a line of its own. The game owns every number; the list reports claim (an achievement's key). It fills its height and scrolls inside it.

`import { Achievements } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/achievements.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The list's heading; empty takes the group's (Achievements). |
| `summary` | `string` | The overall count as the game words it (34 / 120); empty hides it with its bar. |
| `done` | `number` | The overall share unlocked, 0 to 1, for the summary bar. |
| `achievements` | `ReadonlyArray<{ id: string; name: string; about: string; icon: string; state: string; progress: number; progressText: string; date: string; hidden: boolean; rarity: string; tier: string; claimable: boolean; claimed: boolean }>` | The achievements in the game's order: keyed rows of { id, name, about, icon, state, progress, progressText, date, hidden, rarity, tier, claimable, claimed }: state locked, progress or unlocked; progress 0 to 1 and progressText (3 / 10) for one in progress; date when it was unlocked, as the game formats it, empty says Unlocked alone; hidden a secret one, shown as Hidden achievement until unlocked; rarity the share of players who have it (2.4%), empty hides it; tier common, rare, epic or legendary, shown as a word and as the medal's colour, empty hides the word; claimable a reward waiting; claimed it was taken, which hides Claim. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |

Pack slots: `Button.plate.primary`, `Panel.frame`.

Fixtures: `default`, `empty`, `overflow`.

## 70. Chat

A chat panel: channel tabs with unread badges, the message log (system lines, the player's own messages and others' with sender and time, long text wrapping and long words breaking) and an input row. The game owns the channels, the messages, moderation and the clock; the panel reports choosetab (a channel's key), send (the Send button) and onDraft (the field's text, trimmed to maxLength). The log fills the height it is given and scrolls inside it, newest at the foot.

`import { Chat } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/chat.tsx`).

| Prop | Type | What |
|---|---|---|
| `channels` | `ReadonlyArray<{ id: string; label: string; unread: number }>` | The channels, keyed rows of { id, label, unread }: unread the count the game keeps, 0 hides its badge. Empty shows no tab row. |
| `channel` | `string` | The channel shown. |
| `messages` | `ReadonlyArray<{ id: string; kind: string; sender: string; time: string; text: string }>` | The shown channel's messages, oldest first: keyed rows of { id, kind, sender, time, text }: kind player, own (the player's) or system; time as the game formats it; text wraps, and a long word breaks. |
| `draft` | `string` | The message being written, two-way: value={draft} shows it; the field reports every change through onDraft with the text trimmed to maxLength, and the game clears it after a send. |
| `maxLength` | `number` | The longest message the game takes, in characters. |
| `canSend?` | `boolean` | The game takes a message now; false disables the field and Send (muted, slow mode). |
| `notice` | `string` | A line over the field as the game words it (Slow mode: 3 s, You are muted); empty hides it. |
| `onDraft?` | `(e: UiEvent, text: string) => void` | The field's text changed (onDraft), trimmed to maxLength. |
| `onChooseTab?` | `(e: UiEvent, key?: string) => void` | The choosetab event, with the row's key where it comes from a row. |
| `onSend?` | `(e: UiEvent, key?: string) => void` | The send event, with the row's key where it comes from a row. |

Pack slots: `Button.plate.primary`, `Chip.frame`, `Field.frame`, `Panel.frame`.

Fixtures: `default`, `empty`, `disabled`, `overflow`.

## 71. DailyQuests

Today's quests: a band with the title and when they refresh (the game counts it down), then one entry per quest: its icon in an octagon socket, its name over a leaning bar with the count beside it, and its prize in a dark box at the end. A done quest is lit gold, its prize box glows and a gold Claim stands beside it; a claimed one dims behind a Claimed stamp. On a narrow list the socket goes and the prize and Claim take a line of their own. The game owns progress, rewards and the clock; the list reports claim (a quest's key).

`import { DailyQuests } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/daily-quests.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The list's heading; empty takes the group's (Daily quests). |
| `refresh` | `string` | When the quests refresh, as the game words it (Resets in 4h 12m); empty hides it. The game counts it down. |
| `quests` | `ReadonlyArray<{ id: string; name: string; icon: string; progress: number; progressText: string; reward: string; rewardIcon: string; state: string }>` | Today's quests in the game's order: keyed rows of { id, name, icon, progress, progressText, reward, rewardIcon, state }: progress 0 to 1 and progressText (3 / 5); reward the prize in words (50) with its icon; state active, done (its reward waiting) or claimed. |
| `loading?` | `boolean` | Today's quests are on their way: a spinner stands in. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |

Pack slots: `Button.plate.primary`, `Panel.frame`.

Fixtures: `default`, `empty`, `loading`, `overflow`.

## 72. DailyRewards

A login reward calendar: a grid of day tiles with two corners cut, each a header strip with its day, the reward's icon large in the middle, its amount, and a foot with its state in words and a mark. Today is lit gold (a gold header and foot, a glow round the tile); a day taken carries a check stamp over its dimmed art; a missed day is hatched with its word in danger; a milestone is violet with a ribbon. The game owns the schedule, the rewards and the clock; the calendar reports claim (today's key) while canClaim holds: only then is today's tile a button; every other day is a labelled tile that takes no focus and reports nothing. A day shows the word of the state the game gave it. Seven across on a desktop, four on a narrow panel, three on a phone, where a last day alone on its row takes the whole row.

`import { DailyRewards } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/daily-rewards.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The calendar's heading; empty takes the group's (Daily rewards). |
| `refresh` | `string` | When the next day's reward opens, as the game words it; empty hides it. |
| `days` | `ReadonlyArray<{ id: string; label: string; icon: string; amount: string; state: string; milestone: boolean }>` | The calendar's days in order: keyed rows of { id, label, icon, amount, state, milestone }: label the day in words (Day 1); amount the reward in words (x100); state claimed, today, upcoming or missed, each shown by its own word: the game sends claimed for today once its reward is taken; milestone a big reward, lit violet with a ribbon. |
| `canClaim?` | `boolean` | Today's reward is waiting: the day whose state is today is a Claim button; otherwise no day is a button. |
| `compact?` | `boolean` | Its place is narrow (under about 300 px): the days (their state words go) tighten. The place decides, not the screen. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`, `disabled`, `overflow`.

## 73. Friends

The friends panel: a band with the title and Add friend, the friend requests with Accept and Decline, then the friends with an avatar, the name, the status in words beside a mark (online, in game, away, offline), what they are doing and Invite. The game owns the list, presence and whether an invite may go; the panel reports add, accept and decline (a request's key), choose and invite (a friend's key). It fills its height and scrolls inside it.

`import { Friends } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/friends.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading; empty takes the group's (Friends). |
| `requests` | `ReadonlyArray<{ id: string; name: string; avatar: string; note: string }>` | Friend requests waiting for an answer: keyed rows of { id, name, avatar, note }: avatar an icon of the style's atlas; note a second line as the game words it (3 mutual friends), empty hides it. Empty hides the section. |
| `friends` | `ReadonlyArray<{ id: string; name: string; avatar: string; status: string; activity: string; canInvite: boolean; invited: boolean }>` | The friends in the game's order: keyed rows of { id, name, avatar, status, activity, canInvite, invited }: status online, ingame, away or offline, shown in words and by a mark; activity what they are doing (Ranked match, Last seen 2 d ago), empty hides it; canInvite the game will send an invite; invited one is pending. |
| `selected` | `string` | The friend the game marks as chosen (data-selected, aria-current); empty is none. |
| `onAccept?` | `(e: UiEvent, key?: string) => void` | The accept event, with the row's key where it comes from a row. |
| `onAdd?` | `(e: UiEvent, key?: string) => void` | The add event, with the row's key where it comes from a row. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onDecline?` | `(e: UiEvent, key?: string) => void` | The decline event, with the row's key where it comes from a row. |
| `onInvite?` | `(e: UiEvent, key?: string) => void` | The invite event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `empty`, `requests`, `overflow`.

## 74. Inbox

A mail inbox: the messages with their sender, date and unread state, the selected message's body and its attachments to claim, and Claim and Delete. Loading, a pending claim and an error each have their state. The game owns the messages and what a claim grants.

`import { Inbox } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/inbox.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Inbox. |
| `summary?` | `string` | The game's count or capacity wording, such as 2 unread · 8 / 50. |
| `messages?` | `ReadonlyArray<{ id: string; subject: string; sender: string; date: string; preview: string; body: string; unread: boolean; hasAttachments: boolean; claimable: boolean; claimed: boolean; expired: boolean; canDelete: boolean }>` | Keyed { id, subject, sender, date, preview, body, unread, hasAttachments, claimable, claimed, expired, canDelete } messages. Text is plain, not HTML. The game updates read/claim/delete facts after intents. Default []. |
| `attachments?` | `ReadonlyArray<{ id: string; message: string; name: string; icon: string; amount: string }>` | Keyed { id, message, name, icon, amount } rewards; message is the owning message id. Default []. |
| `selected?` | `string` | Selected message id, owned by the game; empty shows a choose-message prompt. |
| `loading?` | `boolean` | Loading messages; no message actions are available. Default false. |
| `busy?` | `boolean` | A claim/delete operation is pending; prevent duplicate mutations. Default false. |
| `error?` | `string` | Loading failed; show recovery advice and retry. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |
| `onDelete?` | `(e: UiEvent, key?: string) => void` | The delete event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `empty`, `loading`, `error`, `expired`.

Actions: `choose` (Read the message by stable key), `claim` (Claim the selected message's attachments by stable message key), `delete` (Request deleting the selected message by stable key), `retry` (Retry loading messages).

## 75. News

News and updates: articles with their date, kind, picture and summary, one expanded at a time with its body and action. Loading and an error each have their state; the game owns the articles.

`import { News } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/news.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses News and updates. |
| `entries?` | `ReadonlyArray<{ id: string; title: string; date: string; kind: string; summary: string; body: string; image: Texture \| 0; unread: boolean; pinned: boolean; actionLabel: string }>` | Keyed { id, title, date, kind, summary, body, image, unread, pinned, actionLabel } articles in the game's order. kind is game-formatted text (Patch notes, Event); body is plain multiline text, never HTML. image is a loaded texture handle or 0. Default []. |
| `selected?` | `string` | Expanded article id, game-owned. A toggle reports choose with that key; the game may clear it to collapse. |
| `loading?` | `boolean` | Loading articles; no article actions are present. Default false. |
| `disabled?` | `boolean` | Disable article toggles and actions. Default false. |
| `error?` | `string` | Failure explanation with retry; empty is no error. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onOpen?` | `(e: UiEvent, key?: string) => void` | The open event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `empty`, `loading`, `error`, `overflow`.

Actions: `choose` (Toggle an article by stable key), `open` (Take the selected article's optional action by stable key), `retry` (Retry loading articles).

## 76. PassTrack

A season pass: a kicker with the season and the time left over the title, a gold Get premium while premium is not owned, the level on a hexagon beside a leaning bar to the next tier, and two lanes of tier tiles, premium (in gold) over free. Each lane scrolls sideways on its own; on a narrow screen its name sits above it. A tile says its tier, shows the reward and how much, and ends in Claim when it is ready and owned, Claimed once taken, or Locked (a premium tile always reads Locked without premium). The game owns progress, entitlement and every reward's state; the track reports claim (a reward's key), upgrade and retry.

`import { PassTrack } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/pass-track.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Season pass. |
| `season?` | `string` | The game's season name, a kicker over the title. |
| `refresh?` | `string` | The game's already formatted time remaining, after the season in the kicker; no clock runs here. |
| `level?` | `string` | The player's pass level as the game words it, on a hexagon beside the progress; empty hides it. |
| `progress?` | `number` | Progress to the next tier, 0 to 1, supplied by the game. Default 0. |
| `progressText?` | `string` | Progress words, such as 240 / 500 XP; empty uses a percent. |
| `premiumUnlocked?` | `boolean` | The game owns the premium entitlement. Without it every premium reward reads Locked. Default false. |
| `upgradeLabel?` | `string` | Optional upgrade action while premium is not unlocked (Get premium); no purchase logic. |
| `rewards?` | `ReadonlyArray<{ id: string; tier: string; premium: boolean; name: string; icon: string; amount: string; detail: string; state: string }>` | Keyed rewards in the game's order: { id, tier, premium, name, icon, amount, detail, state }. state locked/available/claimed. Premium rewards fill the premium lane, the rest the free lane; each reward is one tile, and its tier and state are always written in words. Default []. |
| `loading?` | `boolean` | Loading the track; no reward action is available. Default false. |
| `error?` | `string` | A failure explanation with retry; empty is no error. |
| `disabled?` | `boolean` | Prevent claims and upgrades while the game processes an operation. Default false. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |
| `onUpgrade?` | `(e: UiEvent, key?: string) => void` | The upgrade event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `empty`, `loading`, `error`, `premium`.

Actions: `claim` (Claim the available reward by stable key), `upgrade` (Inspect the premium upgrade), `retry` (Retry loading the track).

## 77. Profile

A player profile: the avatar with level and presence, the name, handle, rank and bio, the stats the game formats and the actions it offers (friend, invite, report). Private stats, loading, an error and no profile each have their state.

`import { Profile } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/profile.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | Player display name; empty uses Player. |
| `handle` | `string` | Stable player handle or discriminator, as the game words it; empty hides it. |
| `portrait` | `Texture \| 0` | Loaded portrait texture; zero uses the base Avatar fallback. |
| `level` | `number` | Level shown by Avatar; zero hides it. |
| `status` | `string` | Avatar presence: online, away, busy or offline; empty hides it. |
| `rank` | `string` | Game-formatted rank or title; empty hides it. |
| `bio` | `string` | Player biography or activity; wraps, empty hides it. |
| `stats` | `ReadonlyArray<{ id: string; label: string; value: string; detail: string }>` | Keyed { id, label, value, detail } readouts. The game formats all values and decides their order; detail is an optional explanation. |
| `actions` | `ReadonlyArray<{ id: string; label: string; disabled: boolean; reason: string; pending: boolean }>` | Keyed { id, label, disabled, reason, pending } actions, e.g. friend, invite, edit or report. action reports the stable row id; pending disables it and reads Sending. The game owns permissions and effects. |
| `privateStats?` | `boolean` | Hide statistics and show Stats are private; no private values are mounted. |
| `loading?` | `boolean` | Loading profile data; hides all stale identity, statistics and actions behind the shared Spinner. |
| `error` | `string` | Error message; hides stale data and offers retry. Empty means no error. |
| `empty?` | `boolean` | No profile found; hides stale identity, statistics and actions. |
| `onAction?` | `(e: UiEvent, key?: string) => void` | The action event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `private`, `pending`, `loading`, `error`, `empty`, `no-stats`, `overflow`.

## 78. Reveal

A reward reveal: a kicker naming the source over the title; sealed, the prize large over a gold glow with the game's words under it and Open; revealing, a spinner and Skip; revealed, each reward in a socket of its rarity (its count in a corner, a dot when it is new) with the rarity in words over its name, and Continue. Rewards stay hidden until the game says they are revealed; a reward is a button that reports inspect with its key. The game owns the roll, the timing and the rewards; the panel reports open, skip, inspect, continue and retry.

`import { Reveal } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/reveal.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Rewards. |
| `kicker?` | `string` | A small line over the title naming where the rewards come from (Expedition rewards); empty hides it. |
| `body?` | `string` | Optional explanation under the sealed prize. |
| `icon?` | `string` | The sealed prize's icon or picture (a chest, a pouch); empty uses the bag. |
| `state?` | `string` | Game-owned sealed/loading/revealing/revealed. No roll, timer or reward mutation runs here. Default "sealed". |
| `rewards?` | `ReadonlyArray<{ id: string; name: string; icon: string; amount: string; rarity: string; fresh: boolean }>` | Keyed { id, name, icon, amount, rarity, fresh } rewards, shown only once revealed. rarity is common/uncommon/rare/epic/legendary/mythic, in words and as the socket's colour; amount (×3) sits in the socket's corner; fresh marks a new one with a dot. Default []. |
| `canSkip?` | `boolean` | Offer skip while revealing; the game decides what skipping does. Default true. |
| `continueLabel?` | `string` | Continue wording; empty uses Continue. |
| `disabled?` | `boolean` | Prevent opening, skipping, inspecting and continuing. Default false. |
| `error?` | `string` | A failed reveal with retry; rewards stay hidden until the game resolves it. |
| `onContinue?` | `(e: UiEvent, key?: string) => void` | The continue event, with the row's key where it comes from a row. |
| `onInspect?` | `(e: UiEvent, key?: string) => void` | The inspect event, with the row's key where it comes from a row. |
| `onOpen?` | `(e: UiEvent, key?: string) => void` | The open event, with the row's key where it comes from a row. |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Slot.frame`.

Fixtures: `default`, `loading`, `revealing`, `revealed`, `empty`, `error`.

Actions: `open` (Open the sealed rewards), `skip` (Finish the reveal early), `inspect` (Inspect a revealed reward by its key), `continue` (Continue after the reveal), `retry` (Retry the reveal).

## 79. ShopConfirm

The purchase confirmation the game shows after a shop's buy: a small heading over the offer's name, its art in a socket of its rarity beside its detail, and a ledger with one group per currency: the price (a discount's old price struck before it, and named: 700 Gems, was 1,000), then the balance before and after, or, for a currency the player lacks, Not enough and how much more it takes. The game decides whether it may be bought and what it costs; the panel reports confirm (Buy) and cancel (Cancel, or Escape and pad B anywhere in it). Enter and pad A press the focused button, Cancel first, so no key buys by accident. A refused purchase disables Buy and says why (the short currency's row, or the game's reason over the buttons); a failed one shows the game's error on a strip and leaves Buy to try again. While busy Buy is disabled. The ledger scrolls when it must; the buttons stay at the foot. It is content, not a dialog: the host that shows it (the built-in confirm screen, a sheet at the foot of a phone, or the game's own modal) owns modality and focus.

`import { ShopConfirm } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/shop-confirm.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading, a small line over the offer's name; empty takes the group's (Confirm purchase). |
| `name` | `string` | The offer's name. |
| `icon` | `string` | The offer's icon in the style's or the game's atlas. |
| `rarity?` | `string` | common, uncommon, rare, epic, legendary or mythic: the socket the icon sits in; empty is common. |
| `detail` | `string` | What the offer gives, as the game words it; empty hides it. |
| `prices` | `ReadonlyArray<{ id: string; icon: string; label: string; amount: string; was: string; after: string; short: boolean; missing: string }>` | What it costs, one group of rows per currency: keyed rows of { id, icon, label, amount, was, after, short, missing }: amount, was (struck, empty none) and after (the balance before and after, 3,200 → 2,500; empty hides it) as the game formats them; label the currency's name; short marks a currency the player lacks, and missing how much more it takes (1,600), shown in place of the balance. |
| `canBuy?` | `boolean` | The game allows the purchase; false disables Buy. |
| `reason?` | `string` | Why the purchase is refused when no price says it, as the game words it (Requires level 40); shown over the buttons while canBuy is false. |
| `busy?` | `boolean` | The purchase is in flight: Buy is disabled and says it is working. |
| `error` | `string` | Why the last attempt failed, as the game words it; empty is none. |
| `onCancel?` | `(e: UiEvent, key?: string) => void` | The cancel event, with the row's key where it comes from a row. |
| `onConfirm?` | `(e: UiEvent, key?: string) => void` | The confirm event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Slot.frame`.

Fixtures: `default`, `disabled`, `refused`, `loading`, `error`, `overflow`.

Actions: `back` (Cancel).

## 80. Shop

A storefront: a band with a kicker over the title and the player's balances as chips, category tabs with their counts between the bumper caps, and a grid of offer tiles. A tile is its art in a well over a glow of its rarity, corner tags (Featured, and a badge the game words), its name and detail, and one ruled foot with its price in one or more currencies (a discount's old price struck before it, and named: 700 Gems, was 1,000). The game owns the catalogue, every price and every verdict (sold out, owned, unaffordable); the shop reports buy (a tile's key, which the game answers with a ShopConfirm), choosetab (a category's key), tabNext and tabPrev (E/Q, bumpers) and retry. A tile the game refused has no buy handler: it keeps the focus so a player can read it, is marked disabled (aria-disabled), dims its art and says why in words on its foot. The selected tile wears corner brackets. The grid fills the height it is given and scrolls inside it; narrow, it is two columns and the tabs scroll sideways.

`import { Shop } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/shop.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the shop's own: it fills its parent by default; `grow-0 basis-auto` makes it hug its offers. |
| `title` | `string` | The shop's heading; empty takes the group's (Shop). |
| `kicker?` | `string` | A small line over the title naming the place (Store, a merchant's town); empty hides it. |
| `wallet` | `ReadonlyArray<{ id: string; icon: string; label: string; amount: string }>` | The player's balances shown in the band, keyed rows of { id, icon, label, amount }: amount as the game formats it; label names the currency for a screen reader. |
| `tabs` | `ReadonlyArray<{ id: string; label: string; count: number }>` | The categories, keyed rows of { id, label, count }: count, how many offers the category holds, is shown beside its label above 0; empty shows no tab row. |
| `tab` | `string` | The category shown. |
| `offers` | `ReadonlyArray<{ id: string; name: string; icon: string; image?: Texture \| 0; detail: string; rarity: string; badge: string; badgeTone: string; featured: boolean; soldOut: boolean; owned: boolean; affordable: boolean }>` | The offers in the shown category, in the game's order: keyed rows of { id, name, icon, image?, detail, rarity, badge, badgeTone, featured, soldOut, owned, affordable }: image the offer's picture filling its art well (a render of the item), 0 or absent shows icon; detail a second line (×500, 7 days); rarity common, uncommon, rare, epic, legendary or mythic, the light behind the art (empty is common); badge a corner tag the game words (−30%, New), empty none, on gold unless badgeTone is success; featured a wide tile; soldOut, owned and affordable the game's verdicts: a sold out, owned or unaffordable (affordable false) tile keeps the focus, is marked disabled, says why in words on its foot and reports nothing. |
| `prices` | `ReadonlyArray<{ id: string; offer: string; icon: string; label: string; amount: string; was: string; short: boolean }>` | Every offer's price, one row per currency: keyed rows of { id, offer, icon, label, amount, was, short }: offer an offer's id; amount and was (the struck price before a discount, empty none) as the game formats them; label the currency's name; short marks the currency the player lacks. |
| `selected` | `string` | The offer the focus rests on; empty is none. |
| `loading?` | `boolean` | The catalogue is on its way: a spinner stands in. |
| `error` | `string` | Why the catalogue could not load, as the game words it; empty is none. Shows a Try again button. |
| `onBuy?` | `(e: UiEvent, key?: string) => void` | The buy event, with the row's key where it comes from a row. |
| `onChoosetab?` | `(e: UiEvent, key?: string) => void` | The choosetab event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |
| `onTabNext?` | `(e: UiEvent, key?: string) => void` | The tabNext event, with the row's key where it comes from a row. |
| `onTabPrev?` | `(e: UiEvent, key?: string) => void` | The tabPrev event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `empty`, `loading`, `error`, `overflow`.

Actions: `tabPrev` (Previous category), `tabNext` (Next category).

## 81. RpgBackpack

A bag of items: a band with a kicker, the title and the purse, category tabs between the action's key caps, how full the bag is, a grid of item tiles and a footer with the carried weight. The game owns what a category holds, the order and every number; the bag reports choose (an item's key), choosetab (a category's key), tabNext and tabPrev. A tile drags onto another (a locked one stays put) and the bag reports move with { dragged, target }, the two items' keys. A tile shows its rarity as its bar along the foot and its tags; its stack, Equipped, New, Locked and Broken sit in its corners, each named in words for a screen reader, Broken also on the tile. The selected tile wears corner brackets. The grid fills the height it is given and scrolls inside it.

Dropped from the .svelte: the load bar's lean (skew is not a class the record draws), and the tile's light rising from its foot (a cut shape takes one bg colour).

`import { RpgBackpack } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/backpack.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The bag's heading (Backpack, Stash, a chest's name). |
| `kicker` | `string` | A small line over the title naming the screen (Inventory); empty hides it. |
| `tabs` | `ReadonlyArray<{ id: string; label: string; count: number }>` | The categories, a keyed collection of { id, label, count }: count, how many items the category holds, is shown beside its label above 0. |
| `tab` | `string` | The id of the category shown. |
| `items` | `ReadonlyArray<{ id: string; name: string; icon: string; count: number; rarity: string; equipped: boolean; fresh: boolean; locked: boolean; broken: boolean }>` | The items shown, in the game's order: each { id, name, icon, count, rarity, equipped, fresh, locked, broken }. rarity is common, uncommon, rare, epic, legendary or mythic; count above 1 is shown; fresh marks an unseen item, locked one the game will not let go, broken one it will not equip. |
| `selected` | `string` | The selected item's id; empty is none. |
| `capacity` | `string` | How full the bag is, as the game words it (30 / 36 slots); empty hides it. |
| `weight` | `string` | The load as the game words it (92.4 / 120); empty hides it. |
| `load` | `number` | The load from 0 to 1 for its bar; above 1 is overburdened. |
| `gold` | `string` | The purse as the game formats it; empty hides it. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onChoosetab?` | `(e: UiEvent, key?: string) => void` | The choosetab event, with the row's key where it comes from a row. |
| `onTabNext?` | `(e: UiEvent, key?: string) => void` | The tabNext event, with the row's key where it comes from a row. |
| `onTabPrev?` | `(e: UiEvent, key?: string) => void` | The tabPrev event, with the row's key where it comes from a row. |
| `onMove?` | `(e: UiEvent, move: { dragged: string; target: string }) => void` | A tile dropped on another: the dragged item's key and the target's. A locked tile is not picked up; the dragged tile goes home, and the game reorders, swaps or stacks as it sees fit. |

Pack slots: `Chip.frame`, `Panel.frame`, `Slot.frame`.

Fixtures: `default`, `empty`, `over`, `overflow`.

Actions: `tabPrev` (Previous category), `tabNext` (Next category).

## 82. Breadcrumbs

A trail in type: dim links, faint chevrons, the current page bold on a lit underline. The trail wraps between crumbs, and a long current page takes a line of its own and wraps between its words. Reports choose (a crumb's key).

Dropped from the .svelte: nothing (the separator's svg path is a `Line`).

`import { Breadcrumbs } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/breadcrumbs.tsx`).

| Prop | Type | What |
|---|---|---|
| `items` | `ReadonlyArray<{ id: string; label: string; icon: string; current: boolean }>` | The crumbs, the current page last: each { id, label, icon, current }, current true on the last. |
| `label` | `string` | What a screen reader calls it; empty is Breadcrumb. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `long`.

Actions: `choose` (Go back).

## 83. Carousel

Pages of art on the HUD plate: the current one ringed in the accent, its neighbours set back; the place is a row of leaning pips between cut arrow keys. Reports prev and next.

Dropped from the .svelte: the pages' scroll-snap (the strip scrolls without snapping to a page), the pips' lean (skew is not a class the record draws), and the plate's rolled corner rims (the cut shape carries no border here); the lit pip keeps its place by the index.

`import { Carousel } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/carousel.tsx`).

| Prop | Type | What |
|---|---|---|
| `pages` | `ReadonlyArray<{ id: string; title: string; body: string; image: Texture \| 0 }>` | The pages: each { id, title, body, image }, image a texture handle (0: none). |
| `index` | `number` | The current page, from 0 (its dot lit). |
| `label` | `string` | What a screen reader calls it. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onPrev?` | `(e: UiEvent, key?: string) => void` | The prev event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`.

Fixtures: `default`, `empty`, `first`.

Actions: `prev` (Previous), `next` (Next).

## 84. RpgConversationBacklog

The conversation's history: the title and Close, then the lines as cut cards — the speaker in gold small capitals, narration quiet and in italics, the line replaying washed gold with a gold bar — with Load earlier lines over them and Replay or Stop replay under a replayable one. Reports close, older, replay and stop (an entry's key). Answers the back action.

Dropped from the .svelte: the rows' lit gradient (a cut shape takes one bg colour).

`import { RpgConversationBacklog } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/conversation-backlog.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The history title; empty uses Conversation history. |
| `lines?` | `ReadonlyArray<{ id: string; speaker: string; text: string; canReplay: boolean }>` | Chronological keyed { id, speaker, text, canReplay } entries. Empty speaker is narration. The game supplies history and audio availability; lines keep their original wording. Default []. |
| `playing?` | `string` | The entry currently replaying; empty means no replay. The game owns playback and reports completion by clearing this id. |
| `hasOlder?` | `boolean` | Earlier history is available; older asks the game to prepend it, preserving row ids. Default false. |
| `loading?` | `boolean` | History is loading; earlier-line and replay actions are disabled, close remains available. Default false. |
| `onClose?` | `(e: UiEvent, key?: string) => void` | The close event, with the row's key where it comes from a row. |
| `onOlder?` | `(e: UiEvent, key?: string) => void` | The older event, with the row's key where it comes from a row. |
| `onReplay?` | `(e: UiEvent, key?: string) => void` | The replay event, with the row's key where it comes from a row. |
| `onStop?` | `(e: UiEvent, key?: string) => void` | The stop event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`, `Row.frame`.

Fixtures: `default`, `empty`, `loading`, `playing`.

Actions: `back` (Close history).

## 85. RpgConversation

A conversation across the foot of the screen: the speaker's portrait, name and line, and the player's numbered replies. Reports reply (a reply's key) and next (the Continue button, or confirm, while there are no replies). With a rate the line shows a few characters at a time by the world's tick (always whole to a screen reader) and reports revealed once it is whole; the game decides what a press before that does.

Dropped from the .svelte: the whole line at once under reduced motion (the TSX UI cannot read the setting yet), the name plate's two-tone gold gradient, the row entries' lit gradient (a cut shape takes one bg colour), and the auto key's inset gold keyline when on (its gold bar and text stay).

`import { RpgConversation } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/conversation.tsx`).

| Prop | Type | What |
|---|---|---|
| `speaker` | `string` | The speaker's name; empty is narration. |
| `portrait` | `Texture \| 0` | The speaker's portrait (og.load.texture); 0 hides it. |
| `line` | `string` | What is said now. |
| `rate?` | `number` | Characters of the line shown per second, by the world's tick, starting again with each new line; 0 (and when left out) shows the line at once, and setting 0 finishes a line early. A screen reader always reads the whole line. |
| `replies` | `ReadonlyArray<{ id: string; number: number; text: string; used: boolean }>` | The player's replies, keyed rows of { id, number, text, used }: number the figure before it; used greys a reply already chosen. Empty shows a Continue button. |
| `selected` | `string` | The reply the focus rests on; empty is none. |
| `showControls?` | `boolean` | Shows optional History, Skip and Auto buttons; hidden in existing compact layouts. Default false. |
| `canBacklog?` | `boolean` | The game permits opening conversation history. Default true. |
| `canSkip?` | `boolean` | The game permits requesting a skip; the component never advances dialogue itself. Default false. |
| `canAuto?` | `boolean` | The game permits requesting an automatic-advance setting change. Default true. |
| `auto?` | `boolean` | The game's automatic-advance setting; toggleAuto requests a change without advancing dialogue. Default false. |
| `onBacklog?` | `(e: UiEvent, key?: string) => void` | The backlog event, with the row's key where it comes from a row. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onReply?` | `(e: UiEvent, key?: string) => void` | The reply event, with the row's key where it comes from a row. |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |
| `onToggleAuto?` | `(e: UiEvent, key?: string) => void` | The toggleAuto event, with the row's key where it comes from a row. |
| `onRevealed?` | `(e: UiEvent) => void` | The revealed event: the line is whole, once per line that was revealed at a rate. |

Pack slots: `Button.plate`, `Chip.frame`, `Plate.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `line`, `controls`, `typing`, `auto`, `disabled`.

Actions: `confirm` (Continue).

## 86. RpgCrafting

A crafting station: a band with the station's name, the recipes as rows (the result's icon in a socket, its rarity on the row's edge and in its name, why it cannot be made in words), and beside them (under them when narrow) the selected recipe: its art large over a glow of its rarity, what it does and how many the player holds, the ingredients as have / need with a short one marked in words, the batch stepper with the cost, and Craft. The game decides every requirement; the panel reports choose, increase, decrease and craft, each with the recipe's key.

Dropped from the .svelte: the hero's radial glow of the rarity (the record draws linear gradients only; a linear wash of the rarity stands in).

`import { RpgCrafting } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/crafting.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The station name; empty uses Crafting. |
| `recipes?` | `ReadonlyArray<{ id: string; name: string; icon: string; description: string; canCraft: boolean; reason: string; rarity: string; held: string }>` | Keyed { id, name, icon, description, canCraft, reason, rarity, held } recipes. Uncraftable recipes remain selectable for inspection; reason explains the restriction. rarity (common … mythic, empty common) colours the row's edge, the socket and the name; held, how many of the result the player has as the game words it, is shown under the result (empty hides it). Default []. |
| `selected?` | `string` | The selected recipe id, or empty for selection guidance. Clear the selection when removing its recipe. |
| `ingredients?` | `ReadonlyArray<{ id: string; name: string; quantity: string; sufficient: boolean }>` | Requirements for the selected recipe and quantity: keyed { id, name, quantity, sufficient }. The game formats quantity (2 / 3) and computes sufficiency. Default []. |
| `quantity?` | `number` | The game-owned batch size. Increase/decrease intents carry the selected recipe id; the game validates and updates it. Default 1. |
| `canIncrease?` | `boolean` | The game permits a larger batch. Default false. |
| `canDecrease?` | `boolean` | The game permits a smaller batch. Default false. |
| `cost?` | `string` | The complete batch cost as the game words it; empty hides it. |
| `loading?` | `boolean` | Recipe data is loading; existing recipes remain visible but inactive. Default false. |
| `busy?` | `boolean` | A craft is in progress; selection, quantities and crafting are inactive. Default false. |
| `error?` | `string` | The game's failure message; empty hides it without discarding the selection. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onCraft?` | `(e: UiEvent, key?: string) => void` | The craft event, with the row's key where it comes from a row. |
| `onDecrease?` | `(e: UiEvent, key?: string) => void` | The decrease event, with the row's key where it comes from a row. |
| `onIncrease?` | `(e: UiEvent, key?: string) => void` | The increase event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`, `Row.frame`, `Row.frame.selected`, `Slot.frame`.

Fixtures: `default`, `empty`, `loading`, `busy`, `missing`.

## 87. DataGrid

The HUD's smoked-glass plate: headings in small tracked capitals over a faint rule, rows dense and plain so the data reads first, figures condensed and tabular. The chosen row is washed in the accent from its leading edge. In a narrow shell each row becomes a two-line card: the first column and the last on one line, the rest as a caption under. Reports choose (a row's key) and retry.

Dropped from the .svelte: the plate's rolled corner rims (the cut shape carries no border here), and the rows' hairline separators (each row is its own plate). The chosen row's wash is one flat colour.

`import { DataGrid } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/data-grid.tsx`).

| Prop | Type | What |
|---|---|---|
| `columns` | `ReadonlyArray<{ id: string; label: string; numeric: boolean }>` | The headings, a keyed collection of { id, label, numeric }: numeric columns align to the end in tabular figures. |
| `tracks` | `string` | The columns' widths as CSS grid tracks, one per column; empty shares the width equally between four. A numeric column's track holds its widest figure in text-scaled units (calc(96 * var(--font-step-px))), so it grows with the player's text size; a figure is never cut, and the text columns (fr) give way. |
| `rows` | `ReadonlyArray<{ id: string; cells: ReadonlyArray<{ id: string; text: string; numeric: boolean }> }>` | The rows, a keyed collection of { id, cells }: cells a keyed list of { id, text, numeric }, in the columns' order. Only the rows in view are built. |
| `selected` | `string` | The id of the selected row; empty is none. |
| `label` | `string` | What a screen reader calls the grid. |
| `loading?` | `boolean` | Rows are on their way: busy placeholder rows under the header. |
| `error` | `string` | An error to show with Retry instead of the rows; empty is none. |
| `empty` | `string` | What an empty grid says; empty is Nothing here yet. |
| `hint` | `string` | A line under the empty message. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `loading`, `error`, `empty`, `selected`.

Actions: `choose` (Select), `retry` (Retry).

## 88. RpgItemCard

One item in detail: its art large over a glow of its rarity, the rarity and kind as a kicker in the rarity's ink over the name, the headline figure large with its change against what is equipped, the other figures as rows (this, change, now), the lore under a rule of the rarity, and up to two buttons, with the reason the main one is refused in words. The game words every figure and decides the trend; the card reports equip (the main button, X or pad Y) and discard (the second button). It fills the height it is given: the figures and the lore scroll when needed; the buttons stay at its foot; the art and the name scroll with the figures. Narrow, the art sits in a socket beside the name and the rows drop the equipped column.

Dropped against the .svelte: the art's drop-shadow filter and the hero glow's radial shape (a linear wash stands in; filters and radial gradients have no surface here).

`import { RpgItemCard } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/item-card.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The item's name. |
| `icon` | `string` | Its icon in the style's or the game's atlas, drawn large at the top. |
| `art` | `Texture \| 0` | A painted picture of the item (og.load.texture) shown large in place of its icon; 0 shows the icon. |
| `rarity` | `string` | common, uncommon, rare, epic, legendary or mythic; the name and the art take its ink. |
| `kind` | `string` | The rarity and kind line as the game words it (Legendary · Longsword), a kicker over the name in the rarity's ink. |
| `line?` | `string` | A line under the name as the game words it (One-handed · Item level 60); empty hides it. |
| `lead` | `ReadonlyArray<{ id: string; icon: string; label: string; value: string; delta: string; trend: string; current: string }>` | The headline figure, zero or one row of { id, icon, label, value, delta, trend, current }: trend is up, down or none; delta and current are text, empty hides them. |
| `stats` | `ReadonlyArray<{ id: string; icon: string; label: string; value: string; delta: string; trend: string; current: string }>` | The other figures, keyed rows of { id, icon, label, value, delta, trend, current }. |
| `lore` | `string` | The flavour text; empty hides it. |
| `primary` | `string` | The main button's text (Equip, Use); empty hides it. |
| `secondary` | `string` | The second button's text (Drop, Sell); empty hides it. |
| `canPrimary?` | `boolean` | The main button is enabled. |
| `canSecondary?` | `boolean` | The second button is enabled. |
| `reason?` | `string` | Why the main button is refused, as the game words it (Requires level 40); shown in red over the buttons while canPrimary is false. |
| `onDiscard?` | `(e: UiEvent, key?: string) => void` | The discard event, with the row's key where it comes from a row. |
| `onEquip?` | `(e: UiEvent, key?: string) => void` | The equip event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`.

Fixtures: `default`, `plain`, `refused`.

Actions: `details` (Equip).

## 89. RpgLoot

What a chest, a body or a reward holds: a band with the source's name and how many things it holds, one row per item (its icon in a socket and its rarity on the row's edge, the name in that colour over what it is and, when the game refuses it, why in words; the count on the name's line and Take), and Close beside Take all. The game decides what can be taken; the panel reports take (an item's key), takeAll and close (Close, or Escape and pad B anywhere in it). The rows scroll inside the height given.

Dropped against the .svelte: the socket's rarity rim (the rarity tints the socket's ground instead; a box shadow cannot follow the cut shape), the art's drop-shadow filter, and the focused row's wash gradient and inset rim (a flat wash and the leading edge stand in; a cut element takes one bg colour and no box shadow).

`import { RpgLoot } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/loot.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The source's name; empty uses Loot. |
| `items?` | `ReadonlyArray<{ id: string; name: string; icon: string; count: number; description: string; available: boolean; reason: string; rarity: string }>` | Keyed { id, name, icon, count, description, available, reason, rarity }. rarity is common, uncommon, rare, epic, legendary or mythic (empty is common): the row's edge, the icon's socket and the name take its colour. The game supplies legality/capacity and removes collected rows after take or takeAll. Default []. |
| `canTakeAll?` | `boolean` | The game permits taking the displayed items together. Empty/loading/busy never sends takeAll. Default false. |
| `loading?` | `boolean` | Rewards are loading; existing items remain visible but cannot be collected. Default false. |
| `busy?` | `boolean` | A collection request is pending; disables take and takeAll to prevent duplicate requests. Default false. |
| `error?` | `string` | The game's collection failure, such as a full inventory; empty hides it. |
| `onClose?` | `(e: UiEvent, key?: string) => void` | The close event, with the row's key where it comes from a row. |
| `onTake?` | `(e: UiEvent, key?: string) => void` | The take event, with the row's key where it comes from a row. |
| `onTakeAll?` | `(e: UiEvent, key?: string) => void` | The takeAll event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Row.frame`.

Fixtures: `default`, `empty`, `loading`, `busy`, `error`.

Actions: `back` (Close).

## 90. Menu

A menu: a dense smoked plate with a lit top edge, its rows dim type that wash in the accent from a lit leading edge under the pointer, the keys or the focus, a shortcut on a dark key chip, a check drawn in the accent, the chosen radio a lit diamond, a destructive command in the danger ink and a disabled row faint. A submenu's rows show in place of the menu's under a back row, and a header or a separator parts the entries. Reports choose (an entry's key) and back.

Dropped against the .svelte: the CJK heading size (a lang has no class variant here; the engine's text scale carries it).

`import { Menu } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/menu.tsx`).

| Prop | Type | What |
|---|---|---|
| `entries` | `ReadonlyArray<{ id: string; label: string; kind: string; icon: string; shortcut: string; checked: boolean; danger: boolean; disabled: boolean; parent: string; more: boolean }>` | Its entries, submenus' too, a keyed collection of { id, label, kind, icon, shortcut, checked, danger, disabled, parent, more }: kind item, check, radio, separator or header; icon an icon name before the label (empty: none); shortcut shown at the end (Ctrl+S); checked a check or radio entry is on; danger a destructive command in the danger ink; disabled shown but inert; parent the id of the entry whose submenu it is in (empty: the top level); more it opens a submenu (a chevron at its end). |
| `submenu` | `string` | The id of the entry whose submenu shows, its rows in place of the menu's under a back row (set it to an entry with more when choose names that entry; back clears it); empty: the top level. |
| `label` | `string` | What a screen reader calls it; empty is Menu. |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Row.frame`.

Fixtures: `default`.

Actions: `choose` (Choose), `back` (Back).

## 91. RpgQuestLog

The quest list, grouped as the game groups it (main, side, finished): each group a gold label in small capitals behind a short gold rule (a finished group's quiet), each quest a row with its mark. The mark is a diamond: hollow, lit gold on the quest the HUD tracks, filled in the success colour once the quest is done. The selected row (the one in detail) is washed gold from its leading edge with a gold bar on it and a chevron toward the detail; a fresh update carries a New tag. Finished quests are struck and quiet. Reports choose (a quest's key) and track (X or pad Y on the focused quest, its key). It fills its height and scrolls inside it. Dropped against the .svelte: the chosen quest's name scrolling in place when it does not fit (it ends in an ellipsis like the others), and the compact touch/narrow row height tweak (one row height everywhere).

`import { RpgQuestLog } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/quest-log.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The log's heading (Quest Journal). |
| `groups` | `ReadonlyArray<{ id: string; label: string; done: boolean }>` | The groups in order, keyed rows of { id, label, done }: done marks the group of finished quests, shown struck and quiet. |
| `quests` | `ReadonlyArray<{ id: string; group: string; name: string; tracked: boolean; fresh: boolean }>` | The quests, keyed rows of { id, group, name, tracked, fresh }: group a group's id; tracked the one the HUD follows; fresh an update the player has not read. |
| `selected` | `string` | The quest shown in detail; empty is none. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onTrack?` | `(e: UiEvent, key?: string) => void` | The track event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `empty`, `overflow`.

Actions: `details` (Track).

## 92. SplitView

Two panes on the HUD plate, the divider between them a faint groove with its grip lit in the accent.

Dropped against the .svelte: nothing.

`import { SplitView } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/split-view.tsx`).

| Prop | Type | What |
|---|---|---|
| `ratio?` | `number` | The start pane's share of the room, 0 to 1; the game keeps it from the divider's drag events. Default 0.5. |
| `column?` | `boolean` | One pane over the other; false is side by side. |
| `label` | `string` | What a screen reader calls the divider; empty is Resize. |
| `end?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the end snippet). |
| `start?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the start snippet). |
| `onResize?` | `(e: UiEvent, key?: string) => void` | The resize event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`.

Actions: `resize` (Resize).

## 93. TooltipCard

An item's tooltip card: a dense smoked plate lit along its top in its rarity's colour, its icon in a socket of that colour, the title and the one-to-six rarity pips, its stats (label, figure, a signed delta in the better green or the worse red with its triangle, an added or removed stat a chip), the body, a footer line and the action hints with their key caps; what is equipped stands beside it on a plainer plate, and under it on a narrow screen. A long name or stat wraps between words. Read only.

Dropped against the .svelte: the socket's octagon (no clip-path here; a square well stands in) and the CJK heading sizes (a lang has no class variant here).

`import { TooltipCard } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/tooltip-card.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The title, in its rarity's colour. |
| `subtitle` | `string` | A line under it (Legendary sword, Passive); empty: none. |
| `rarity` | `string` | none, common, uncommon, rare, epic, legendary or mythic: the title's and frame's colour and its pips (one to six, so no tier is told by colour alone). |
| `icon` | `string` | An icon beside the title when there is no image; empty: none. |
| `image` | `Texture \| 0` | An art slot beside the title (an item or ability texture handle, 1:1); 0: the icon. |
| `body` | `string` | The body text; empty: none. |
| `stats` | `ReadonlyArray<{ id: string; label: string; value: string; delta?: number; deltaText?: string; better?: boolean; change?: string }>` | Keyed { id, label, value, delta, deltaText, better, change? } comparison facts supplied by the game. Signed delta and its arrow describe the numeric direction; better true means lower is better, flipping the colours. Optional change is added or removed; absent means both items have the stat. Added shows value without delta; removed shows the compared value muted. Presence is translated and exposed as data-change, never inferred from strings. |
| `compareTitle` | `string` | The heading over a second card of what is equipped (Equipped); empty draws no second card. |
| `compareStats` | `ReadonlyArray<{ id: string; label: string; value: string }>` | The equipped item's stat rows, { id, label, value }. |
| `footer` | `string` | A last line (Sell for 120, a flavour quote) in the muted ink; empty: none. |
| `hints` | `ReadonlyArray<{ id: string; text: string; action: string }>` | Action hints (Right-click to equip), a keyed collection of { id, text, action }: action an input action whose glyph leads the hint; empty: none. |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `comparison`.

## 94. RpgTrade

A merchant: a band with the merchant's name, the Buy / Sell switch and the purse; an alert when a deal failed; the goods as rows (the item's icon in a socket, its rarity on the row's edge and in its name, its stock or why it cannot be had, and its price, a discount struck before it); and at the foot the selected good with its total, the balance after, and Buy or Sell. The game decides every price and verdict; the panel reports choose, switchMode, buy and sell, each with the good's key.

Dropped against the .svelte: the purse coin's radial sheen (a flat gold disc stands in; radial gradients have no surface here), the socket's rarity rim (the rarity tints the socket's ground instead), and the chosen row's wash gradient and inset rim (a flat wash and the leading edge stand in; a cut element takes one bg colour and no box shadow).

`import { RpgTrade } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/trade.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The merchant or trade title; empty uses Trade. |
| `mode?` | `string` | buy or sell. The game changes this in response to switchMode and supplies that side's goods and prices. Default "buy". |
| `wallet?` | `string` | The player's balance as the game words it; empty hides it. |
| `goods?` | `ReadonlyArray<{ id: string; name: string; icon: string; stock: string; description: string; price: string; was: string; after: string; available: boolean; reason: string; rarity: string }>` | Keyed { id, name, icon, stock, description, price, was, after, available, reason, rarity }. stock, price (the total), was (the struck price before a discount, empty none) and after (the balance once paid, empty hides it) are game-formatted strings; available includes balance, capacity, quantity and legality checks; rarity (common … mythic, empty common) colours the row's edge, the socket and the name. Default []. |
| `selected?` | `string` | The inspected good's id. Unavailable goods remain selectable so their restriction can be read. |
| `loading?` | `boolean` | Goods are loading; selection and transactions are disabled. Default false. |
| `busy?` | `boolean` | A transaction is pending; selection, mode switching and transactions are disabled. Default false. |
| `error?` | `string` | The game's transaction failure, kept visible alongside the selection. |
| `onBuy?` | `(e: UiEvent, key?: string) => void` | The buy event, with the row's key where it comes from a row. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onSell?` | `(e: UiEvent, key?: string) => void` | The sell event, with the row's key where it comes from a row. |
| `onSwitchMode?` | `(e: UiEvent, key?: string) => void` | The switchMode event, with the row's key where it comes from a row. |

Pack slots: `Button.plate.primary`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `empty`, `loading`, `sell`, `unavailable`.

## 95. PuzzleBoosters

The booster tray: each booster a round socket with its art, how many the player holds on a tag at its foot and its name under it. The one the game has armed is lifted, ringed and tagged Armed; one the game will not take is disabled, its art dimmed, tagged in words (Unavailable), and one the player has none of shows a gold + where its count was. Reports choose (a booster's key; tap, click, Enter or pad A); the game decides whether it applies, spends it and arms or fires it. Many boosters scroll across the tray; a narrow tray wraps them onto more rows.

`import { PuzzleBoosters } from "@origingame/engine/ui/components"` (kit `puzzle`, `packages/engine/src/ui/components/puzzle/boosters.tsx`).

| Prop | Type | What |
|---|---|---|
| `boosters?` | `ReadonlyArray<{ id: string; icon: string; name: string; count: number; available: boolean }>` | The boosters in tray order, keyed rows of { id, icon, name, count, available }: count how many the player holds, as the game counts them; available false when the game will not take it now (none left, not allowed on this level, a move in progress): the booster is disabled, sends nothing and says so. |
| `armed` | `string` | The booster the game has armed (waiting for its target on the board); empty is none. Shown pressed and worded Armed. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `selected`, `disabled`, `empty`, `overflow`.

Actions: `choose` (Use).

## 96. PuzzleGoals

The in-level HUD: what limits the level (moves or time left) in a cut well, amber with a tag in words when the game says it runs low; the goals as sockets with what is left of each, its name under it and a check once met; and the score over a leaning bar with the game's star marks, lit up to its progress. It draws what the game counts; it decides nothing. Where it is given less than a phone's width (its own, not the screen's) the moves and the score share the top line and the goals wrap under them.

`import { PuzzleGoals } from "@origingame/engine/ui/components"` (kit `puzzle`, `packages/engine/src/ui/components/puzzle/goals.tsx`).

| Prop | Type | What |
|---|---|---|
| `goals?` | `ReadonlyArray<{ id: string; icon: string; label: string; left: number; done: boolean }>` | The level's goals in order, keyed rows of { id, icon, label, left, done }: left how many are still to clear, as the game counts them; done the goal is met (a check replaces the count). |
| `counter` | `string` | What limits the level: moves (a move count) or time (a countdown). |
| `moves` | `number` | Moves left, when counter is moves. |
| `seconds` | `number` | Seconds left, when counter is time. |
| `low?` | `boolean` | The game says the moves or time are running low: the counter warns, in words as well as its dress. |
| `score` | `number` | The score so far, as the game counts it. |
| `progress?` | `number` | The score toward the level's top star, 0 to 1, for the bar under it; negative hides the bar. Default -1. |
| `stars?` | `ReadonlyArray<{ id: string; at: number }>` | The star marks on that bar, keyed rows of { id, at }: at where the star sits along the bar, 0 to 1; a star at or under progress is lit. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `time`, `low`, `lowTime`, `done`, `empty`, `overflow`.

## 97. PuzzleLevelPath

A scrolling level path over the game's own map art, which the component leaves at full brightness: chapters in order, each a glass header with its name, its stars and a gold line of them, then its levels as discs on a road, each at the place across the path the game gives (x). The road is lit gold up to the level to play and pale past it. A disc shows its number, a lock when it is locked, its stars on a dark capsule at its foot once done, and the Play plate under the current one, the road leaving only below it after a gap; the chosen one is ringed and says Selected, an open one says Open. Reports choose (a level's key; a locked level is disabled and sends nothing) and, with X or pad Y on the focused disc, details. The game decides what is open and counts the stars. The path opens scrolled to the level to play, each chapter's header held at the top while its levels scroll under it.

`import { PuzzleLevelPath } from "@origingame/engine/ui/components"` (kit `puzzle`, `packages/engine/src/ui/components/puzzle/level-path.tsx`).

| Prop | Type | What |
|---|---|---|
| `chapters?` | `ReadonlyArray<{ id: string; label: string; earned: number; total: number }>` | The chapters in path order, keyed rows of { id, label, earned, total }: earned and total the chapter's stars as the game counts them. |
| `levels?` | `ReadonlyArray<{ id: string; chapter: string; number: number; state: string; stars: number; maxStars: number; x: number }>` | The levels in path order, keyed rows of { id, chapter, number, state, stars, maxStars, x }: chapter a chapter's id; state locked, open, current or done; stars earned (0 to 3) of maxStars (1 to 3); x the node's place across the path, 0 (start) to 1 (end), as the game lays the path out. |
| `selected` | `string` | The level the player has picked (its details shown elsewhere); empty is none. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onDetails?` | `(e: UiEvent, key?: string) => void` | The details event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `empty`, `selected`, `locked`, `overflow`.

Actions: `details` (Play).

## 98. PuzzleSummary

The end of a level: won or failed, the level's name, the stars on the shared StarBar with the stars the game awarded in words, a new-best badge, the score breakdown the game words, and the next steps. Reports next (only when canNext), retry and menu (also Escape or pad B). The game computes the score, the stars and what comes next.

`import { PuzzleSummary } from "@origingame/engine/ui/components"` (kit `puzzle`, `packages/engine/src/ui/components/puzzle/summary.tsx`).

| Prop | Type | What |
|---|---|---|
| `result` | `string` | How the level ended: win or lose. |
| `level` | `string` | The level's name as the game words it (Level 12); empty hides it. |
| `score` | `number` | The final score, as the game counts it. |
| `scoreMax?` | `number` | The score of the star bar's end (its last threshold). Default 1. |
| `thresholds?` | `ReadonlyArray<{ id: string; at: number }>` | The star bar's thresholds, keyed rows of { id, at }: at a fraction of scoreMax (0 to 1), as the game's star rules place them. |
| `stars` | `number` | Stars the game awarded (0 to maxStars): spoken and written beside the bar. |
| `maxStars?` | `number` | Stars the level can award. Default 3. |
| `record?` | `boolean` | The game says the score is a new best: a badge with its word. |
| `rows?` | `ReadonlyArray<{ id: string; label: string; value: string }>` | The score breakdown, keyed rows of { id, label, value }: value already worded by the game (+2,400, ×3). |
| `total` | `string` | The breakdown's total as the game words it; empty hides the total row. |
| `canNext?` | `boolean` | Next is offered (a won level with a next one). |
| `onMenu?` | `(e: UiEvent, key?: string) => void` | The menu event, with the row's key where it comes from a row. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `full`, `lose`, `empty`, `overflow`.

Actions: `back` (Map).

## 99. RacingBanner

The race's calls at the top centre: a plate of smoked glass with a lit top edge in the HUD's accent, the call in widely set capitals (the count, GO, the final lap, the finish with the player's place, or the game's own words) and a line under it. The game steps the call and its id (at); a new id plays the entrance again. Announced as a status.

`import { RacingBanner } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/banner.tsx`).

| Prop | Type | What |
|---|---|---|
| `kind?` | `string` | What the banner calls: countdown (the count), go, finalLap, finish, call (the game's own words, in text), or none (nothing shown, as for any other word). Default "none". |
| `figure` | `number` | The countdown's figure (3, 2, 1); the game steps it. |
| `text` | `string` | The call's words as the game words them for kind call (Final lap next, Sector best); empty shows nothing. |
| `at` | `number` | The call's id: a new value replays the entrance (each count, the go, the flag). |
| `place` | `number` | On the finish banner, the player's place as an ordinal (1st); 0 hides it. |
| `detail` | `string` | A line under the call as the game words it (New lap record, Lap 3 of 3); empty hides it. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `countdown`, `go`, `finalLap`, `finish`, `overflow`.

## 100. RacingBonus

Feedback above the road at the top centre: a meter (the slipstream) as a word over a bar in the HUD's accent, and the points a drift or a chain earns, large and slanted, with what earns them in gold under it. The game decides the meter and every figure; read only.

`import { RacingBonus } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/bonus.tsx`).

| Prop | Type | What |
|---|---|---|
| `meter` | `string` | A meter's name as the game words it (Slipstream, Drafting), over a bar in the HUD's accent; empty hides the meter. |
| `value` | `number` | The meter's share, 0 to 1. |
| `points` | `string` | The points being earned now as the game words them (+2,450), large and slanted; empty hides them. |
| `detail` | `string` | What earns them as the game words it (Drift · Near miss ×3), in gold under the points; empty hides it. |
| `at` | `number` | The chain's id: a new value replays the points' entrance. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `meter`, `points`, `overflow`.

## 101. RacingBoost

A boost meter (nitro, ERS) beside the tachometer: its picture in a cut tile, the name with the key that fires it, and a leaning bar cut into cells, glowing while it fires. The game owns the store and when it fires; read only.

`import { RacingBoost } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/boost.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The meter's name as the game words it (Nitro, ERS, Boost); empty takes the kit's (Boost). |
| `icon` | `Texture \| 0` | The meter's picture (og.load.texture) in a cut tile before it; 0 hides it. |
| `key` | `string` | The input that fires it as the game names it (Shift), on a key cap after the label; empty for none; hidden under a touch screen. |
| `value` | `number` | What is left now. |
| `max?` | `number` | The most there can be. Default 1. |
| `cells?` | `number` | The cells the bar is cut into (5); 1 draws one bar. Default 5. |
| `firing` | `boolean` | It is firing now: the fill glows (data-firing). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `firing`, `empty`, `overflow`.

## 102. RacingCar

The car's condition at the bottom left: the tyres as tags two to a line, each worded and inked by its wear, the fuel as a thin bar with the game's words after it (danger and Fuel low when it runs low), and Box this lap when the team calls the car in. Read only.

`import { RacingCar } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/car.tsx`).

| Prop | Type | What |
|---|---|---|
| `tyres?` | `ReadonlyArray<{ id: string; label: string; text: string; state: string }>` | The tyres in the game's order (front left, front right, rear left, rear right), keyed rows of { id, label, text, state }: label and text as the game words them (FL, 92); state good, worn or bad, a green, gold or red ink on the tag. Two to a line. Empty hides the tyres. |
| `fuel?` | `number` | The fuel left, 0 to 1; negative hides the fuel. Default -1. |
| `fuelText` | `string` | The fuel as the game words it (18 L · 4 laps), after the bar; empty for none. |
| `fuelLow` | `boolean` | The fuel is low: the bar turns to danger with a Fuel low word (data-low). |
| `pit` | `boolean` | The team calls the car in: a Box this lap tag stands under the readout (data-pit). |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `fuel`, `fuelLow`, `overflow`.

## 103. RacingGarage

Choosing a car: the shown car large (its picture, name and class) with arrows to step the cars, a strip of tiles to jump to one, the shown car's ratings as bars, and one action for it: Drive when owned, Buy with its price when it can be had, a disabled Locked with the reason when it cannot, Driving when it is already the player's car. The game owns prices, unlocking and every rating; the garage reports the car's key and Back.

`import { RacingGarage } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/garage.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The page's heading; empty takes the kit's (Garage). |
| `vehicles?` | `ReadonlyArray<{ id: string; title: string; body: string; image: Texture \| 0; owned: boolean; locked: boolean; affordable: boolean; price: string }>` | The cars in the game's order, keyed rows of { id, title, body, image, owned, locked, affordable, price }: title the car's name, body its class line, image a texture (og.load.texture, 0 none); owned the player has it; locked it cannot be had yet; affordable the game allows buying it now (false disables Buy and names it Too costly); price what buying costs or why it is locked, as the game words it (12,500 cr, Reach level 12). |
| `shown` | `string` | The car in view (its tile selected, its stats shown, the action for it); empty is none. |
| `index` | `number` | The shown car's place in vehicles, from 0, read out as n of N beside the arrows (the game keeps it with shown). |
| `chosen` | `string` | The car the player drives now (Driving); empty is none. |
| `stats?` | `ReadonlyArray<{ id: string; label: string; value: number; max: number; text: string }>` | The shown car's ratings, keyed rows of { id, label, value, max, text }: label as the game words it (Speed); value over max fills the bar; text the figure shown beside it (8.4). |
| `onTabPrev?` | `(e: UiEvent) => void` | Previous car. |
| `onTabNext?` | `(e: UiEvent) => void` | Next car. |
| `onBack?` | `(e: UiEvent) => void` | Back. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a car from its tile. |
| `onSelect?` | `(e: UiEvent, id: string) => void` | Drive the shown car. |
| `onBuy?` | `(e: UiEvent, id: string) => void` | Buy the shown car. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `selected`, `buy`, `short`, `locked`, `empty`, `overflow`.

## 104. RacingLapHud

The race's place and clock at the top left: the place large and slanted over the field's size, the lap on a lit plate and the places gained beside it; under them the lap time running, the last and best laps and the split delta on a green or red tag. Every figure and time comes from the game, formatted; read only.

`import { RacingLapHud } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/lap-hud.tsx`).

| Prop | Type | What |
|---|---|---|
| `lap` | `number` | The lap the player is on, from 1; 0 before the start shows a dash. |
| `laps` | `number` | Laps in the race; 0 hides the lap's total (an open session). |
| `position` | `number` | The player's place, from 1; 0 shows a dash. |
| `racers` | `number` | Cars in the race; 0 hides the place. |
| `gained` | `number` | Places gained since the start as the game counts them: positive reads Up n places, negative Down n places (on data-trend), 0 hides it. |
| `current` | `string` | The lap time running now, as the game formats it (1:24.618); empty shows the blank time. |
| `best` | `string` | The player's best lap, formatted, in the best lap's violet; empty shows the blank time. |
| `last` | `string` | The last lap, formatted; empty shows the blank time. |
| `sector` | `string` | The split's name as the game words it (Sector 2); shown beside the delta. |
| `split` | `string` | The split delta as the game words it (−0.412, +1.204), on a green tag ahead and a red one behind; empty hides the row. |
| `splitState?` | `string` | ahead, behind or even: the delta's sense, on data-state and read with it. Default "even". |
| `freshBest` | `boolean` | The last lap was a new best: a Best lap tag stands on the last row. |
| `finalLap` | `boolean` | The player is on the final lap: the lap plate reads Final lap (data-final). |
| `timeLeft?` | `number` | Seconds left to the next checkpoint or the session's end (HudTimer); negative hides it. Default -1. |
| `timeTotal` | `number` | The checkpoint's whole time, for the timer's ring; 0 draws none. |
| `warnAt?` | `number` | Seconds left when the timer turns warn (HudTimer's warnAt); 10 when left out. Default 10. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `leading`, `last`, `grid`, `checkpoint`, `overflow`.

## 105. RacingPedals

The pedals at the bottom left: throttle and brake as two leaning upright wells filled from the foot, green and red, each named under it. Read only.

`import { RacingPedals } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/pedals.tsx`).

| Prop | Type | What |
|---|---|---|
| `throttle` | `number` | The throttle's travel now, 0 to 1. |
| `brake` | `number` | The brake's travel now, 0 to 1. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `braking`, `empty`, `overflow`.

## 106. RacingResults

The classification after the flag: place, driver, time or gap, points and the fastest-lap mark per row, the player's row marked You, cars that did not finish or were disqualified named so. The game computes every time and point. Continue and Retry (also the details action, X or pad Y) report once; a Retry the game refuses is disabled.

`import { RacingResults } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/results.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The page's heading (the event's name); empty takes the kit's (Results). |
| `rows?` | `ReadonlyArray<{ id: string; position: number; name: string; time: string; points: string; fastest: boolean; player: boolean; state: string }>` | The finishing order as the game classifies it, keyed rows of { id, position, name, time, points, fastest, player, state }: time the race time or gap as the game words it; points as text (empty hides it); fastest marks the fastest lap; player the local player's row; state finished, dnf or dsq (dnf and dsq are named in words and show no place or time). |
| `bestLap` | `string` | The race's fastest lap time, formatted; empty hides the line under the table. |
| `bestBy` | `string` | Who set the fastest lap; empty leaves the name out of the line. |
| `canRetry` | `boolean` | Retry is offered (false: disabled). |
| `hasPlayer` | `boolean` | A row is the player's: the hero beside the tower shows its place, time and points; false hides it. |
| `onRetry?` | `(e: UiEvent) => void` | Retry (also the details action, X or pad Y). |
| `onContinue?` | `(e: UiEvent) => void` | Continue. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `empty`, `disabled`, `overflow`.

## 107. RacingStandings

The position tower at the left edge: one leaning glass row per car in the order the game ranks them, the place in a box, the car's colour on a stripe, the name and the gap (Leader for the leader). The player's row is lit in the HUD's accent; pitting, retired and finished cars carry a word on a tag. On a phone it collapses to the leader and the rows either side of the player. Read only.

`import { RacingStandings } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/standings.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The tower's name for a reader; empty takes the kit's (Standings). Not drawn. |
| `rows?` | `ReadonlyArray<{ id: string; position: number; name: string; gap: string; color: string; player: boolean; state: string }>` | The running order as the game ranks it, keyed rows of { id, position, name, gap, color, player, state }: position the place shown in its box; gap the interval as the game words it (+1.204, +1 lap; the leader's empty gap reads Leader); color the car's or team's colour as a CSS colour, on a stripe beside the name (empty: a neutral stripe); player marks the local player's row (lit in the HUD's accent, aria-current); state racing, pitting, retired or finished, each a word on a tag (Pit, DNF, Fin), a retired row dimmed. |
| `room` | `string` | How short its place is: empty shows every row; short keeps the leader and the rows beside the player; shorter the leader and the player. |
| `hasPlayer` | `boolean` | A row is the player's. On a phone the tower keeps that row and the rows beside it, or the first three when false. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `leading`, `last`, `empty`, `single`, `finished`, `overflow`.

## 108. RacingTelemetry

The tachometer at the bottom right: the revs as a white arc round a dark glass face with the red zone at its end, the dial's figures inside the arc, and on the face the shift lights, the speed in the game's units and the gear on a lit tag with the gearbox's mode. The game decides every figure, the lights and the shift cue; read only.

`import { RacingTelemetry } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/telemetry.tsx`).

| Prop | Type | What |
|---|---|---|
| `speed` | `number` | The speed as the game measures it, already in its units (a whole number reads best). |
| `units` | `string` | The speed's unit as the game words it (km/h, mph); shown under the figure and read with it. |
| `gear` | `string` | The gear as the game names it: R, N, 1 to 8; on a tag in the HUD's accent; empty hides it. |
| `mode` | `string` | The gearbox's mode as the game words it (Auto, Manual), beside the gear; empty hides it. |
| `rpm` | `number` | Engine speed now: the rev arc's length. |
| `rpmMax?` | `number` | The top of the rev range, the arc's end. Default 1. |
| `redline` | `number` | The engine speed where the red zone starts, marked on the arc (lit, with data-redline on the frame, once the revs reach it); 0 is none. |
| `scale?` | `ReadonlyArray<{ id: string; value: number; label: string }>` | The dial's figures, keyed rows of { id, value, label }: value an engine speed on the arc, label its figure as the game words it (2 for 2,000 rpm); a figure at or past the redline is red. A tick stands by each. Empty draws none. A phone shows the ticks alone. |
| `lights?` | `number` | The share of the shift lights lit, 0 to 1, as the game decides it (seven cells, green to amber to red); negative hides the lights. Default -1. |
| `shift` | `boolean` | The game's shift cue: the Shift word stands beside the gear in place of the mode, and the frame takes data-shift. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `redline`, `pit`, `reverse`, `idle`, `overflow`.

## 109. RacingTrackMap

The track map at the top right: the circuit as a light line over a dark band, the lap driven so far lit in the HUD's accent, the start line, the other cars as dots in their colours and the player's car as an arrow over them; the track's name and length under it. The game gives the shape and every position in the map's box; read only.

`import { RacingTrackMap } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/track-map.tsx`).

| Prop | Type | What |
|---|---|---|
| `path` | `string` | The track's racing line as an SVG path (d) in the map's 260 by 220 box; empty draws no road. |
| `driven` | `string` | The part of the lap the player has driven, as an SVG path in the same box, lit in the HUD's accent; empty draws none. |
| `startX` | `number` | The start line's place in the box, across. |
| `startY` | `number` | The start line's place in the box, down. |
| `startAngle` | `number` | The start line's turn in degrees. |
| `cars?` | `ReadonlyArray<{ id: string; x: number; y: number; heading: number; color: string; player: boolean; name: string }>` | The cars on the map, keyed rows of { id, x, y, heading, color, player, name }: x and y in the map's box; color the car's colour as a CSS colour (empty: neutral); player marks the player's car, drawn as an arrow turned to heading (degrees clockwise from up) over the others' dots; name for a reader. |
| `name` | `string` | The track's name as the game words it, under the map; empty hides it. |
| `length` | `string` | The track's length as the game words it (4.8 km), on a tag beside the name; empty hides it. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `start`, `overflow`.

## 110. RhythmChain

The score chain: a lit plate with the multiplier, and beside it the steps to the next one as leaning pips. The game counts both; read only.

`import { RhythmChain } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/chain.tsx`).

| Prop | Type | What |
|---|---|---|
| `multiplier` | `string` | The score multiplier as the game words it (×4); empty hides the chain. |
| `step` | `number` | How many of the steps to the next multiplier are lit. |
| `steps?` | `number` | The steps to the next multiplier; 0 hides the strip (at its top multiplier). Default 8. |
| `hit` | `number` | Changes when the multiplier rises: the plate pops again. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `empty`, `top`, `overflow`.

## 111. RhythmDifficulty

Choosing a chart: one leaning tile per difficulty with its tier's colour along its top, the tier's name, its level as a large figure, the player's best grade on a tag beside the best score, and Locked or Selected in words. A tile reports choose with its key; a locked chart is disabled and reports nothing. The game owns the charts, unlocking, grades and scores.

`import { RhythmDifficulty } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/difficulty.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The heading (the song's title); empty takes the kit's (Difficulty). |
| `levels` | `ReadonlyArray<{ id: string; tier: string; label: string; rating: number; grade: string; score: string; locked: boolean }>` | The song's charts in the game's order, keyed rows of { id, tier, label, rating, grade, score, locked }: tier the chart's family (easy, normal, hard, expert, master, special; on data-tier for the pack's colour, never alone); label its name as the game words it (Hard); rating its level number; grade the player's best grade on it (S, AA; empty unplayed); score the best score as the game formats it (981,200; empty Not played); locked it cannot be played yet (disabled). |
| `selected` | `string` | The chart the game holds as chosen (selected, named Selected); empty is none. |
| `onChoose?` | `(e: UiEvent, key: string) => void` | Choose the chart; the tile's key. A locked chart is disabled and reports nothing. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `selected`, `disabled`, `empty`, `overflow`.

## 112. RhythmGauge

A tall gauge at a screen edge, cut into ten cells and leaning toward the play field: the run's life (green; danger red with a pulsing rim under the game's mark, Danger and Failed in words) or its fever charge (pink; Ready and the key that spends it). Its height is the space the page gives it. The game owns the value and every verdict; read only.

`import { RhythmGauge } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/gauge.tsx`).

| Prop | Type | What |
|---|---|---|
| `kind?` | `string` | What it measures: life (the run fails at empty) or fever (a charge the player spends when full); on data-kind. Default "life". |
| `value` | `number` | How full, 0 to 1. |
| `side?` | `string` | The screen edge it stands at, start or end: it leans toward the play field. Default "start". |
| `danger` | `boolean` | The game says life is low: the gauge turns to danger, its rim pulses and Danger shows under it. |
| `failed` | `boolean` | The game says the run is failed (life gone): Failed shows under it. |
| `ready` | `boolean` | The game says the charge is ready to spend: the gauge glows and Ready shows. |
| `key` | `string` | The input that spends it as the game names it (Space, RB); empty hides the key cap. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `fever`, `ready`, `danger`, `failed`, `overflow`.

## 113. RhythmJudgement

The play field's centre: the running combo as the hero figure with its word (Combo broken when it breaks, Full combo or All perfect while the run holds one), and under it the last hit's judgement, each in its own lit colour, with Early or Late and the offset. A new hit id pops the word and the count again. The game owns timing windows, judging and counting; this only shows what it was told. Read only; not a live region.

`import { RhythmJudgement } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/judgement.tsx`).

| Prop | Type | What |
|---|---|---|
| `judgement?` | `string` | The last hit's judgement as the game judged it: perfect, great, good, bad, miss, or none (nothing shown); on data-judgement for the pack, and always a word. Default "none". |
| `label` | `string` | The judgement's word as the game names it (Critical, Marvelous); empty takes the kit's word for judgement. |
| `timing` | `string` | early or late under the word (Early, Late); empty hides it. |
| `offset` | `string` | The hit's offset as the game words it (+4 ms, −12 ms), beside the timing; empty hides it. |
| `hit` | `number` | The hit's id: a new value draws the word afresh (each note judged), so the word and the combo pop again. |
| `combo` | `number` | The running combo as the game counts it; 0 hides it. |
| `broken` | `boolean` | The combo just broke: Combo broken shows in its place. |
| `mark` | `string` | The run is still on course for a mark: fullCombo (Full combo), allPerfect (All perfect), or empty for none; on data-mark. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `great`, `good`, `miss`, `overflow`.

## 114. RhythmLaneKeys

The lanes' key caps under the judgement line, spread across the field's width, one per lane, each lit while the player holds it. The game reads input and names the keys; read only.

`import { RhythmLaneKeys } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/lane-keys.tsx`).

| Prop | Type | What |
|---|---|---|
| `lanes` | `ReadonlyArray<{ id: string; key: string; held: boolean }>` | The lanes left to right, keyed rows of { id, key, held }: key the input that plays the lane as the game names it (S, LB); held the player holds it now (the cap lights). |

Fixtures: `default`, `four`, `overflow`.

## 115. RhythmResults

After the song: the grade, Cleared or Failed, the score with the old best and New best, Full combo or All perfect, max combo and accuracy, and the judgement tally. Retry (disabled when the game refuses it) and Next (also Escape, pad B) report once. The game computes every figure, grade and verdict.

`import { RhythmResults } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/results.tsx`).

| Prop | Type | What |
|---|---|---|
| `song` | `string` | The song's title as the heading; empty takes the kit's (Results). |
| `grade` | `string` | The grade the game awarded (S, AA, C); empty hides it. |
| `cleared` | `boolean` | The game's verdict: Cleared, or Failed (data-cleared). |
| `score` | `number` | The run's score as the game counts it, shown with grouping. |
| `previous` | `number` | The player's best before this run (Best …); 0 hides it. |
| `newBest` | `boolean` | The game says this run beat the best: New best shows by the score. |
| `maxCombo` | `number` | The longest combo of the run. |
| `accuracy` | `string` | The accuracy as the game words it (97.42%); empty hides it. |
| `mark` | `string` | A run mark the game awarded: fullCombo (Full combo), allPerfect (All perfect), or empty for none; on data-mark. |
| `counts` | `ReadonlyArray<{ id: string; judgement: string; label: string; count: number }>` | The judgement tally in the game's order, keyed rows of { id, judgement, label, count }: judgement the kind (perfect, great, good, bad, miss; on data-judgement), label its word as the game names it, count how many. |
| `canRetry` | `boolean` | Retry is offered (false: disabled). |
| `onRetry?` | `(e: UiEvent) => void` | Retry; silent when the game refuses it. |
| `onNext?` | `(e: UiEvent) => void` | Next; also Escape and the back action (pad B). |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `failed`, `fullCombo`, `allPerfect`, `disabled`, `empty`, `overflow`.

## 116. RhythmScoreHud

The run's figures over the play field, set against the screen's end: the score as the hero figure, the accuracy with the grade it is on course for, and the player's best with the gap to it. The game owns every figure; read only. Pair it with RhythmGauge for life and fever and RhythmSongProgress for the song's progress.

`import { RhythmScoreHud } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/score-hud.tsx`).

| Prop | Type | What |
|---|---|---|
| `score` | `number` | The score as the game counts it, shown with grouping. |
| `accuracy` | `string` | The accuracy as the game words it (98.42%); empty hides it. |
| `grade` | `string` | The grade the run is on course for as the game names it (S, AA); empty hides it. |
| `best?` | `number` | The player's best on this chart; 0 hides it. Ahead or behind it shows as a signed gap beside it (on data-trend). Default 0. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `ahead`, `first`, `start`, `overflow`.

## 117. RhythmSongPlate

The song in play, at the screen's top corner: the jacket with a lit edge, the title, the artist and tempo, the chart's chip in its tier's colour with its level, and the modifiers in play. Read only.

`import { RhythmSongPlate } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/song-plate.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The song's title. |
| `artist` | `string` | The artist as the game words it; empty hides it. |
| `bpm` | `string` | The tempo as the game words it (174, 90–180); empty hides it. |
| `jacket` | `Texture \| 0` | The cover art texture (og.load.texture); 0 shows the music glyph in its place. |
| `tier` | `string` | The chart's family (easy, normal, hard, expert, master, special): the colour of its chip, on data-tier, never alone. |
| `chart` | `string` | The chart's name as the game words it (Expert); empty hides the chip. |
| `rating` | `number` | The chart's level, beside its name; 0 hides it. |
| `modifiers` | `ReadonlyArray<{ id: string; label: string }>` | The modifiers in play, keyed rows of { id, label } in the game's order, each on a tag (×1.0, Mirror). |

Pack slots: `Chip.frame`.

Fixtures: `default`, `plain`, `master`, `special`, `overflow`.

## 118. RhythmSongProgress

The song's progress as a thin lit line along the screen's top edge, and the time played over the length riding under its played end. The game keeps time; read only.

`import { RhythmSongProgress } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/song-progress.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | How far through the song, 0 to 1. |
| `elapsed` | `string` | The time played as the game words it (2:07); empty hides the time. |
| `length` | `string` | The song's length as the game words it (3:21), after the time played. |

Fixtures: `default`, `start`, `end`, `bare`, `overflow`.

## 119. RhythmSongSelect

Choosing a song: the game's list (jacket, title, artist, BPM, with New, Cleared, Locked, Selected and Previewing named in words) beside the selected song's details. Each song is a cut plate of dark glass; the selected one is washed in the HUD's accent and stepped in. A row reports choose with its key; Preview (the button, or the details action on a row) reports preview with its key. A locked song is a disabled row and reports nothing. The game owns unlocking, the selection, the preview audio and the loading; status shows its loading and error states, and the error state offers Retry (retry).

`import { RhythmSongSelect } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/song-select.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The page's heading (a pack or folder name); empty takes the kit's (Songs). |
| `songs` | `ReadonlyArray<{ id: string; title: string; artist: string; bpm: string; length: string; jacket: Texture \| 0; locked: boolean; fresh: boolean; cleared: boolean }>` | The songs in the game's order, keyed rows of { id, title, artist, bpm, length, jacket, locked, fresh, cleared }: bpm and length as the game words them (128, 90–180; 2:14); jacket the cover art texture (og.load.texture, 0 none); locked the song cannot be played yet (its row is disabled); fresh newly added (New); cleared the player has cleared it (Cleared). |
| `selected` | `string` | The song in focus as the game holds it (its row selected, its details shown); empty is none. |
| `previewing` | `string` | The song whose preview the game is playing (Previewing); empty is none. |
| `status` | `string` | The list's state: empty when the songs are there, loading while the game fetches them, error when it could not. |
| `onChoose?` | `(e: UiEvent, key: string) => void` | Choose the song; the row's key. A locked song's row is disabled and reports nothing. |
| `onPreview?` | `(e: UiEvent, key: string) => void` | Preview; the details action (X, pad Y) on a focused row, with that row's key, and the Preview button with the selected song's key. |
| `onRetry?` | `(e: UiEvent) => void` | Retry; offered only while status is error. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `selected`, `disabled`, `empty`, `loading`, `error`, `overflow`.

## 120. RhythmTiming

How the run is being judged, at a corner of the field: the tally so far, each judgement in its colour, and under it the timing scale from Early to Late with the beat at its centre and the latest hits as ticks, the newest brightest. The game judges and counts; read only.

`import { RhythmTiming } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/timing.tsx`).

| Prop | Type | What |
|---|---|---|
| `counts` | `ReadonlyArray<{ id: string; judgement: string; label: string; count: number }>` | The judgement tally so far in the game's order, keyed rows of { id, judgement, label, count }: judgement the kind (perfect, great, good, bad, miss; its colour, on data-judgement), label its word as the game names it, count how many. |
| `hits` | `ReadonlyArray<{ id: string; judgement: string; offset: number; age: number }>` | The latest hits on the timing scale, keyed rows of { id, judgement, offset, age }: offset from −1 (earliest the game judges) through 0 (on the beat) to 1 (latest); age from 0 (this hit) to 1 (about to drop), which fades it. The game keeps the list short (eight or so). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `early`, `empty`, `overflow`.

## 121. RpgCharacter

A character sheet: a portrait with name and class, two columns of equipment slots either side of the full figure, and the attributes in two columns beneath. The game owns what fits where; the sheet reports choose (a slot's key) and unequip (X or pad Y on the focused slot, its key).

`import { RpgCharacter } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/character.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | A heading band over the figure (Equipment); empty shows the name header instead. |
| `name` | `string` | The character's name; with an empty title it heads the sheet beside the portrait. |
| `line` | `string` | The class and level line as the game words it (Rogue · Level 12). |
| `portrait` | `Texture \| 0` | A head-and-shoulders texture (og.load.texture); 0 hides it. |
| `figure` | `Texture \| 0` | A full-figure texture shown between the slot columns; 0 leaves the ground. |
| `slots?` | `ReadonlyArray<{ id: string; side: string; label: string; icon: string; rarity: string }>` | The equipment slots, keyed rows of { id, side, label, icon, rarity }: side is start or end (the column it stands in, mirrored right to left); an empty icon is an empty slot showing its label. |
| `selected` | `string` | The selected slot's id; empty is none. |
| `attributes?` | `ReadonlyArray<{ id: string; group: string; icon: string; label: string; value: string }>` | The attributes, keyed rows of { id, group, icon, label, value }: group a or b, the two columns; empty hides them (a page with RpgStats beside it). |
| `compact` | `boolean` | Its place is narrow (under about 300 px): the attributes (their split goes) tighten. The place decides, not the screen. |
| `dense` | `boolean` | A wide but short place (a laptop window): smaller slots and tighter attributes. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | The choose event, with the slot's key. |
| `onUnequip?` | `(e: UiEvent, id: string) => void` | Unequip, from the focused slot (X or pad Y, the details action), with the slot's key. |

Pack slots: `Button.plate`, `Panel.frame`.

Fixtures: `default`.

## 122. RpgCombo

The combo counter beside the fight: the hits as a large slanted figure with its word, the window still to run as a decay bar under it and the damage multiplier the chain gives. The game owns the chain; read only.

`import { RpgCombo } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/combo.tsx`).

| Prop | Type | What |
|---|---|---|
| `count` | `number` | The hits in the chain; 0 hides the counter. |
| `word` | `string` | The word after the figure as the game words it; empty takes the component's Hits. |
| `remaining?` | `number` | The share of the chain's window still to run, 0 to 1 (the decay bar); negative hides the bar. |
| `multiplier?` | `number` | The damage multiplier the chain gives (Damage ×1.6); 0 or 1 hides it. |

Fixtures: `default`, `fading`, `empty`, `overflow`.

## 123. RpgCommands

A turn's commands as a row of plates, each an icon over its label; the selected one is the primary plate. Reports choose (a command's key).

`import { RpgCommands } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/commands.tsx`).

| Prop | Type | What |
|---|---|---|
| `commands?` | `ReadonlyArray<{ id: string; label: string; icon: string; enabled: boolean }>` | The commands in order, keyed rows of { id, label, icon, enabled }: label as the game words it (Attack, Skills, Spells, Items, Defend, Flee). |
| `selected` | `string` | The command the focus rests on; it takes the primary plate. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | The choose event, with the command's key. |

Pack slots: `Button.plate.…`, `Panel.frame`.

Fixtures: `default`.

## 124. RpgCompanions

The leader's companions as small portraits with health bars, a companion under the low share turning to danger. Read only.

`import { RpgCompanions } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/companions.tsx`).

| Prop | Type | What |
|---|---|---|
| `members?` | `ReadonlyArray<{ id: string; name: string; portrait: Texture \| 0; health: number; healthMax: number; ghost: number; mana: number; manaMax: number; key: string; fallen: boolean }>` | The leader's companions in the game's order, keyed rows of { id, name, portrait, health, healthMax, ghost, mana, manaMax, key, fallen }: portrait a texture (0: the person glyph); ghost the health before the latest loss; manaMax 0 hides the mana bar; key the input that switches to them as the game names it (2), on a key cap at the plate's end, empty for none; fallen out of the fight (on data-fallen: the plate dims, the portrait greys and a Fallen tag stands beside the name). |
| `low?` | `number` | The share of health under which a companion is low: the fill turns to danger and the rim glows (0.25). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `fallen`, `critical`, `overflow`.

## 125. RpgEffects

The buffs and debuffs on the leader, under the vitals: a row of HudEffect tiles. The row only spaces them; a game that wants another arrangement lays the tile out itself. The game owns every timer; read only.

`import { RpgEffects } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/effects.tsx`).

| Prop | Type | What |
|---|---|---|
| `effects?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; remaining: number; label: string; harmful: boolean }>` | The buffs and debuffs on a character in the game's order, keyed rows of { id, name, icon, glyph, remaining, label, harmful }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); remaining the share of its time still to run, 0 to 1 (a dark radial sweep over the art; negative: no timer); label its time or stacks as the game words it, on a tag (0:12, ×3), empty for none; harmful a debuff (on data-harmful, never colour alone: the edge and the tag's ink both change). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `untimed`, `expiring`, `overflow`.

## 126. RpgEnemyInfo

The targeted enemy: name, portrait, level, a health bar with its figure and its status effects. Read only.

`import { RpgEnemyInfo } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/enemy-info.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The enemy's name. |
| `portrait` | `Texture \| 0` | Its portrait (og.load.texture); 0 is none. |
| `level` | `number` | Its level. |
| `health` | `number` | Health now. |
| `healthMax` | `number` | Most health. |
| `effects?` | `ReadonlyArray<{ id: string; icon: string; label: string; tone: string }>` | Status effects and traits, keyed rows of { id, icon, label, tone }: tone harm (a debuff on it), boon (a buff), weak (vulnerable to) or strong (resistant to). |

Pack slots: `Plate.frame`.

Fixtures: `default`.

## 127. RpgExperience

The experience line across the screen's foot: a thin violet line lit to the share of the way to the next level, with the level and that share in small capitals over its start, inset from the screen's edge as the HUD is. Read only.

`import { RpgExperience } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/experience.tsx`).

| Prop | Type | What |
|---|---|---|
| `level` | `number` | The level now; 0 hides the caption's level. |
| `value` | `number` | The share of the way to the next level, 0 to 1. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `empty`, `full`.

## 128. RpgHints

Input hints at the screen's foot: a row of key caps, each with what it does in small tracked capitals. The row only spaces them; a game lays a hint out anywhere else itself. Read only; hidden under a touch screen.

`import { RpgHints } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/hints.tsx`).

| Prop | Type | What |
|---|---|---|
| `hints?` | `ReadonlyArray<{ id: string; key: string; label: string }>` | The inputs worth a reminder, keyed rows of { id, key, label }: key the input as the game names it (Tab, Space) on a key cap, label what it does (Target). Under a touch screen the hints are hidden: the controls on screen say it. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `overflow`.

## 129. RpgHits

Damage and healing figures beside the fight: a critical hit's figure largest, gold and slanted under its call, older figures stepping down and away, a burn in an ember ink with its word, a heal in green. The game owns every figure and where the page puts them; read only.

`import { RpgHits } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/hits.tsx`).

| Prop | Type | What |
|---|---|---|
| `hits?` | `ReadonlyArray<{ id: string; amount: number; kind: string; call: string; word: string }>` | The latest damage and healing figures, newest first, keyed rows of { id, amount, kind, call, word }: kind normal, critical (gold and slanted, the largest), burn (damage over time in an ember ink, its word beside it), heal (green with a plus) or miss (on data-kind); call the words over a critical as the game words them (Critical · Weak point), empty for none; word the figure's kind as the game words it (burn), empty for none. The newest stands largest; when there are older ones it keeps room before it, where they step down and away (a phone shows the newest alone); the game trims the list and places it. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `heal`, `miss`, `overflow`.

## 130. RpgLeader

The party leader's vitals at the screen's top corner, built from the RPG HUD's blocks: a portrait in a large octagon, the name and level, health as a figure over its most, and three bars: a fat health bar that leaves a warm ghost when it drops and turns to danger when low, a mana bar and a stamina bar cut into cells. Read only.

`import { RpgLeader } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/leader.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The leader's name; empty hides it. |
| `level` | `number` | The level beside the name; 0 hides it. |
| `portrait` | `Texture \| 0` | A head-and-shoulders texture (og.load.texture); 0 draws the person glyph. |
| `health` | `number` | Health now, shown as a figure over its most and as the fat bar. |
| `healthMax` | `number` | Most health. |
| `ghost` | `number` | Health before the latest loss: a warm ghost behind the fill that drains after it (at or under health: none). |
| `mana` | `number` | Mana now. |
| `manaMax` | `number` | Most mana; 0 hides the mana bar. |
| `stamina` | `number` | Stamina (or energy) now. |
| `staminaMax` | `number` | Most stamina; 0 hides the stamina bar. |
| `segment?` | `number` | Stamina per cell of the stamina bar (one dodge's cost); 0 draws it whole. Default 0. |
| `low?` | `number` | The share of health under which it is low: the fill turns to danger and the bar's rim glows (0.25). Default 0.25. |

Fixtures: `default`, `full`, `low`, `critical`, `empty`, `overflow`.

## 131. RpgLockOn

The lock-on reticle the page centres on the locked target: lit arcs round it with a glow in the HUD's accent. The game decides what is locked and where it stands on screen; read only.

`import { RpgLockOn } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/lock-on.tsx`).

| Prop | Type | What |
|---|---|---|
| `target` | `string` | The locked target's name for a reader; empty hides the reticle (nothing is locked). |

Fixtures: `default`, `empty`.

## 132. RpgLootFeed

The loot feed low on the screen's side: a column of HudLootEntry blocks, newest first, each an icon, the item's name, its rarity on a tag and the count gained on a dark glass with its edge tinted by rarity. Read only.

`import { RpgLootFeed } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/loot-feed.tsx`).

| Prop | Type | What |
|---|---|---|
| `items?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; rarity: string; label: string; quantity: number; currency: boolean; fading: boolean }>` | What the player just looted, newest first, keyed rows of { id, name, icon, glyph, rarity, label, quantity, currency, fading }: icon a texture (0: the glyph, an icon name, in its place); rarity common, uncommon, rare, epic or legendary (the edge's colour, on data-rarity; an epic or legendary item's name is inked too); label the rarity as the game words it (Epic) on a tag, empty for none; quantity the count gained (0 hides it), written ×3, or +1,240 when currency is true (gold); fading an old row on its way out (dimmed). A new row slides in from the screen's edge; the game trims old rows. |

Fixtures: `default`, `rare`, `overflow`.

## 133. RpgMenuBar

The menu's top bar: its screens as tabs and a close button. Reports choose (a screen's key), tabPrev, tabNext and close (the button, or the tabPrev, tabNext and back input actions).

`import { RpgMenuBar } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/menu-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `tabs?` | `ReadonlyArray<{ id: string; label: string }>` | The menu's screens, keyed rows of { id, label } (Character, Inventory, Skills, Quests, Map). |
| `tab` | `string` | The screen shown. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a screen (its key). |
| `onTabPrev?` | `(e: UiEvent) => void` | Previous screen. |
| `onTabNext?` | `(e: UiEvent) => void` | Next screen. |
| `onClose?` | `(e: UiEvent) => void` | Close (the button, Escape or B). |

Pack slots: `Button.plate`.

Fixtures: `default`.

## 134. RpgMinimap

The minimap at the screen's top corner: the map in a dark round well with a lit ring, the party, the foes and the objective as blips, the player's arrow at the centre turned to the heading and N on the ring's top; the place under it with the time of day on a tag. Read only.

`import { RpgMinimap } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/minimap.tsx`).

| Prop | Type | What |
|---|---|---|
| `map` | `Texture \| 0` | The map texture (og.load.texture); 0 draws the dark well alone. |
| `centerX?` | `number` | The map point under the player, 0 to 1 across. Default 0.5. |
| `centerY?` | `number` | The map point under the player, 0 to 1 down. Default 0.5. |
| `zoom?` | `number` | How much of the map the minimap shows across, 0 to 1 (0.5). Default 0.5. |
| `heading` | `number` | The player's heading in degrees clockwise from north: north stays up and the player's arrow turns. |
| `blips?` | `ReadonlyArray<{ id: string; kind: string; x: number; y: number; label: string }>` | What the minimap shows, keyed rows of { id, kind, x, y, label }: kind ally, foe, boss or objective (on data-kind, each its own shape and colour); x and y on the map, 0 to 1; label names it for a reader. A blip off the minimap's edge is clipped. |
| `place` | `string` | Where the player is as the game words it (Ashen Vale), under the map; empty hides it. |
| `clock` | `string` | The world's time of day as the game words it (06:40), on a tag beside the place; empty hides it. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `empty`, `edge`, `overflow`.

## 135. RpgParty

The party as a row of portrait cards: a level seal, the portrait, the name, and a health and a mana HudBar with their figures, filled in the page's health and mana colours. Reports choose (a member's key). The cards share the width evenly.

`import { RpgParty } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/party.tsx`).

| Prop | Type | What |
|---|---|---|
| `members?` | `ReadonlyArray<{ id: string; name: string; portrait: Texture \| 0; level: number; health: number; healthMax: number; mana: number; manaMax: number; down: boolean }>` | Keyed rows of { id, name, portrait, level, health, healthMax, mana, manaMax, down }: portrait a texture (og.load.texture), 0 none; down greys the card and marks it. |
| `selected` | `string` | The member chosen; empty is none. |
| `dense` | `boolean` | A wide but short place (a laptop window): wider, shorter portraits. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose (a member's key). |

Pack slots: `Chip.frame`.

Fixtures: `default`, `empty`.

## 136. RpgPlaceCard

A place picked on the map, on dark glass: its picture at the start fading into the glass, its name in the HUD's title type over a gold rule, a line about it, and two buttons, Mark (a lit ghost plate) and Fast travel (the gold primary plate). Reports mark (marking or unmarking) and travel (fast travel). Narrow the line about it is left out, so the name and the buttons keep their room.

`import { RpgPlaceCard } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/place-card.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The place's name. |
| `about` | `string` | A line or two about it; empty hides it. |
| `picture` | `Texture \| 0` | Its picture (og.load.texture); 0 hides it. |
| `marked` | `boolean` | The player has marked it: the button offers Unmark. |
| `canTravel` | `boolean` | Fast travel to it is possible: the button is enabled. |
| `onMark?` | `(e: UiEvent) => void` | Mark or unmark. |
| `onTravel?` | `(e: UiEvent) => void` | Fast travel. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`.

Fixtures: `default`, `marked`.

## 137. RpgPrompt

What the player can do now, over the ability bar: a key cap and the action in words (a finisher, an interaction). The game decides when it shows and what the input does; read only.

`import { RpgPrompt } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/prompt.tsx`).

| Prop | Type | What |
|---|---|---|
| `key` | `string` | The input that does it as the game names it (F, Hold E); empty hides the key cap. |
| `text` | `string` | What it does as the game words it (Finisher: Sunder the guard); empty hides the prompt. |
| `disabled` | `boolean` | It cannot be done now: the prompt dims and the reason shows. |
| `reason` | `string` | Why it cannot (Guard holds); shown while disabled. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `disabled`, `overflow`.

## 138. RpgQuestDetail

One quest: its kind in gold small capitals over its name on the heading band (over the scenery, shaded into the glass, when the game gives one), the story so far, its objectives as stages with their tasks on a rail under them, and its rewards as cut chips, each with its icon in an octagon socket (one with a detail, a unique reward, lit gold). A done objective is struck with its diamond filled in the success colour; the first stage not done is lit gold. The body scrolls inside the frame's height, fading out above the actions (Show on map, Track or Stop tracking), which stay at its foot; they report locate and track. Track is the primary action; once the quest is tracked it reads Stop tracking on a quiet secondary button. With neither action it is read only.

`import { RpgQuestDetail } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/quest-detail.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The quest's name, in the heading band. |
| `summary` | `string` | Its story so far, as the game writes it. |
| `objectives?` | `ReadonlyArray<{ id: string; text: string; done: boolean; optional: boolean; step: number }>` | Keyed rows of { id, text, done, optional, step }: step 0 is a stage, 1 a task under it; done ticks it, optional marks it so. |
| `rewards?` | `ReadonlyArray<{ id: string; icon: string; label: string; detail: string }>` | Keyed rows of { id, icon, label, detail }: detail a second, quieter line (Unique relic); empty hides it. |
| `label` | `string` | What kind of quest it is, as the game words it (Main quest), over the name; empty hides it. |
| `tracked` | `boolean` | The HUD follows this quest: its action reads Stop tracking. |
| `canTrack` | `boolean` | Show the Track (or Stop tracking) action, which reports track. |
| `canLocate` | `boolean` | Show the Show on map action, which reports locate. |
| `scenery` | `Texture \| 0` | A texture beside the rewards (og.load.texture), with its own space; 0 is none. |
| `onTrack?` | `(e: UiEvent) => void` | Track or Stop tracking. |
| `onLocate?` | `(e: UiEvent) => void` | Show on map. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `long`.

## 139. RpgSkills

The ability bar at the foot of the screen, a row of HudSlot blocks: the consumables in smaller slots, set apart before the skills, then each skill, and the ultimate, larger, at the end. The slot draws itself (its cooldown, cost, charges and key cap are HudSlot's); the bar only sizes and spaces them. A game that wants another bar lays HudSlot out itself. The game owns every cooldown, cost and charge; read only.

The bar's per-kind sizes (a smaller item slot, a larger ultimate, the compact shrink) are the block's own look now: HudSlot draws one size.

`import { RpgSkills } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/skills.tsx`).

| Prop | Type | What |
|---|---|---|
| `slots?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; key: string; kind: string; cooldown: number; time: string; count: number; ready: boolean; unaffordable: boolean; reason: string; charges: number; chargesMax: number; charge: number }>` | The bar's slots in the game's order, keyed rows of { id, name, icon, glyph, key, kind, cooldown, time, count, ready, unaffordable, reason, charges, chargesMax, charge }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); key the input as the game names it (1, Q), on a key cap under the slot; kind skill, item (a consumable: a smaller slot with its count on a tag, set apart before the skills; a phone leaves items out) or ultimate (a larger hexagon inside a gold ring that fills with charge, 0 to 1, its share in figures until full; on data-kind); cooldown the share still to run, 0 to 1 (a dark radial sweep over the art with a lit leading edge), with time its figure as the game words it (8); count the items held (×5; 0 hides it); ready lit (a lit edge and glow); unaffordable it cannot be paid for (the art greys under a veil and reason, as the game words it (No mana), stands on it); chargesMax 0 hides the charge pips, else charges of them are lit over the slot. |

Fixtures: `default`, `cooling`, `unaffordable`, `overflow`.

## 140. RpgStats

A character's figures in two columns of grouped rows, each an icon, a label and a value. Read only; fills its height and scrolls inside it.

`import { RpgStats } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/stats.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading (Stats). |
| `groups?` | `ReadonlyArray<{ id: string; column: string }>` | The groups, keyed rows of { id, column }: column a or b; a rule parts one group from the next in a column. |
| `rows?` | `ReadonlyArray<{ id: string; group: string; icon: string; label: string; value: string }>` | The figures, keyed rows of { id, group, icon, label, value }: group a group's id; value as the game words it. |

Pack slots: `Panel.frame`.

Fixtures: `default`.

## 141. RpgTurnOrder

The initiative: round portraits in acting order, the one acting first and ringed, between two end flourishes, and a round plate on a rule under them. All portraits share the width, so five fit a phone. Reports nothing.

`import { RpgTurnOrder } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/turn-order.tsx`).

| Prop | Type | What |
|---|---|---|
| `round` | `number` | The round, on the plate under the portraits. |
| `turns?` | `ReadonlyArray<{ id: string; name: string; portrait: Texture \| 0; side: string; acting: boolean; down: boolean }>` | Keyed rows of { id, name, portrait, side, acting, down }, in the order they act: side ally, enemy or neutral; acting the one whose turn it is; portrait a texture, 0 none. |

Pack slots: `Chip.frame`.

Fixtures: `default`.

## 142. RpgVitals

A character's vitals: a round portrait with a level badge and health and mana bars beside it. large is the player's plate, with the figures on the bars; compact is a party row. Read only.

`import { RpgVitals } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/vitals.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The character's name; read out, and shown on the large plate. |
| `portrait` | `Texture \| 0` | A head-and-shoulders texture (og.load.texture); 0 is none. |
| `level` | `number` | The level on the badge; 0 hides the badge. |
| `health` | `number` | Health now. |
| `healthMax` | `number` | Most health; the bar shows health over it. |
| `mana` | `number` | Mana now. |
| `manaMax` | `number` | Most mana; 0 hides the mana bar. |
| `plate` | `string` | large (the player's plate, figures on the bars) or compact (a party row). |
| `down` | `boolean` | Out of the fight: the plate greys. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `compact`.

## 143. RpgWorldMap

A map with its markers, a one-line legend and an optional caption. The map's art is the screen: no panel lies over it, only each marker's name on a small cut plate, the legend and the caption. A marker's kind is its shape as well as its colour (a quest a gold diamond, a waypoint a cyan ring, danger a red triangle, the player a cyan disc with an arrow); its name is white at any kind. The chosen marker is ringed and larger. The map is drawn to cover the box it is given, times zoom and moved by panX and panY; markers sit at fractions of it, keep their size at any zoom and report choose (a marker's key). The player's marker is not a button target. The game owns the zoom and the pan: the map reports zoom (the wheel, with deltaY), pinch (two touches, with scale), and grab, pan and release (a press, its moves and its end, each with x and y in the map's box), and draws what the game sets.

`import { RpgWorldMap } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/world-map.tsx`).

| Prop | Type | What |
|---|---|---|
| `map` | `Texture \| 0` | The map's texture (og.load.texture); markers sit on it in fractions of its size. |
| `markers?` | `ReadonlyArray<{ id: string; kind: string; x: number; y: number; label: string }>` | Keyed rows of { id, kind, x, y, label }: kind is quest, waypoint, danger or player; x and y 0 to 1 across and down the map. |
| `selected` | `string` | The marker selected; empty is none. |
| `zoom?` | `number` | How far the map is zoomed in: 1 (and 0, and when left out) fits it to the box, 2 shows it twice the size. The game keeps it, from the zoom and pinch events, within its own limits. |
| `panX?` | `number` | How far the zoomed map is moved across, CSS px (0, and when left out, pins its start corner). The game keeps it, from grab, pan and release. |
| `panY?` | `number` | How far the zoomed map is moved down, CSS px; 0 when left out. |
| `legend?` | `ReadonlyArray<{ id: string; kind: string; label: string }>` | The legend, keyed rows of { id, kind, label }, one line along the map's foot; empty hides it. |
| `here` | `string` | A caption pinned to the map's top corner (Current location); empty hides it. |
| `onZoom?` | `(e: UiEvent) => void` | The wheel zooms: the game reads the wheel's delta and keeps the zoom within its limits. |
| `onGrab?` | `(e: UiEvent) => void` | A press on the map begins a grab. |
| `onPan?` | `(e: UiEvent) => void` | A grabbed map's moves pan it, each with x and y in the map's box. |
| `onRelease?` | `(e: UiEvent) => void` | The grab's end releases it. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a marker; the marker's key. The player's marker is not a button target. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `places`.

## 144. ShooterAbilities

The ability bar at the foot of the screen: each ability in an octagon with a lit edge, its cooldown a dark sweep over the art with the seconds left, its charges on a tag and its key cap under it; the ultimate in a larger hexagon inside a ring that fills with its charge. The game owns every cooldown and charge; read only. Approximations: the octagon and hexagon cuts are rounded; the cooldown's conic mask is a top-down veil; the charge ring's SVG arc is a View stand-in that fills from its foot; the ready glow and the grey-out filters are shadows and opacity.

`import { ShooterAbilities } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/abilities.tsx`).

| Prop | Type | What |
|---|---|---|
| `slots?` | `ReadonlyArray<AbilitySlotData>` | The abilities and items on the bar in the game's order. The game owns every cooldown and charge; read only. |

Each `AbilitySlotData`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls it (Phase dash). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `key?` | `string` | The input as the game names it (Q), on a key cap under the slot; empty is none. |
| `cooldown?` | `number` | The share of its cooldown still to run, 0 to 1: a dark sweep over the art. |
| `time?` | `string` | The seconds left as the game words it (6); shown over the sweep. |
| `count?` | `number` | The charges or items held, on a tag (×2; 0 hides it). |
| `ready?` | `boolean` | Lit (a lit edge and glow). |
| `disabled?` | `boolean` | Greyed. |
| `ultimate?` | `boolean` | The ultimate: larger, in a ring that fills with its charge. |
| `charge?` | `number` | The ultimate's charge, 0 to 1; its share in figures until it is ready. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `ready`, `cooling`, `overflow`.

## 145. CombatHud

The shooter's whole HUD: the reticle and the hit directions centred, the killfeed at the top's end, the vitals at the bottom's start and the ammunition at the bottom's end. The edges sit inside a fixed inset, so a phone's notch and home bar are the caller's (the svelte's safe-area insets and its right-to-left mirror are dropped), and a phone's compact reflow keeps the corners.

`import { CombatHud } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/combat-hud.tsx`).

| Prop | Type | What |
|---|---|---|
| `health?` | `number` | Health now. |
| `healthMax?` | `number` | Most health. Default 1. |
| `shield?` | `number` | Shield now, a strip over the health bar. |
| `shieldMax?` | `number` | Most shield; 0 hides the strip. |
| `spread?` | `number` | Crosshair spread, 0 to 1 (Crosshair). |
| `hit?` | `string` | Crosshair hit mark: none, hit, headshot, kill or shieldBreak. Default none. |
| `hitAt?` | `number` | The hit's id; a new one replays the mark. |
| `aimed?` | `boolean` | The aim rests on an enemy. |
| `shape?` | `string` | Crosshair shape: cross, dot or circle. Default cross. |
| `weapon?` | `string` | Weapon name (AmmoCounter). |
| `mode?` | `string` | Fire mode: auto, burst or single. Default auto. |
| `magazine?` | `number` | Rounds in the magazine. |
| `capacity?` | `number` | Magazine size. |
| `reserve?` | `number` | Rounds in reserve. |
| `reloading?` | `number` | Reload progress 0 to 1; negative when not reloading. Default -1. |
| `hits?` | `ReadonlyArray<DamageDirectionHit>` | Incoming hits (DamageDirection). |
| `feed?` | `ReadonlyArray<KillfeedRow>` | Killfeed rows, newest first (Killfeed). |

Pack slots: `Plate.frame`.

Fixtures: `default`.

## 146. ShooterDamage

Damage the player deals, beside the reticle: the newest hit's figure largest with its word (Headshot) over it, the older ones stepping down and away from the aim. Read only. The rows' pop-in and fade-out transitions have no TSX surface and are dropped.

`import { ShooterDamage } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/damage.tsx`).

| Prop | Type | What |
|---|---|---|
| `hits?` | `ReadonlyArray<DamageHit>` | The player's latest hits, newest first. The newest stands largest beside the reticle and the older ones step down and away (a phone shows the newest alone); the game trims the list (three or so) and a new row pops in. Read only. |

Each `DamageHit`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `amount` | `number` | The damage dealt. |
| `kind?` | `string` | normal, headshot, shield (it struck a shield) or kill. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `shield`, `kill`, `overflow`.

## 147. ShooterEffects

The effects on the player, over the vitals: each a small tile with two corners cut, its edge green for a boon and red for a harm, the time already run as a dark sweep round its art and its time or stacks on a tag under it. The game owns every timer; read only. The tiles' cut corners have no TSX surface: they are rounded ones.

`import { ShooterEffects } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/effects.tsx`).

| Prop | Type | What |
|---|---|---|
| `effects?` | `ReadonlyArray<ShooterEffect>` | The effects on the player in the game's order. Read only. |

Each `ShooterEffect`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls it (Haste). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `remaining?` | `number` | The share of its time still to run, 0 to 1: a dark sweep over the art; negative is no timer. |
| `label?` | `string` | Its time or stacks as the game words it (0:12, ×3), on a tag under the tile; empty is none. |
| `harmful?` | `boolean` | A debuff (on data-harmful, never colour alone: the edge and the tag's ink both change). |

Pack slots: `Chip.frame`.

Fixtures: `default`, `untimed`, `overflow`.

## 148. Loadout

A loadout screen over the scene: the title in large condensed capitals, a category's name over a rule, each item a cut plate of smoked glass (the chosen one ringed in the accent and washed in it), Equipped on a green tag, the price in gold; names wrap on whole words. Approximations: the chosen wash is flat and keeps no rim, the Equipped tag's sheen is dropped, the scroll fade at the list's foot is dropped, and the safe-area padding and short-screen reflow are the caller's.

`import { Loadout } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/loadout.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The page's heading; empty takes the kit's (shooter.loadout.title, Loadout). |
| `categories?` | `ReadonlyArray<LoadoutCategory>` | The slot groups in order. |
| `slots?` | `ReadonlyArray<LoadoutSlot>` | What can be carried. |
| `selected?` | `string` | The slot id selected now (empty: none). |
| `onSelect?` | `(e: UiEvent, id: string) => void` | Choose a slot. |
| `onEquip?` | `(e: UiEvent, id: string) => void` | Equip. |
| `onBack?` | `(e: UiEvent) => void` | Back. |

Each `LoadoutCategory`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `label?` | `string` | An empty label takes the kit's string for the id (shooter.loadout.category.<id>). |

Each `LoadoutSlot`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `category` | `string` | A category's id. |
| `name` | `string` | What it is. |
| `icon?` | `string` | An icon of the style's atlas. |
| `detail?` | `string` | A short line under the name (ammunition, damage). |
| `equipped?` | `boolean` | It is carried now. |
| `locked?` | `boolean` | It cannot be chosen yet (shown with its cost). |
| `cost?` | `string` | Its cost text; empty is none. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`.

## 149. ShooterObjectives

The objectives at the screen's top corner: a heading in the HUD's accent, the objective in play lit with its progress on a tag, the others dimmed under it with theirs (a phone shows the one in play alone). Read only. The tags' scrolling overflow (the svelte's -og-scroll marquee) has no TSX surface: long values truncate.

`import { ShooterObjectives } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/objectives.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The block's heading as the game words it; empty takes the component's Objective. |
| `goals?` | `ReadonlyArray<ShooterGoal>` | The objectives in the game's order. The one in play is lit with its progress on a tag, the others dimmed under it with theirs (a phone shows the one in play alone). The game owns progress and completion. Read only. |

Each `ShooterGoal`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `text` | `string` | The objective as the game words it. |
| `tag?` | `string` | Its progress or time on a tag (2 / 3, 0:45), empty for none. |
| `current?` | `boolean` | The one in play (lit; the others are dimmed). |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `one`, `empty`, `overflow`.

## 150. ShooterPickups

The pickup feed lower on the screen's side: each row an icon, the item's name and the count gained on a dark glass that fades toward the middle, its edge at the screen's side tinted by rarity. Read only. A row the For removes fades out on exit: by itself; the rows' FLIP reflow (the svelte's animate:flip) has no TSX surface and is dropped, as is the row's right-to-left mirror.

`import { ShooterPickups } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/pickups.tsx`).

| Prop | Type | What |
|---|---|---|
| `items?` | `ReadonlyArray<PickupItem>` | What the player just picked up, newest first. The game trims old rows (a phone shows the newest); a new row slides in from the screen's edge and an old one fades out. Read only. |

Each `PickupItem`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What the game calls it. |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `quantity?` | `number` | The count gained (+30). |
| `rarity?` | `string` | common, rare, epic or legendary: the edge's and the count's colour. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `epic`, `overflow`.

## 151. ShooterPrompt

What the player can do with what is in front of them, under the aim: the key cap and the action in words. The game decides when it shows and what the input does; read only. The key cap's scrolling overflow (the svelte's -og-scroll marquee) has no TSX surface: long values truncate.

`import { ShooterPrompt } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/prompt.tsx`).

| Prop | Type | What |
|---|---|---|
| `key?` | `string` | The input that does it as the game names it (E, X, Hold F); empty hides the key cap. |
| `text` | `string` | What it does as the game words it (Open supply bin); empty hides the prompt. |
| `disabled?` | `boolean` | It cannot be done now: the prompt dims and the reason shows. |
| `reason?` | `string` | Why it cannot (Locked); shown while disabled. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `disabled`, `empty`, `overflow`.

## 152. ShooterRadar

A round radar: the map turned so north stays up around the player's point, the player's arrow by its heading and the blips the game hands it, clipped at the edge. Read only.

`import { ShooterRadar } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/radar.tsx`).

| Prop | Type | What |
|---|---|---|
| `map?` | `Texture \| 0` | The map texture (og.load.texture); 0 draws the dark well alone. |
| `centerX?` | `number` | The map point under the player, 0 to 1 across. |
| `centerY?` | `number` | The map point under the player, 0 to 1 down. |
| `zoom?` | `number` | How much of the map the radar shows across, 0 to 1 (0.4). |
| `heading?` | `number` | The player's heading in degrees clockwise from north: north stays up and the player's arrow turns. |
| `blips?` | `ReadonlyArray<RadarBlip>` | What the radar shows. A blip off the radar's edge is clipped. Read only. |

Each `RadarBlip`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `kind?` | `string` | ally, foe or objective (each its own shape and colour). |
| `x` | `number` | On the map, 0 to 1 across. |
| `y` | `number` | On the map, 0 to 1 down. |
| `label?` | `string` | Names it for a reader. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `empty`, `edge`.

## 153. ShooterRound

The round on a plate lit along its top, its time left beside it, and the score with the player's side in its blue against the other's red. The game owns the clock and the score; read only. The plate wears the pack's cut and keeps no rim or urgent glow (a cut takes one fill).

`import { ShooterRound } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/round.tsx`).

| Prop | Type | What |
|---|---|---|
| `round?` | `number` | The round in play; 0 hides the word. |
| `clock?` | `string` | The round's time left as the game words it (1:42); empty hides it. |
| `ours?` | `number` | The player's side's score. |
| `theirs?` | `number` | The other side's score; negative hides the score. |
| `urgent?` | `boolean` | The game says time is short: the plate's lit edge and the clock turn to danger. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `urgent`, `clock`, `overflow`.

## 154. ShooterSquad

The squad at the screen's top corner: each squadmate's portrait in an octagon, the name and a state in words, a thin shield bar cut into cells over a thin health bar with its ghost. The game owns every value and state; read only. Approximations: the portrait's octagon cut is rounded, the out state's grey-out is opacity, the cells' mask is hairline dividers, and a phone's side-by-side reflow keeps the plate's stack.

`import { ShooterSquad } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/squad.tsx`).

| Prop | Type | What |
|---|---|---|
| `members?` | `ReadonlyArray<SquadMember>` | The player's squadmates in the game's order. The game owns every value and state; read only. |
| `segment?` | `number` | Shield per cell of each shield bar (25). |

Each `SquadMember`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls them. |
| `portrait?` | `Texture \| 0` | Their portrait, a texture (og.load.texture); 0 draws the person glyph. |
| `status?` | `string` | The state as the game words it (Reviving), empty for none. |
| `state?` | `string` | ok, down, reviving or out. |
| `shield?` | `number` | Shield now. |
| `shieldMax?` | `number` | Most shield; 0 hides the shield bar. |
| `health?` | `number` | Health now. |
| `healthMax?` | `number` | Most health. |
| `ghost?` | `number` | The health before the latest loss. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `down`, `empty`, `overflow`.

## 155. ShooterVitals

The player's vitals at the screen's bottom corner: the portrait in an octagon with a lit edge, the name and level, health as a figure over its most, a shield bar cut into cells over a fat health bar that leaves a warm ghost when it drops and turns to danger when low. The game owns every value; read only. Approximations: the portrait's octagon cut is rounded and the cells' mask is hairline dividers.

`import { ShooterVitals } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/vitals.tsx`).

| Prop | Type | What |
|---|---|---|
| `name?` | `string` | The player's name as the game words it; empty hides it. |
| `level?` | `number` | The player's level beside the name; 0 hides it. |
| `portrait?` | `Texture \| 0` | The player's portrait texture (og.load.texture); 0 shows the person glyph in its place. |
| `health` | `number` | Health now. |
| `healthMax?` | `number` | Most health. Default 100. |
| `ghost?` | `number` | Health before the latest loss: the bar keeps a warm ghost to it that drains; at or under health shows none. |
| `shield?` | `number` | Shield now, a cut bar over the health bar. |
| `shieldMax?` | `number` | Most shield; 0 hides the shield bar. |
| `segment?` | `number` | Shield per cell of the shield bar (25: four cells for 100). Default 25. |
| `low?` | `number` | The fraction of most health at or under which the health bar turns to danger (0.3). Default 0.3. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `full`, `low`, `critical`, `noshield`, `overflow`.

## 156. StrategyAlerts

Alerts and notices at the top left under the resources: one feed row each, newest first, lit from the edge it hangs on, a mark and an edge in danger, done or info, the game's words and the key that jumps to it; an older notice set back. The game owns which alerts stand and for how long; read only.

`import { StrategyAlerts } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/alerts.tsx`).

| Prop | Type | What |
|---|---|---|
| `alerts?` | `ReadonlyArray<{ id: string; kind: string; text: string; key: string; faded: boolean }>` | The alerts and notices, newest first, keyed rows of { id, kind, text, key, faded }: kind danger (under attack), done (research complete) or info, on data-kind, each with its own mark and edge; text the game's words; key the input that jumps to it as the game names it (Space), on a key cap, empty none, hidden under a touch screen; faded an older notice, set back (data-faded). |

Fixtures: `default`, `info`, `overflow`.

## 157. StrategyClock

The match clock at the top right: what it counts (Day 3), the time large and the game's speed on a tag, or Paused. The game owns the time and the speed; read only.

`import { StrategyClock } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/clock.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | What the clock counts as the game words it (Day 3, Match), before the time; empty hides it. |
| `time` | `string` | The time as the game words it (12:34). |
| `speed` | `string` | The game's speed as it words it (×1, ×2), on a tag after the time; empty hides it. |
| `paused` | `boolean` | The game is paused: the Paused word stands on the tag in place of the speed (data-paused). |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `paused`, `empty`, `overflow`.

## 158. StrategyCommands

The command card at the bottom right: abilities, orders and buildings as square tiles four to a row, each its picture with a lit edge, the hotkey in its corner, a cooldown as a dark sweep with the seconds left, a command that cannot be paid for greyed with its price on a red tag, a disabled one greyed, the order in hand lit; the info card of the command in focus over the grid (its name and key, what it does, its price, cooldown and when it is ready, or why it cannot be used). Reports command (the tile's key; a disabled or unaffordable tile sends nothing); the game owns every cost, cooldown and rule.

`import { StrategyCommands } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/commands.tsx`).

| Prop | Type | What |
|---|---|---|
| `commands?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; key: string; action: string; enabled: boolean; affordable: boolean; cost: string; costIcon: Texture \| 0; cooldown: number; time: string; active: boolean; detail: string; cooldownText: string; readyText: string; reason: string }>` | The command card in grid order, four to a row, keyed rows of { id, name, icon, glyph, key, action, enabled, affordable, cost, costIcon, cooldown, time, active, detail, cooldownText, readyText, reason }: name for a reader and the info card's title, empty draws an empty cell; icon the command's picture (og.load.texture), 0 draws glyph (an icon of the atlas); key the hotkey as the game names it (A), or action the input action whose bound glyph the cap shows (og.input.defineAction), both empty hide it, hidden under a touch screen; enabled false greys it and it sends nothing; affordable false greys it and shows cost on a red tag; cost and costIcon the price as the game words it (200) and the resource's picture; cooldown 0 to 1 still to run, a dark sweep over the art, with time its figure (18); active a toggle that is on or the order in hand, its edge lit; detail, cooldownText and readyText the info card's words (Cooldown 45 s, Ready in 18 s); reason why it cannot be used now (Need a Blacksmith), on the info card. |
| `focused` | `string` | The command whose info card stands over the grid (the one under the pointer, as the game decides); empty shows none. A phone shows none. |
| `onCommand?` | `(e: UiEvent, id: string) => void` | Command. |

Pack slots: `Button.plate`, `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `unaffordable`, `cooling`, `overflow`.

## 159. StrategyGroups

The control groups over the selection: one small plate each with its key cap and the units it holds, the group selected now lit; a quick pick (the idle villagers) may carry a picture, its count in gold while it wants the player. Reports pick (the group's key); the game owns the groups.

`import { StrategyGroups } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/groups.tsx`).

| Prop | Type | What |
|---|---|---|
| `groups?` | `ReadonlyArray<{ id: string; label: string; key: string; icon: Texture \| 0; count: number; active: boolean; alert: boolean }>` | The control groups and quick picks (the idle villagers, the army) in the game's order, keyed rows of { id, label, key, icon, count, active, alert }: label for a reader (Group 1, Idle villagers); key the group's key as the game names it (1), on a key cap; icon a picture (og.load.texture) before the count, 0 none; count what it holds; active the group selected now, lit (aria-pressed); alert one that wants the player (idle villagers waiting), its count and edge in gold (data-alert). |
| `onPick?` | `(e: UiEvent, id: string) => void` | Pick. |

Pack slots: `Button.plate`.

Fixtures: `default`, `idle`, `overflow`.

## 160. StrategyMinimap

The minimap at the bottom left: the whole map square in a lit-edged tile with two corners cut, your units and buildings, your allies' and your foes' as dots in their colours (a base larger, a resource a gold diamond), the camera's view as a white frame and each alert a red ring. The game gives every position on the map, 0 to 1; read only.

`import { StrategyMinimap } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/minimap.tsx`).

| Prop | Type | What |
|---|---|---|
| `map` | `Texture \| 0` | The whole map's picture (og.load.texture), drawn square; 0 draws the dark well alone. |
| `dots?` | `ReadonlyArray<{ id: string; kind: string; x: number; y: number; base: boolean; label: string }>` | What the minimap shows, keyed rows of { id, kind, x, y, base, label }: kind own, ally, foe or neutral (a resource, a relic; on data-kind, each its colour, a neutral a diamond); x and y on the map, 0 to 1; base marks a building or a town, drawn larger (data-base); label names it for a reader. |
| `viewX` | `number` | The camera's view on the map: its left edge, 0 to 1. |
| `viewY` | `number` | The camera's view: its top edge, 0 to 1. |
| `viewWidth` | `number` | The camera's view: its width, 0 to 1; 0 hides the frame. |
| `viewHeight` | `number` | The camera's view: its height, 0 to 1. |
| `pings?` | `ReadonlyArray<{ id: string; x: number; y: number; label: string }>` | Alerts on the map, keyed rows of { id, x, y, label }: a red ring round the place (x and y, 0 to 1); label names it for a reader. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `allies`, `empty`, `overflow`.

## 161. StrategyQueue

A production queue as one row: what produces in small capitals, the order in production as a larger tile with a dark sweep for what is still to run and its time left, the orders waiting as smaller tiles (a stack's count on a tag), the free places as empty tiles, the count over the limit, and Paused, Blocked with the reason, or Queue full as words. Reports cancel (a tile's key); the game owns costs, times, refunds and the limit.

`import { StrategyQueue } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/queue.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | What produces, as the game words it (Barracks queue); empty takes the kit's (Production). |
| `items?` | `ReadonlyArray<{ id: string; name: string; picture: Texture \| 0; icon: string; count: number; progress: number; timeText: string; state: string; reason: string }>` | The queue in the game's order, first in production, keyed rows of { id, name, picture, icon, count, progress, timeText, state, reason }: name for a reader; picture a texture (og.load.texture), 0 draws icon from the atlas; count a stack of the same order (×2, shown above 1); progress 0 to 1 done, a dark sweep over the first order's art for what is still to run; timeText its time left as the game words it (0:08), on the first order's tile, empty none; state building, waiting, paused or blocked (data-state; paused and blocked are also words after the tiles); reason why it is blocked (Need 3 supply), empty none. |
| `queued` | `number` | Orders queued, as the game counts them (stacks included), shown over limit on a tag. |
| `limit` | `number` | The most the queue holds: its free places are drawn as empty tiles after the orders; 0 hides the count and the free places. |
| `full` | `boolean` | The game says no more can be queued: the Queue full word on the count's tag and data-full on the frame. |
| `onCancel?` | `(e: UiEvent, id: string) => void` | Cancel the order (a slot's key). |

Pack slots: `Button.plate`, `Chip.frame`.

Fixtures: `default`, `blocked`, `empty`, `full`, `paused`, `overflow`.

## 162. StrategyResources

The resource bar at the top left: each stockpile's picture in a cut tile, its amount large (a ceiling after it, dim, as the population's), and its income under it, green while it grows and red while it drains, or the game's word (Near cap) in gold at a limit; the age or tier on a lit tag. The game owns every figure, rate and limit; read only.

`import { StrategyResources } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/resources.tsx`).

| Prop | Type | What |
|---|---|---|
| `resources?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; amount: string; cap: string; rate: string; trend: string; note: string; warn: boolean }>` | The stockpiles in the game's order, keyed rows of { id, name, icon, glyph, amount, cap, rate, trend, note, warn }: name for a reader (Gold); icon its picture (og.load.texture), 0 draws glyph (an icon of the atlas) instead; amount the store as the game words it (1,250); cap a ceiling shown after it (/ 50, the population's), empty none; rate the income as the game words it (+12/s), under the amount, inked by trend (gain, loss or none: data-trend); note a word in place of the rate (Near cap), empty none; warn marks a store at its limit in gold (data-warn). |
| `tier` | `string` | The age or tier as the game words it (Castle Age), on a lit tag after the stockpiles; empty hides it. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `tier`, `capped`, `empty`, `overflow`.

## 163. StrategySelection

The selection panel at the foot: the portrait in a tile with a lit edge, its health as a bar and in figures under it; the name large with how many are selected, its class line in small capitals, what it is doing and its stats on tags; and every unit selected as a card with its health, the focused one lit. Reports choose (a card's key: show that unit or subgroup). The game owns every figure and what the panel shows.

`import { StrategySelection } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/selection.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The selected unit's or building's name, or the focused subgroup's (Men-at-Arms); empty shows Nothing selected. |
| `count` | `number` | How many of it are selected (×12, shown above 1). |
| `kind` | `string` | unit or building (data-kind on the frame). |
| `line` | `string` | Its class line as the game words it (Heavy infantry · Veteran II), under the name; empty none. |
| `status` | `string` | What it is doing as the game words it (Holding, Under construction), on a tag before the stats; empty none. |
| `portrait` | `Texture \| 0` | Its portrait (og.load.texture) in a tile with a lit edge; 0 draws icon instead. |
| `icon` | `string` | An icon of the atlas drawn when portrait is 0. |
| `health` | `number` | Its health now (the subgroup's together), under the portrait as a bar and in figures. |
| `healthMax?` | `number` | Its whole health; 0 hides the bar and the figures. Default 1. |
| `stats?` | `ReadonlyArray<{ id: string; label: string; value: string }>` | Its stats on tags, keyed rows of { id, label, value }: label short as the game words it (ATK), value its figure (12). |
| `units?` | `ReadonlyArray<{ id: string; name: string; kind: string; portrait: Texture \| 0; icon: string; count: number; health: number; healthMax: number }>` | Each unit or subgroup selected, as cards in the game's order (shown when there are two or more; two rows, then they scroll), keyed rows of { id, name, kind, portrait, icon, count, health, healthMax }: portrait a texture, 0 draws icon from the atlas; count shown above 1 on a tag; health over healthMax a bar on the card, green, gold when hurt and red when low (data-tone). |
| `focused` | `string` | The card whose unit or subgroup the panel shows (its edge lit, aria-selected); empty none. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a card (its key: show that unit or subgroup). |

Pack slots: `Button.plate`, `Chip.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `single`, `empty`, `groups`, `dense`, `overflow`.

## 164. StrategyTower

A tower's inspector: its picture, name and level, its current stats with the value after the next upgrade where the game gives one, the upgrade paths (tier of tiers, the next tier's cost, a state word: affordable, too costly, locked with the reason, maxed) and Sell with what it returns. Reports upgrade (the path's key; only an affordable path is enabled) and sell. The game owns prices, refunds, stats and what is unlocked.

`import { StrategyTower } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/tower.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The tower's name; empty takes the kit's (Tower). |
| `icon` | `string` | An icon of the atlas for the tower, drawn when portrait is 0. |
| `portrait` | `Texture \| 0` | A texture of the tower (og.load.texture); 0 draws icon. |
| `level` | `number` | The tower's level as the game counts it; 0 hides it. |
| `stats?` | `ReadonlyArray<{ id: string; label: string; value: string; next: string }>` | Its current stats, keyed rows of { id, label, value, next }: value as the game words it (42); next the value after the upgrade in focus (51), shown after an arrow, empty none. |
| `paths?` | `ReadonlyArray<{ id: string; name: string; icon: string; detail: string; tier: number; tiers: number; cost: string; state: string; reason: string }>` | Upgrade paths, keyed rows of { id, name, icon, detail, tier, tiers, cost, state, reason }: tier of tiers bought; cost the next tier's price as the game words it (250 gold); state affordable, unaffordable, locked or maxed (data-state and a word; only affordable can be bought); reason why it is locked (Requires tier 3 on another path), empty none. |
| `sellText` | `string` | What selling returns, as the game words it (+180 gold), on the Sell button. |
| `canSell?` | `boolean` | The game allows selling now; false disables Sell. |
| `onUpgrade?` | `(e: UiEvent, id: string) => void` | Upgrade. |
| `onSell?` | `(e: UiEvent) => void` | Sell. |

Pack slots: `Button.plate`, `Button.plate.…`, `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `maxed`, `disabled`, `empty`, `overflow`.

## 165. StrategyWave

The wave readout at the top right: a plate whose edge is lit in danger while a wave comes or fights, its picture, Wave n (of N) and its name in red capitals, the countdown with a bar of the wait still to run, or Under attack or All waves cleared; the wave's enemies by kind under it, and Call early with what it pays. Reports callEarly; the game owns the timing, the wave's makeup and the reward.

`import { StrategyWave } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/wave.tsx`).

| Prop | Type | What |
|---|---|---|
| `wave` | `number` | The wave on now or coming, from 1. |
| `waves` | `number` | Waves in the level; 0 is an endless run (no total shown). |
| `name` | `string` | The wave's name as the game words it (Siege), after its number; empty none. |
| `icon` | `Texture \| 0` | The wave's picture (og.load.texture) in a cut tile before it; 0 hides it. |
| `phase` | `string` | countdown (the wave is coming), active (a wave is on) or done (every wave cleared), as the game says: data-phase; active and done stand as words in place of the countdown. |
| `next?` | `number` | Seconds to the wave while it is coming, shown as a countdown (0:42); negative hides it. Default -1. |
| `nextTotal` | `number` | The whole wait: the bar beside the countdown shows the share still to run; 0 draws no bar. |
| `warnAt?` | `number` | Seconds left when the countdown is near (data-near on the frame); 5 when left out. Default 5. |
| `finalWave` | `boolean` | The coming or current wave is the last: the Final wave badge (data-final). |
| `enemies?` | `ReadonlyArray<{ id: string; name: string; icon: string; count: number; boss: boolean; scouted: boolean }>` | The wave's enemies by kind, under the plate, keyed rows of { id, name, icon, count, boss, scouted }: icon an icon of the atlas; count shown as ×n; boss marks it with a Boss badge (data-boss); scouted false hides name and count behind Not yet scouted. Empty shows none. |
| `canCallEarly` | `boolean` | The game allows calling the wave now; false hides Call early. |
| `reward` | `string` | What calling early pays, as the game words it (+40 gold), on the button in gold; empty none. |
| `onCallEarly?` | `(e: UiEvent) => void` | Call the wave early. |

Pack slots: `Button.plate.primary`, `Chip.frame`, `Plate.frame`, `Row.frame`, `Row.frame.muted`.

Fixtures: `default`, `incoming`, `active`, `final`, `done`, `empty`, `overflow`.

## 166. Compass

A compass strip: the cardinal and intercardinal points and ticks sliding under the heading, the heading in degrees on a tab, and the points of interest with their distances. Read only.

`import { Compass } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/compass.tsx`).

| Prop | Type | What |
|---|---|---|
| `heading?` | `number` | The player's heading in radians clockwise from north, 0 to 2π. |
| `pois?` | `ReadonlyArray<CompassPoi>` | Its points. |
| `units?` | `string` | The distance's unit (m). |

Each `CompassPoi`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `bearing` | `number` | In radians clockwise from north (0 to 2π). |
| `distance?` | `number` | In m (negative: none). |
| `kind?` | `string` | objective, ping, enemy, ally, loot or poi. |
| `icon?` | `string` | An icon of the atlas; empty draws a diamond. |
| `label?` | `string` | For a reader. |

Fixtures: `default`.

## 167. InteractionPrompt

What the player can do with what is in front of them: a key cap beside a glass plate, the verb in condensed capitals and the object under it; a hold's ring round the cap. It rises in when in range. The game decides when it shows and what the input does; read only. The old SVG hold ring becomes a nine-ring with the hold's share as its wipe (the 200 ms drain on release is dropped); a nine-cut plate takes no rim (it paints as a rectangle around the cut), so the glass's rim is dropped. The key cap's action glyph has no TSX surface: the cap shows the game's key when it gives one and keeps the action's name as a state otherwise, and the old anchor's avoidance and max distance are dropped too. It keeps its place over the plates and labels that stack (the anchor's declutter, priority 1).

`import { InteractionPrompt } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/interaction-prompt.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | What it is over, by its bits (0: use at). |
| `at?` | `string` | A world point it sits over without an entity (x y z). |
| `height?` | `number` | World units above the point. |
| `action?` | `string` | The action whose glyph the key cap shows, its binding on the device used last (interact); empty shows key. |
| `key?` | `string` | The key as the game names it (E), shown when action is empty. |
| `pad?` | `boolean` | A gamepad's face button: a round cap. |
| `verb` | `string` | What it does, in condensed capitals (Open, Talk, Revive); it always fits whole. |
| `name?` | `string` | What it is under the verb (Gilded Chest); empty is none. |
| `pressed?` | `boolean` | The key is down: the cap sinks 2 px and lights. |
| `hold?` | `number` | A hold's progress 0 to 1: a white ring round the cap on a dark disc (1 is done: the cap lights); negative draws none. Released, the game sets 0 and the ring drains in 200 ms. |
| `disabled?` | `boolean` | It cannot be used now: the cap greys and the reason shows in red with a lock. |
| `reason?` | `string` | Why it cannot (Needs the Warden's key). |
| `maxDistance?` | `number` | Hidden past this many world units (6). |

Fixtures: `default`, `pressed`, `hold`, `done`, `disabled`, `gamepad`, `action`, `overflow`.

## 168. LootLabel

An item on the ground: over it a dark chip with the tier's strip on its leading edge and the name in the tier's ink (a stack's count after it); epic and legendary raise a beam. Reports pick (the chip); the game owns what is picked up. The old ground pool (the tier's glow where the item lies, a `<ground>` plane) has no TSX surface and is dropped, as are the old anchor's avoidance and max distance; labels stack clear of each other and fade behind drawn geometry (the anchor's declutter and occlusion). The hovered chip's pick-up key (the action's bound glyph) has no TSX surface and is dropped; the chip keeps the action's name as a state. A long name wraps without breaking.

`import { LootLabel } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/loot-label.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | The item, by its bits (0: use at). |
| `at?` | `string` | A world point it lies at without an entity (x y z). |
| `height?` | `number` | World units above it the label stands. |
| `name?` | `string` | The item's name, in its tier's ink; a long one scrolls on one line. |
| `count?` | `number` | A stack's size, shown as ×n from 2. |
| `rarity?` | `string` | Its tier: common (empty), uncommon, rare, epic or legendary: the strip on the chip's leading edge, the name's ink and the glow on the ground; epic and legendary raise a beam, a legendary wears a gold wash and rim and shimmers every 2.4 s. |
| `hovered?` | `boolean` | Aimed at or under the pointer (the chip takes :hover too): it lifts 1.06 times, its rim goes all round and the pick-up key shows. |
| `action?` | `string` | The input action whose bound key shows on a hovered chip (og.input.defineAction); empty shows none. |
| `more?` | `number` | A chip standing for this many more items the game folded into one (+n), with a dot per tier in tiers; 0 is an item's own chip. |
| `tiers?` | `ReadonlyArray<LootTier>` | The folded items' tiers, keyed rows of { id, rarity }, a dot each on a +n chip. |
| `onPick?` | `(e: UiEvent) => void` | The chip is pressed: the game owns what is picked up. |

Each `LootTier`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `rarity?` | `string` |  |

Pack slots: `Chip.frame`.

Fixtures: `default`, `common`, `stack`, `uncommon`, `rare`, `legendary`, `hovered`, `more`, `overflow`.

## 169. MinimapView

A minimap: the map texture zoomed round the player's point, player-up or north-up, round or square, with its markers and the zone's name under it. It reports onPick for a tap on the glass.

`import { MinimapView } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/minimap-view.tsx`).

| Prop | Type | What |
|---|---|---|
| `texture?` | `Texture \| 0` | The map texture (og.load.texture); 0 draws the well with a grid. |
| `centerX?` | `number` | The map point at its centre (the player's), 0 to 1 across. |
| `centerY?` | `number` | The map point at its centre, 0 to 1 down. |
| `zoom?` | `number` | How much of the map it shows across, 0 to 1 (0.3). Default 0.3. |
| `heading?` | `number` | The player's heading in degrees clockwise from north. |
| `rotate?` | `boolean` | Player-up: the map turns with the heading; false keeps north up and turns the player's arrow. |
| `square?` | `boolean` | A square frame; false is round. |
| `markers?` | `ReadonlyArray<MinimapViewMarker>` | Its markers. |
| `zone?` | `string` | The zone's name under it; empty is none. |
| `onPick?` | `(e: UiEvent) => void` | Pick: a tap on the glass. |

Each `MinimapViewMarker`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `kind?` | `string` | ally, enemy, objective, ping, loot or poi. |
| `x` | `number` | On the map, 0 to 1 across. |
| `y` | `number` | On the map, 0 to 1 down. |
| `label?` | `string` | For a reader. |

Pack slots: `Plate.frame`.

Fixtures: `default`.

## 170. Minimap

A square minimap: a region of the map texture (or a grid without one), markers over it and a zone name under it on a plate. Read only.

`import { Minimap } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/minimap.tsx`).

| Prop | Type | What |
|---|---|---|
| `texture?` | `Texture \| 0` | The map texture (og.load.texture); 0 draws a grid over the stage colour. |
| `u0?` | `number` | The region of the map shown: its left edge, 0 to 1. |
| `v0?` | `number` | Its top edge, 0 to 1 (v down). |
| `u1?` | `number` | Its right edge (1: the whole width). Default 1. |
| `v1?` | `number` | Its bottom edge (1: the whole height). Default 1. |
| `size?` | `number` | Its side in px (160). |
| `markers?` | `ReadonlyArray<MinimapMarker>` | Markers over the map. |
| `label?` | `string` | A zone name under it; empty is none. |

Each `MinimapMarker`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `x` | `number` | Across the shown region, 0 to 1. |
| `y` | `number` | Down the shown region, 0 to 1. |
| `tone?` | `string` | neutral, info, success, warn, danger or accent. |
| `player?` | `boolean` | The player's: the primary, larger and ringed. |

Pack slots: `Plate.frame`.

Fixtures: `default`.

## 171. Nameplate

A unit's plate over its head: the level chip and the name in its reaction's ink, its title in small capitals, its health as a bar cut in quarters (a cream ghost of a hit, the shield after the fill) and the cast row (the spell and its time over a thin bar), all on a HUD plate (compact, the bar alone, has none). Read only; the game owns every figure and state. The old style's lean (skewX), the wings' and the target brackets' clip-path arrows, the interrupted cast's hatch and the anchor's max distance have no TSX surface and are dropped; it stacks over the plates it overlaps and fades behind drawn geometry (the anchor's declutter and occlusion); the quarters are drawn as three notch marks, the plate does not scale when targeted, and the plate's room cap (a viewport measure) stands as max-w-full.

`import { Nameplate } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/nameplate.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | The unit it sits over, by its bits (0: use at). |
| `at?` | `string` | A world point it sits over without an entity (x y z). |
| `height?` | `number` | World units above the point (its head). |
| `name` | `string` | The name, in the reaction's ink; a long one scrolls on one line. |
| `title?` | `string` | A line under it in small capitals (a role, a guild); empty is none. |
| `level?` | `string` | Its level on a chip as the game words it (32); empty is none, a boss's empty level draws a skull. |
| `reaction?` | `string` | Its colour: neutral (empty), friendly, hostile or player. |
| `health?` | `number` | Health 0 to 1 for a skewed bar cut in quarters under the name; negative draws none (a friendly NPC that cannot be hurt). When it drops, a cream ghost of what was lost holds 400 ms and drains in 300 ms. At 0.25 or less the rim turns danger and breathes, and the percent shows. |
| `shield?` | `number` | An absorb after the health, 0 to 1, in cyan. |
| `castName?` | `string` | The spell it is casting; empty is nothing. A boss or a caster sits above the interaction prompt when plates overlap. |
| `castProgress?` | `number` | How far the cast is, 0 to 1 (the game's progress, read every tick). |
| `castTime?` | `string` | The cast's time left as the game words it (1.2); empty is none. |
| `castState?` | `string` | The cast: casting (empty), locked (it can't be interrupted: steel grey with a shield mark) or interrupted (a red hatch and the word; the game shows it 600 ms, then clears castName). |
| `icon?` | `string` | A class, quest or role icon before the name; empty is none. |
| `elite?` | `boolean` | An elite: a gold level chip, gold wings at the bar's ends and a gold rim. |
| `boss?` | `boolean` | A boss: as an elite, with its name in tracked capitals, a longer bar and the percent. |
| `targeted?` | `boolean` | The player's target: 1.14 times larger, its name and rim white, white brackets that close in 120 ms; drawn over the rest. |
| `dead?` | `boolean` | Dead: the name greys and the bars go. |
| `lootable?` | `boolean` | Dead with loot: Lootable in gold under the name. |
| `compact?` | `boolean` | Far or crowded: only the bar (no name, title or cast). |
| `maxDistance?` | `number` | Hidden past this many world units (80). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `friendly`, `neutral`, `hostile`, `player`, `boss`, `targeted`, `low`, `casting`, `locked`, `interrupted`, `dead`, `compact`, `overflow`.

## 172. OverheadBars

A health bar over every unit in the side's colour, a thin resource line under it. Read only. The old anchors were GPU mass elements (one instanced draw for a battle's thousands, a hit leaving a cream ghost): each bar is now a text-free pair of nodes and the ghost is dropped.

`import { OverheadBars } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/overhead-bars.tsx`).

| Prop | Type | What |
|---|---|---|
| `bars?` | `ReadonlyArray<OverheadBar>` | The units' bars, keyed rows of { id, entity, at, value, max, side, resource, resourceMax }: entity the unit by its bits (0: at, a world point x y z); value its health out of max; side whose it is, own (green), ally (blue), enemy (red) or neutral (yellow); resource out of resourceMax a thin line under the health (mana, energy), resourceMax 0 draws none. Thousands are fine: each bar is a GPU mass element of a fixed size (never under 30 × 3 px) that keeps the ghost of a hit. |
| `height?` | `number` | World units over each unit's point (its head). |

Each `OverheadBar`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `entity` | `number` | The unit, by its bits (0: at, a world point x y z). |
| `at` | `string` | A world point x y z without an entity. |
| `value` | `number` | Its health out of max. |
| `max` | `number` |  |
| `side?` | `string` | Whose it is: own (green), ally (blue), enemy (red) or neutral (yellow). |
| `resource?` | `number` | A thin resource line under the health (mana, energy). |
| `resourceMax?` | `number` |  |

Fixtures: `default`, `sides`, `low`.

## 173. SpeechBubble

A line spoken over a head: the HUD's glass with a cut corner, the speaker in their colour, the line wrapping at 27 characters and a thin line of the time left. It pops from its tail. Read only; the game decides when it shows and when it goes. The old tail (a clip-path triangle) has no TSX surface: it stands in as a small diamond under the box. A nine-cut box takes no rim or outer shadow (they paint as rectangles around the cut), so the glass's rim, the drop shadow and a shout's warm rim are dropped; a shout keeps its bolder line. The 27 character measure stands as a fixed 12.6rem (the ch unit and the viewport room caps have no class surface). Bubbles stack clear of each other (the anchor's declutter).

`import { SpeechBubble } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/speech-bubble.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | Who speaks, by its bits (0: use at). |
| `at?` | `string` | A world point it speaks from without an entity (x y z). |
| `height?` | `number` | World units above the point (over the head). |
| `speaker?` | `string` | Who speaks, over the line in their reaction's ink; empty is none. |
| `reaction?` | `string` | The speaker's colour: friendly (empty), hostile, neutral or player. |
| `text` | `string` | The line; it wraps at 27 characters. |
| `shout?` | `boolean` | A shout: a warm rim and a bolder line. |
| `left?` | `number` | How much of the line's time is left, 0 to 1, a thin line along the bubble's foot; negative draws none. The game times the line out and removes the bubble. |

Fixtures: `default`, `short`, `shout`, `nameless`, `overflow`.

## 174. Waypoint

A place to go: a lit diamond with a dark core in the kind's colour, its name on a plate under it and the distance on a dark tag. Off screen, a chevron points the way. Read only; the game owns the distance and what counts as near and far. Off screen or behind the camera it pins 24 px inside the screen's edge toward its point (the anchor's clamp, which marks the side as `data-edge-*`). The old anchor's decluttering, avoidance and chevron have no TSX surface: the edge prop leans the label toward the side the game names, and the chevron it drew is dropped.

`import { Waypoint } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/waypoint.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | What it marks, by its bits (0: use at). |
| `at?` | `string` | A world point it marks without an entity (x y z). |
| `height?` | `number` | World units above the point. |
| `kind?` | `string` | What it is: objective (empty, gold), ping (cyan, a ripple), danger (red), ally (green) or loot (violet). |
| `label?` | `string` | Its name on a HUD plate under the diamond (The Ninefold Gate); empty is none. It drops when far. |
| `distance?` | `string` | How far it is as the game words it (412), on a dark tag; empty is none. |
| `units?` | `string` | The distance's unit, smaller after it (m). |
| `range?` | `string` | near (a soft halo), mid (empty) or far (the label drops, the distance stays), as the game judges it. |
| `edge?` | `string` | The side of the screen it is off, as the game reads it (left or right): a chevron toward it before the pin; empty is on screen. The anchor holds it inside the screen's edge. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `near`, `far`, `ping`, `danger`, `ally`, `loot`, `offscreen`, `offscreenRight`, `overflow`.

## 175. Art

A slot of generated art: an image that covers its box, cropped about its centre.

`import { Art } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/art.tsx`).

| Prop | Type | What |
|---|---|---|
| `kind` | `string` | The slot's kind of generated art (keyArt, logo, levelThumb, bust …); the art tools fill slots by kind. |
| `src` | `Texture \| 0` | The loaded texture; 0 is none yet. |
| `class?` | `string \| undefined` |  |
| `label?` | `string \| undefined` |  |

## 176. Button

A button: a row of its content, centred, at least a touch target tall, in the button type. A pack restyles it through the "Button.base" variants.

`import { Button } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/button.tsx`).

| Prop | Type | What |
|---|---|---|
| `variant?` | `"primary" \| "danger" \| "ghost" \| "quiet" \| undefined` | The look: a plain button, the primary action, a destructive one, a ghost (a rim on the plain surface) or quiet (the plain surface, no rim). |
| `small?` | `boolean \| undefined` | The smaller padding a toolbar or a row uses. |
| `pressed?` | `boolean \| undefined` | A toggle that is on (aria-pressed / aria-checked). |
| `disabled?` | `boolean \| undefined` |  |
| `label?` | `string \| undefined` | What a screen reader calls it when its content is not words. |
| `class?` | `string \| undefined` | Classes of the document that uses it; they come after its own, so they win. |
| `tabIndex?` | `number \| undefined` |  |
| `autofocus?` | `boolean \| undefined` | It takes the focus once it shows (a menu's first choice, a dialog's safe one). |
| `hold?` | `number \| undefined` | Seconds a press or confirm must be held before it presses (hold-to-confirm). |
| `onPress?` | `((e: UiEvent) => void) \| undefined` |  |
| `onHold?` | `((e: UiEvent) => void) \| undefined` | The hold's progress, 0 to 1, in `e.value` ("0" when let go early). |
| `onLongPress?` | `((e: UiEvent) => void) \| undefined` |  |
| `onAction?` | `Record<string, (e: UiEvent) => void> \| undefined` |  |
| `children?` | `any` |  |

Fixtures: `default`, `primary`, `danger`, `ghost`, `quiet`, `pressed`, `disabled`.

## 177. Counter

A number that counts to its value as it changes. The count itself is the motion head's (a value-change transition on the text); until it lands the number shows its value at once.

`import { Counter } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/counter.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The number it shows. |
| `format?` | `"grouped" \| "compact" \| "percent" \| undefined` | How it writes the number: grouped (1,234, the default), compact (1.2K) or percent (0 to 1). |
| `decimals?` | `number \| undefined` | Digits after the point. |
| `from?` | `number \| string \| undefined` | Where its first count starts (a score counting up as its screen opens); the count is the motion head's. |
| `duration?` | `number \| string \| undefined` | How long a count takes, in ms; the motion head's. |
| `class?` | `string \| undefined` |  |

## 178. Meter

A track whose fill wears its value's region: good, middling or poor.

`import { Meter } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/meter.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` |  |
| `min?` | `number \| undefined` | The range; default 0 to 1. |
| `max?` | `number \| undefined` |  |
| `region?` | `"optimum" \| "suboptimum" \| "poor" \| undefined` | Where the value lies against the meter's optimum, as the game judges it: the fill wears its colour. |
| `class?` | `string \| undefined` |  |
| `label?` | `string \| undefined` |  |

Fixtures: `optimum`, `suboptimum`, `poor`.

## 179. Progress

A track filled to its value: a capsule well and its fill, which eases as the value moves.

`import { Progress } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/progress.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | Done, 0 to 1. |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `label?` | `string \| undefined` |  |

Fixtures: `empty`, `third`, `full`.

## 180. Banner

A call across the scene with no plate: the headline in large HUD type (white, one stroke under the fill and a soft shadow), a lit gold rule under it and the line beneath in quieter HUD type. It rises in with the reward sound; under reduced motion it fades in place.

`import { Banner } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/banner.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | Its headline (Level up!). |
| `subtitle` | `string` | A line under it; empty: none. |

Pack slots: `Plate.frame`.

Fixtures: `default`.

## 181. Calendar

A HUD plate: the month between cut arrow keys, the days dark recesses, today lit in the accent and a marked day carrying a gold diamond.

`import { Calendar } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/calendar.tsx`).

| Prop | Type | What |
|---|---|---|
| `month` | `string` | The month shown (March 2027). |
| `weekdays` | `ReadonlyArray<{ id: string; label: string }>` | The seven weekday headings, a keyed collection of { id, label }. |
| `days` | `ReadonlyArray<{ id: string; label: string; today: boolean; outside: boolean; marked: boolean }>` | The cells, a keyed collection of { id, label, outside, today, marked } from the week's first day (outside: another month's). |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onPick?` | `(e: UiEvent, key?: string) => void` | The pick event, with the row's key where it comes from a row. |
| `onPrevious?` | `(e: UiEvent, key?: string) => void` | The previous event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`.

Fixtures: `default`.

Actions: `previous` (Previous month), `next` (Next month), `pick` (Choose day).

## 182. CalibrationScreen

Calibration in three steps: brightness (gamma), the audio offset measured from taps, and the controller's dead zone, each with its slider and test. The game owns the values and writes them back from the events.

`import { CalibrationScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/calibration.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `step` | `string` | brightness, offset or controller. |
| `brightness` | `number` | The gamma's adjustment, 0 to 1 (two-way). |
| `offset` | `number` | The audio offset in ms, -200 to 200 (two-way). |
| `taps` | `number` | Taps measured so far. |
| `deadZone` | `number` | The sticks' dead zone, 0 to 0.5 (two-way). |
| `onBrightness?` | `(e: UiEvent, value: number) => void` | The brightness slider stepped (onBrightness), the value clamped to 0 to 1. |
| `onOffset?` | `(e: UiEvent, value: number) => void` | The offset slider stepped (onOffset), the value clamped to -200 to 200 ms. |
| `onDeadZone?` | `(e: UiEvent, value: number) => void` | The dead-zone slider stepped (onDeadZone), the value clamped to 0 to 0.5. |
| `test?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the test snippet). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onDone?` | `(e: UiEvent, key?: string) => void` | The done event, with the row's key where it comes from a row. |
| `onMeasure?` | `(e: UiEvent, key?: string) => void` | The measure event, with the row's key where it comes from a row. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onTap?` | `(e: UiEvent, key?: string) => void` | The tap event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Next), `back` (Back).

## 183. Card

A card: a picture across its top, a heading, a line or two under it, a badge at its top end and the actions the caller supplies. It reports onOpen.

`import { Card } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/card.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | Its heading. |
| `body` | `string` | A line or two under the heading; empty: none. |
| `image` | `Texture \| 0` | A picture the game registered across its top; 0: none. |
| `badge` | `string` | A badge at its top end (New, -20%); empty: none. |
| `actions?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the actions snippet). |
| `onOpen?` | `(e: UiEvent, key?: string) => void` | The open event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`.

Fixtures: `default`.

Actions: `open` (Open).

## 184. Confirm

A confirmation dialog on its own panel: the question as its heading, what follows from it, and Cancel and Confirm (the danger variant when confirming destroys something). The game closes it.

`import { Confirm } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/confirm.tsx`).

| Prop | Type | What |
|---|---|---|
| `open?` | `boolean` | Shown; the game writes it false to close (onCancel does on a dismissal). |
| `title` | `string` | Its question, the dialog's heading and accessible name. |
| `body` | `string` | What follows from it; empty: none. |
| `confirm` | `string` | The confirming button's words (Quit, Delete). |
| `cancel` | `string` | The cancelling button's words. |
| `danger?` | `boolean` | Confirming destroys something: the confirm button is the danger variant. |
| `onCancel?` | `(e: UiEvent, key?: string) => void` | The cancel event, with the row's key where it comes from a row. |
| `onConfirm?` | `(e: UiEvent, key?: string) => void` | The confirm event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Modal.frame`, `Panel.frame`.

Fixtures: `default`, `overflow`.

Actions: `confirm` (Confirm), `cancel` (Cancel).

## 185. CreatorToolsScreen

The creator's tools on the HUD's smoked glass: the four tools a segmented switch lit in the accent, the panels plates with tracked capital titles. On a phone the panel tabs are one strip that scrolls sideways, each a finger's target; a focus move scrolls its tab into view.

`import { CreatorToolsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/creator-tools.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `tool` | `string` | select, move, rotate or scale. |
| `playing?` | `boolean` | The world is playing. |
| `selected` | `string` | The selected object's name (empty: none). |
| `panel` | `string` | On a phone, the open panel: hierarchy, inspector or catalogue. |
| `dirty?` | `boolean` | Unsaved changes. |
| `cataloguePanelCatalogue?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the cataloguePanelCatalogue snippet). |
| `scrollInspector?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the scrollInspector snippet). |
| `treePanelHierarchy?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the treePanelHierarchy snippet). |
| `view?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the view snippet). |
| `onPlay?` | `(e: UiEvent, key?: string) => void` | The play event, with the row's key where it comes from a row. |
| `onRedo?` | `(e: UiEvent, key?: string) => void` | The redo event, with the row's key where it comes from a row. |
| `onSave?` | `(e: UiEvent, key?: string) => void` | The save event, with the row's key where it comes from a row. |
| `onUndo?` | `(e: UiEvent, key?: string) => void` | The undo event, with the row's key where it comes from a row. |
| `onTool?` | `(e: UiEvent, key?: string) => void` | A tool pressed, with its key (select, move, rotate or scale): the game sets `tool`. |
| `onPanel?` | `(e: UiEvent, key?: string) => void` | A panel tab pressed on a phone, with its key (hierarchy, inspector or catalogue): the game sets `panel`. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `back` (Exit).

## 186. CreditsScreen

The credits: the sections' titles and names rolling over the key art's veil, the logo at the head, and the end and licences the caller supplies. The game moves the roll (offset); it reports onBack and onFaster.

`import { CreditsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/credits.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `sections` | `ReadonlyArray<{ id: string; names: ReadonlyArray<{ id: string; name: string; role: string }>; title: string }>` | The sections: { title, names: [{ name, role }] }. |
| `offset` | `number` | How far the roll has run, CSS px. |
| `keyArt` | `Texture \| 0` | The key art's texture under the roll's veil (og.load.texture; 0: the style's dark ground). |
| `logo` | `Texture \| 0` | The game's logo texture at the head of the roll (og.load.texture; 0: the logo slot's placeholder). |
| `end?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the end snippet). |
| `licences?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the licences snippet). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onFaster?` | `(e: UiEvent, key?: string) => void` | The faster event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Faster), `back` (Back).

## 187. DataList

A HUD plate of rows: a name, its detail and its figure.

`import { DataList } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/data-list.tsx`).

| Prop | Type | What |
|---|---|---|
| `rows?` | `ReadonlyArray<{ id: string; title: string; detail: string; value: string }>` | The rows, a keyed collection of { id, title, detail, value }. |
| `loading?` | `boolean` | Rows are on their way: a busy line instead of the empty one. |
| `empty` | `string` | What an empty list says. |
| `busy` | `string` | What a loading list says. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`.

Fixtures: `default`.

Actions: `choose` (Open).

## 188. FlowChaptersScreen

A chapter select: the story's chapters in order with their thumbnails, the selected one shown, and Back. It reports onSelect with the chapter's key.

`import { FlowChaptersScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-chapters.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `chapters` | `ReadonlyArray<{ id: string; title: string; thumb: Texture \| 0; locked: boolean }>` | The chapters, in order (og.flow.load's chapters): keyed rows of { id, title, thumb (texture: the chapter's save's poster), locked (not reached yet: no save to play from) }. |
| `selected` | `string` | The chapter selected now (its key). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Screen.frame`.

Fixtures: `default`.

Actions: `select` (Play the chapter), `back` (Back).

## 189. FlowChoices

An interactive film's choice window over its footage (flow guide §4): a thin bar over the choices drains with the time left; the choices are cut plates docked at the foot, the pending one ringed in the accent, one that cannot be taken dim, a quick-time one with its progress under its words. Hotspots are lit frames at the fractions of the picture the flow gives, so they land on it when the game draws the picture to fill the surface. Past the lock the choices stay and take nothing.

`import { FlowChoices } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-choices.tsx`).

| Prop | Type | What |
|---|---|---|
| `view` | `FlowView \| null` | What the flow shows now (og.flow.view's value, set each tick); null or a view without a window hides the screen. |
| `onChoose?` | `((e: UiEvent, index: number) => void) \| undefined` | A choice was taken, by its row or its hotspot: pass `index` (ChoiceView.index) to og.flow.choose. |

Pack slots: `Screen.frame`.

Fixtures: `default`, `locked`, `closed`.

## 190. FlowChartScreen

The story so far over the dimmed flow: a band with Back and the title, the chart of flow nodes (seen ones done, the one playing current, the rest locked), and the picked node on a plate lit in the accent, its words beside Play from here, which drops under them where the plate is narrow.

`import { FlowChartScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-chart.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `nodes` | `ReadonlyArray<{ id: string; x: number; y: number; label: string; icon: string; state: string; shape: string; team: string }>` | og.flow.chart's nodes as NodeGraph's: keyed rows of { id (the node), x, y, label, icon, state, shape, team }. state: current (where the flow is), done (a segment seen), locked (not seen); an ending's shape diamond, a chapter's square, the rest circle. |
| `links` | `ReadonlyArray<{ id: string; points: readonly number[]; active: boolean }>` | og.flow.chart's links as NodeGraph's: keyed rows of { id, points, active }, points the link's FlowChartLink.points, active a choice the player has taken. |
| `view` | `readonly [number, number, number, number]` | The chart's extent, [0, 0, chart.width, chart.height] from og.flow.chart. |
| `zoom` | `number` | NodeGraph's zoom, kept by the game from its zoom and pinch actions. |
| `panX` | `number` | NodeGraph's pan across. |
| `panY` | `number` | NodeGraph's pan down. |
| `selected` | `string` | The node picked (its id); empty is none. |
| `detail` | `string` | What the picked node is, as the game words it (its chapter, or Not seen yet); empty hides the panel. |
| `canPlay` | `boolean` | The picked node can be played from (a chapter the player has saved at). |
| `onPick?` | `(e: UiEvent, id: string) => void` | A node was picked: its id. |
| `onZoom?` | `(e: UiEvent, delta: number) => void` | The wheel over the chart: `delta` its notches, up negative (the game keeps zoom). |
| `onPan?` | `(e: UiEvent, dx: number, dy: number) => void` | The chart was dragged by `dx`, `dy` px (the game keeps the pan). |
| `onPlay?` | `(e: UiEvent) => void` | Play from the picked node. |
| `onBack?` | `(e: UiEvent) => void` |  |

Pack slots: `Screen.frame`.

Fixtures: `default`, `unseen`, `none`.

Actions: `pick` (Pick a node), `play` (Play from here), `back` (Back).

## 191. FlowGalleryScreen

An endings gallery: the endings reached and locked, with their thumbnails, the selected one shown, and Back. It reports onSelect with the ending's key.

`import { FlowGalleryScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-gallery.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `endings` | `ReadonlyArray<{ id: string; title: string; thumb: Texture \| 0; locked: boolean }>` | The endings, in order (og.flow.load's endings): keyed rows of { id, title, thumb (texture: the ending segment's poster), locked (not reached in any playthrough: its title stays hidden) }. |
| `selected` | `string` | The ending selected now (its key). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Screen.frame`.

Fixtures: `default`.

Actions: `select` (Watch the ending again), `back` (Back).

## 192. FlowPicture

The footage filling the surface, cropped to cover it, over a deep shade (so a live scene under the flow shows only where the footage has none): the previous picture under it while a dissolve runs. FlowChoices' hotspot fractions land on it because it fills the same surface.

`import { FlowPicture } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-screen.tsx`).

| Prop | Type | What |
|---|---|---|
| `picture` | `Texture \| 0` | The footage's picture: FlowView.texture, or a cutscene's og.VideoPlayer.texture (0 before its first frame). |
| `previous?` | `Texture \| 0 \| undefined` | The picture dissolving out under it (FlowView.previous; 0: none). |
| `mix?` | `number \| undefined` | How far the dissolve is: the picture's opacity over the previous one (FlowView.mix). |
| `onSkip?` | `((e: UiEvent) => void) \| undefined` | Given, a Skip button at the top corner calls it: a cutscene the player has seen (og.flow.skip, or the game's own). |

Pack slots: `Button.plate`, `Screen.frame`.

Fixtures: `default`, `skip`.

## 193. FlowScreen

A flow as the player sees it, what the engine mounts for project.ogt's `flow`: its picture with its dissolve (a segment with no footage leaves the live scene showing), the captions its timelines say, and its choice window with hotspots and quick-time progress, each press passed to og.flow.choose. Read from og.flow.view each tick; nothing of it is world state.

`import { FlowScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-screen.tsx`).

| Prop | Type | What |
|---|---|---|
| `flow?` | `Entity \| undefined` | The flow it draws; left out, og.flow.current's (project.ogt's `flow`). |

Pack slots: `Button.plate`, `Screen.frame`.

Fixtures: `default`, `skip`.

## 194. GameOverScreen

Game over: the defeat's line, the run's score and the best, an optional revive offer with its price and the seconds it stays, and Retry, Checkpoint and Quit. The game decides the offer and what each choice costs.

`import { GameOverScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/game-over.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `title` | `string` | The defeat's line. |
| `score` | `number` | The run's score. |
| `best` | `number` | The best score. |
| `offer?` | `boolean` | A revive is offered. |
| `price` | `string` | The revive's price, as text (50 gems, Watch an ad). |
| `left` | `number` | Seconds the offer stays. |
| `checkpoint?` | `boolean` | A checkpoint exists. |
| `offerArt?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the offerArt snippet). |
| `stats?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the stats snippet). |
| `onCheckpoint?` | `((e: UiEvent, key?: string) => void) \| undefined` | The checkpoint event, with the row's key where it comes from a row. |
| `onDecline?` | `((e: UiEvent, key?: string) => void) \| undefined` | The decline event, with the row's key where it comes from a row. |
| `onQuit?` | `((e: UiEvent, key?: string) => void) \| undefined` | The quit event, with the row's key where it comes from a row. |
| `onRetry?` | `((e: UiEvent, key?: string) => void) \| undefined` | The retry event, with the row's key where it comes from a row. |
| `onRevive?` | `((e: UiEvent, key?: string) => void) \| undefined` | The revive event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Retry).

## 195. HudEditorScreen

The HUD editor's frame: a hint at the top until the player moves something, over the editor the caller supplies.

`import { HudEditorScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/hud-editor.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `showHint?` | `boolean` | The hint at the top shows (false once the player has moved something). |
| `editorEditor?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the editorEditor snippet). |

Pack slots: `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `back` (Done).

## 196. KeyValues

A table of key and value rows, each with an optional change (a stat comparison). Read only.

`import { KeyValues } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/key-values.tsx`).

| Prop | Type | What |
|---|---|---|
| `rows` | `ReadonlyArray<{ id: string; key: string; value: string; delta: string }>` | The rows, a keyed collection of { id, key, value, delta } (keyValueRows; statCompare when delta is set: + better, - worse). |

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`.

## 197. LeaderboardScreen

A leaderboard: global, friends and league tabs, the period, the top rows with the player's own marked, the player's row pinned, and the league's promotion and demotion lines. The game owns the rows.

`import { LeaderboardScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/leaderboard.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `tab` | `string` | global, friends or league. |
| `period` | `string` | The period, as text (This week). |
| `rows` | `ReadonlyArray<{ id: string; name: string; rank: number; score: number; texture: Texture \| 0; you: boolean }>` | The top rows: { rank, name, score, you (bool), texture }. |
| `selected` | `string` | The stable collection key selected now. |
| `me` | `Record<string, any>` | The player's row: { rank, name, score }. |
| `league` | `Record<string, any>` | The league: { name, promote (rank), demote (rank), ends (seconds) }. |
| `board?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the board snippet). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onPeriod?` | `(e: UiEvent, key?: string) => void` | The period event, with the row's key where it comes from a row. |
| `onRow?` | `(e: UiEvent, key?: string) => void` | The row event, with the row's key where it comes from a row. |
| `onTab?` | `(e: UiEvent, key?: string) => void` | A tab pressed, with its key (global, friends or league): the game sets `tab`. |

Pack slots: `Button.plate`, `Chip.frame`, `Panel.frame`, `Plate.frame`, `Row.frame`, `Row.frame.selected`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Profile), `back` (Back), `tabNext` (Next tab), `tabPrev` (Previous tab).

## 198. LoadingScreen

The key art fills the screen. At its foot, over a shade that grows with the words: where the player is going (an eyebrow and the place), the tip on a cut smoked-glass plate, and the progress as the HUD's leaning bar with a spinner and the figure; once ready, Continue (or nothing, when the game does not wait). A press anywhere asks for the next tip.

`import { LoadingScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/loading.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `keyArt` | `Texture \| 0` | The key art's texture (og.load.texture; 0: the style's placeholder). |
| `place` | `string` | Where the player is going. |
| `tip` | `string` | A tip or a line of lore. |
| `progress` | `number` | 0 to 1. |
| `ready?` | `boolean` | Loading is done. |
| `waits?` | `boolean` | The game waits for a press once ready. |
| `status?` | `string` | What the wait is for, beside the figure while it is not ready (Waiting for the server); empty: none. |
| `lore?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the lore snippet). |
| `onContinue?` | `(e: UiEvent, key?: string) => void` | The continue event, with the row's key where it comes from a row. |
| `onNextTip?` | `(e: UiEvent, key?: string) => void` | The nextTip event, with the row's key where it comes from a row. |

Pack slots: `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Continue).

## 199. MainMenuScreen

The main menu over the key art: the logo or the name as a wordmark, Continue with the last save's line when a save exists, New game, Modes, Settings, Credits and Quit, each drawn only when the game passes its handler, a news card (pressable with onNews) and the version. It resolves in the dark scheme in both schemes.

`import { MainMenuScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/main-menu.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `canContinue?` | `boolean` | A save exists: Continue shows and takes the focus. |
| `saveLine` | `string` | Under Continue: where the last save stands (chapter, time played). |
| `version` | `string` | The build's version, shown small in the corner. |
| `news` | `string` | A line for the news card (empty: no card). |
| `keyArt` | `Texture \| 0` | The key art's texture behind the menu (og.load.texture; 0: the style's dark ground). The screen sits over a picture, so it resolves in the dark scheme in both: its veils darken the art instead of washing it pale. |
| `logo` | `Texture \| 0` | The game's logo texture (og.load.texture; 0: the name set as a wordmark). |
| `name` | `string` | The game's name, set as a wordmark where there is no logo (empty: the logo slot's placeholder). |
| `background?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the background snippet). |
| `extra?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the extra snippet). |
| `onContinue?` | `((e: UiEvent, key?: string) => void) \| undefined` | The continue event, with the row's key where it comes from a row. |
| `onCredits?` | `((e: UiEvent, key?: string) => void) \| undefined` | The credits event, with the row's key where it comes from a row. |
| `onModes?` | `((e: UiEvent, key?: string) => void) \| undefined` | The modes event, with the row's key where it comes from a row. |
| `onNewGame?` | `((e: UiEvent, key?: string) => void) \| undefined` | The newGame event, with the row's key where it comes from a row. |
| `onNews?` | `((e: UiEvent, key?: string) => void) \| undefined` | The news event, with the row's key where it comes from a row. |
| `onQuit?` | `((e: UiEvent, key?: string) => void) \| undefined` | The quit event, with the row's key where it comes from a row. |
| `onSettings?` | `((e: UiEvent, key?: string) => void) \| undefined` | The settings event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Select).

## 200. ModeSelectScreen

A mode select: the modes as cards with their line, tag and picture, a locked mode's reason, the selected one's detail, and Play. It reports onSelect and onPlay; the game decides what is locked.

`import { ModeSelectScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/mode-select.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `modes` | `ReadonlyArray<{ id: string; line: string; locked: boolean; name: string; reason: string; tag: string; texture: Texture \| 0 }>` | The modes: { name, line, tag, locked, reason, texture }. |
| `selected` | `string` | The stable collection key selected now. |
| `keyArt` | `Texture \| 0` | The key art's texture under the screen's veil (og.load.texture; 0: the style's dark ground). |
| `detail?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the detail snippet). |
| `onBack?` | `((e: UiEvent, key?: string) => void) \| undefined` | The back event, with the row's key where it comes from a row. |
| `onPlay?` | `((e: UiEvent, key?: string) => void) \| undefined` | The play event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Play), `back` (Back).

## 201. NotificationsScreen

The notification stack in the HUD's top-right corner. A notification is a glass plate with two corners cut, a rail down its leading edge and a wash from it in its tone, the tone's mark in an octagon socket, its title in small capitals of the tone and its body in white, and a quiet close. A toast is the compact form: its mark and its words on a plate that slides back out when it leaves. Each tone has its own hue (warn an amber well apart from gold). Under reduced motion they appear in place.

`import { NotificationsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/notifications.tsx`).

| Prop | Type | What |
|---|---|---|
| `items` | `ReadonlyArray<{ id: string; title: string; body: string; tone: string }>` | The game's own notifications, newest last, a keyed collection of { id, title, body, tone (info, success, warn, danger) }; the game removes one when it is read or times out. The toasts follow them. |
| `toasts?` | `ReadonlyArray<{ id: string; tone: string; text: string; leaving?: boolean }>` | The game's toasts, as its own keyed rows of { id, tone (info, success, warn, danger), text, leaving (its leave is playing): the game removes one when it leaves and a press dismisses it. |
| `onDismiss?` | `(e: UiEvent, key?: string) => void` | The dismiss event, with the row's key where it comes from a row. |
| `onDismissToast?` | `(e: UiEvent, key?: string) => void` | The dismissToast event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Plate.frame`.

Fixtures: `default`.

Actions: `dismiss` (Dismiss).

## 202. ObjectiveTracker

The tracked objectives over the scene, with no plate: the quest in gold small capitals, then each objective with a gold diamond (filled once done, its words struck and dimmed) over a thin leaning bar in a dark well, cut into cells when the game counts its steps. The first not done is lit; on a phone only it is shown. Read only.

`import { ObjectiveTracker } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/objective-tracker.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The quest or chapter. |
| `objectives` | `ReadonlyArray<{ id: string; label: string; progress: number; done: boolean; steps: number }>` | A keyed collection of { id, label, progress (0–1), done, steps }: steps, when above 1, cuts the bar into that many cells (2 / 5 is five); 0 or absent is one bar. |

Pack slots: `Plate.frame`.

Fixtures: `default`.

## 203. Pagination

A row of cut keys, the current page lit.

`import { Pagination } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/pagination.tsx`).

| Prop | Type | What |
|---|---|---|
| `pages` | `ReadonlyArray<{ id: string; label: string; current?: boolean; gap?: boolean }>` | The page buttons, a keyed collection of { id, label, current }; a gap is { id, label: "…", gap: true }. |
| `label` | `string` | The navigation's accessible name (Pages). |
| `atStart?` | `boolean` | The first page is shown: Previous takes no press. |
| `atEnd?` | `boolean` | The last page is shown: Next takes no press. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onPage?` | `(e: UiEvent, key?: string) => void` | The page event, with the row's key where it comes from a row. |
| `onPrevious?` | `(e: UiEvent, key?: string) => void` | The previous event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`.

Fixtures: `default`.

Actions: `previous` (Previous page), `next` (Next page), `page` (Go to page).

## 204. PauseScreen

The pause menu: Resume, Settings, Controls, Save (grey when not allowed), Load, Quit to menu and Quit game, each drawn only when the game passes its handler, and the game's own rows among them, with the current objective and the time played beside it. Each reports its event.

`import { PauseScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/pause.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `objective` | `string` | The current objective (empty: none). |
| `played` | `number` | Seconds played this run. |
| `canSave?` | `boolean` | Saving is allowed here. |
| `rows?` | `ReadonlyArray<PauseRow>` | The game's own rows (Restart stage), each drawn as a built-in row and reported by onRow with its key: in the main group after the built-in ones, or after the row `after` names (a built-in row's key: resume, settings, controls, save, load, quitToMenu, quit; or an earlier row of the game's); a `minor` row goes below the rule with Quit to menu and Quit game. |
| `onRow?` | `((e: UiEvent, key: string) => void) \| undefined` | A row of the game's (props.rows) pressed, with its key. |
| `side?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the side snippet). |
| `onControls?` | `((e: UiEvent, key?: string) => void) \| undefined` | The controls event, with the row's key where it comes from a row. |
| `onLoad?` | `((e: UiEvent, key?: string) => void) \| undefined` | The load event, with the row's key where it comes from a row. |
| `onQuit?` | `((e: UiEvent, key?: string) => void) \| undefined` | The quit event, with the row's key where it comes from a row. |
| `onQuitToMenu?` | `((e: UiEvent, key?: string) => void) \| undefined` | The quitToMenu event, with the row's key where it comes from a row. |
| `onResume?` | `((e: UiEvent, key?: string) => void) \| undefined` | The resume event, with the row's key where it comes from a row. |
| `onSave?` | `((e: UiEvent, key?: string) => void) \| undefined` | The save event, with the row's key where it comes from a row. |
| `onSettings?` | `((e: UiEvent, key?: string) => void) \| undefined` | The settings event, with the row's key where it comes from a row. |

Each `PauseRow`:

| Field | Type | What |
|---|---|---|
| `key` | `string` | Its stable key, reported by onRow. |
| `label` | `string` | What it says. |
| `after?` | `string` | The key of the row it follows; empty or unknown: the end of its group. |
| `minor?` | `boolean` | Below the rule, with Quit to menu and Quit game, without a chevron. |
| `danger?` | `boolean` | Tinted as a danger when hovered, as Quit game is. |
| `disabled?` | `boolean` | Drawn grey and takes no press. |

Pack slots: `Panel.frame`, `Screen.frame`.

Fixtures: `default`, `ownRows`.

Actions: `confirm` (Select), `back` (Resume).

## 205. ResultsScreen

After a match: the verdict set large in a lit fill (gold for a win, red for a loss) over the dimmed scene, a rule with a lit diamond, the level in an octagon beside the XP gained and the leaning progress bar, the grade on a hexagon, then the tabs. Summary shows each stat on a cut plate (a best in gold) and each reward as octagon art with its amount on a tag and its name under it. The page scrolls over the actions.

`import { ResultsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/results.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `won?` | `boolean` | The player won. |
| `grade` | `string` | The grade (S, A, B …; empty: none). |
| `title` | `string` | The result's line (Victory, Stage clear). |
| `stats` | `ReadonlyArray<{ best: boolean; count?: boolean; id: string; name: string; value: number; format?: string; decimals?: number }>` | The stats: { name, value, best (bool), format (number or percent; defaults to number), count (bool), decimals }. A stat with count counts up from 0 as the screen opens, written whole or with decimals digits after the point; the others show their value as it is, fractions kept. |
| `level?` | `number` | The level after the match; absent, no level, XP or progress is drawn (a game without levels). |
| `xp?` | `number` | Progress to the next level after the match, 0 to 1. |
| `gained?` | `number` | XP gained. |
| `rewards` | `ReadonlyArray<{ amount: number; id: string; name: string; texture: Texture \| 0; icon?: string }>` | The rewards: { name, amount, texture, icon? }: texture the reward's picture filling its well; 0 shows icon (an atlas icon name, as a Shop price's) on the well's plate. |
| `tab` | `string` | summary, graphs or board. |
| `celebration?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the celebration snippet). |
| `graph?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the graph snippet). |
| `scores?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the scores snippet). |
| `onAgain?` | `((e: UiEvent, key?: string) => void) \| undefined` | The again event, with the row's key where it comes from a row. |
| `onContinue?` | `((e: UiEvent, key?: string) => void) \| undefined` | The continue event, with the row's key where it comes from a row. |
| `onReplay?` | `((e: UiEvent, key?: string) => void) \| undefined` | The replay event, with the row's key where it comes from a row. |
| `onTab?` | `(e: UiEvent, value: string) => void` | The tab switched (onTab), the tab's value (summary, graphs or board). Without it no tab row is drawn, and the page shows `tab`. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Continue).

## 206. SaveLoadScreen

Saving or loading: the slots with their title, place, time played, date and thumbnail (autosaves marked, empty slots offered for a new save), the selected one's preview, and Save or Load and Delete. The game writes the saves.

`import { SaveLoadScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/save-load.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `mode` | `string` | save or load: the title and the main action. |
| `slots` | `ReadonlyArray<{ auto: boolean; date: string; empty: boolean; id: string; place: string; played: number; thumb: Texture \| 0; title: string }>` | The saves: { title, place, played (seconds), date (text), auto (bool), empty (bool), thumb (texture) }. |
| `selected` | `string` | The stable collection key selected now. |
| `preview?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the preview snippet). |
| `onBack?` | `((e: UiEvent, key?: string) => void) \| undefined` | The back event, with the row's key where it comes from a row. |
| `onDelete?` | `((e: UiEvent, key?: string) => void) \| undefined` | The delete event, with the row's key where it comes from a row. |
| `onLoad?` | `((e: UiEvent, key?: string) => void) \| undefined` | The load event, with the row's key where it comes from a row. |
| `onNewSlot?` | `((e: UiEvent, key?: string) => void) \| undefined` | The newSlot event, with the row's key where it comes from a row. |
| `onSave?` | `((e: UiEvent, key?: string) => void) \| undefined` | The save event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Row.frame`, `Row.frame.selected`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Select), `back` (Back), `details` (Delete).

## 207. SettingsScreen

Settings: a tab per page and its entries (toggles, ranges, choices and key bindings), Apply and Revert while changes wait, and Defaults. The game owns every value and writes back what the events report.

`import { SettingsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/settings.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `note` | `string` | A line under the title (Some changes apply after a restart); empty: none. |
| `pages` | `ReadonlyArray<{ id: string; name: string }>` | The settings' pages as a keyed collection of { id, name }: a tab each. |
| `entries` | `ReadonlyArray<{ id: string; page: string; name: string; description: string; kind: string; value: boolean \| number \| string; min?: number; max?: number; step?: number; options?: ReadonlyArray<string>; action?: string }>` | The settings as a keyed collection of { id, page, name, description, kind: toggle\|range\|select\|binding, value, min, max, step, options, action }, shown on their page's tab; each value is written back as the player changes it (two-way via onChange). |
| `page` | `string` | The id of the page shown (two-way via onPage: the game writes it back as the player picks a tab); empty: no tree of pages, the `tree` content alone. |
| `pending?` | `boolean` | Changes wait to be applied: Apply and Revert take presses. |
| `tree?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the tree snippet). |
| `onPage?` | `(e: UiEvent, id: string) => void` | The page tab picked, with the tab's id; the game writes props.page back. |
| `onChange?` | `(e: UiEvent, id: string, value: boolean \| number \| string) => void` | An entry's value changed, with the entry's id and its new value; the game writes entries' value back. |
| `onRebind?` | `(e: UiEvent, id: string, key: string) => void` | The key captured while a binding row listens, with the entry's id and the key; the game writes entries' value back. Escape cancels without calling this. |
| `onApply?` | `((e: UiEvent, key?: string) => void) \| undefined` | The apply event, with the row's key where it comes from a row. |
| `onBack?` | `((e: UiEvent, key?: string) => void) \| undefined` | The back event, with the row's key where it comes from a row. |
| `onDefaults?` | `((e: UiEvent, key?: string) => void) \| undefined` | The defaults event, with the row's key where it comes from a row. |
| `onRevert?` | `((e: UiEvent, key?: string) => void) \| undefined` | The revert event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Row.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Select), `back` (Back), `tabNext` (Next page), `tabPrev` (Previous page).

## 208. SplashScreen

The boot splash: the studio marks one after another (a texture, or the name set as type), boot progress and what is loading, and Skip. The game moves through the marks.

`import { SplashScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/splash.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `marks` | `ReadonlyArray<{ id: string; name: string; texture: Texture \| 0 }>` | The marks in order: { name, texture } (texture: an og.load.texture handle; 0 sets the name as a wordmark). |
| `mark` | `string` | The stable collection key selected now. |
| `progress` | `number` | Boot progress, 0 to 1 (below 0: no bar). |
| `status` | `string` | What is loading, small beside the bar's figure (empty: nothing). |
| `custom?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the custom snippet). |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |

Pack slots: `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Skip).

## 209. StarRating

A row of rating stars, each lit once it is earned. The style draws the stars as SVG glyphs (an outlined empty star, a gold one with a lit edge and a glow); the engine draws no vector shapes, so each is the atlas star icon, one tint lit and a dim one unearned, landing with the pop entrance (the style's land, without its overshoot) delayed by the star's index.

`import { StarRating } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/star-rating.tsx`).

| Prop | Type | What |
|---|---|---|
| `stars` | `ReadonlyArray<{ id: string; index: number; lit: boolean }>` | A keyed collection of { id, index (0 first), lit }: one star each; a lit star is earned and drawn filled. |
| `label` | `string` | What it says to a screen reader (2 of 3 stars). |
| `size?` | `"md" \| "lg"` | md (the default): 32 px stars in a line. lg: a result's grade, 78 px stars (52 on a phone) with the middle of three raised and larger, as the puzzle summary draws them. |
| `class?` | `string \| undefined` | Classes after the row's own (a placement, a margin). |

Fixtures: `default`, `none`, `large`, `all`.

## 210. Subtitles

The latest lines at the foot of the screen, centred in HUD type over a soft dark band: the speaker in gold small capitals over the words in white. An older line steps back, smaller. Nothing shows without a line.

`import { Subtitles } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/subtitles.tsx`).

| Prop | Type | What |
|---|---|---|
| `lines?` | `ReadonlyArray<{ id: string; speaker: string; text: string }> \| undefined` | The game's own lines, oldest first, each { id, speaker, text }; the game drops a line when it is done. |
| `captions?` | `ReadonlyArray<{ id: string; text: string }> \| undefined` | The captions the timelines are saying (og.timeline.captions(world), set each tick), after the game's lines: a film segment's SRT track, a cutscene's caption track. |

Pack slots: `Plate.frame`, `Screen.frame`.

Fixtures: `default`, `captions`.

## 211. Table

A HUD plate of plain rows: the first column a name, the others figures.

`import { Table } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/table.tsx`).

| Prop | Type | What |
|---|---|---|
| `columns` | `ReadonlyArray<{ id: string; label: string }>` | The headings, a keyed collection of { id, label }. |
| `cells` | `ReadonlyArray<{ id: string; row: string; text: string }>` | The body's cells row by row, a keyed collection of { id, row, text }: three to a row (eject it to change the columns). A press sends choose with the cell's id; its row field names the row. |
| `label` | `string` | The table's accessible name. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`.

Actions: `choose` (Open row).

## 212. TitleScreen

The title screen over the key art: the logo or the name as a wordmark, Press any button (Tap on a touch screen) when the game passes onStart, the legal line and the version. Any press reports onStart. It resolves in the dark scheme in both schemes.

`import { TitleScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/title.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `version` | `string` | The build's version. |
| `legal` | `string` | The legal line at the foot (copyright, trademarks). |
| `keyArt` | `Texture \| 0` | The key art's texture under the title (og.load.texture; 0: the style's dark ground). The screen sits over a picture, so it resolves in the dark scheme in both: its veils darken the art instead of washing it pale. |
| `logo` | `Texture \| 0` | The game's logo texture (og.load.texture; 0: the name set as a wordmark). |
| `name` | `string` | The game's name, set as a wordmark where there is no logo (empty: the logo slot's placeholder). |
| `background?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the background snippet). |
| `onStart?` | `((e: UiEvent, key?: string) => void) \| undefined` | The start event, with the row's key where it comes from a row. |

Pack slots: `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Start).

## 213. Tree

A HUD plate of rows, each indented by its depth with a faint guide down from its parent; the chosen row is washed in the accent from its leading edge. A leaf has no twisty and keeps its room, so its name lines up with its siblings' that open. A name longer than its row wraps inside it, at any word.

`import { Tree } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/tree.tsx`).

| Prop | Type | What |
|---|---|---|
| `rows?` | `ReadonlyArray<{ id: string; label: string; depth: number; open: boolean \| null; selected: boolean }>` | The visible rows in order, a keyed collection of { id, label, depth (0 at the top), open (true, false, or null for a leaf), selected }; the game flattens the tree as it opens and closes. |
| `label` | `string` | The tree's accessible name. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onToggle?` | `(e: UiEvent, key?: string) => void` | The toggle event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`.

Fixtures: `default`, `overflow`.

Actions: `toggle` (Open or close), `choose` (Select).

## 214. TutorialScreen

A tutorial step: a card with the step's title, what to do, its art and the step count, and Next (hidden while the step waits for the player) and Skip. Given a target that shows, the screen dims around that element, which keeps its own presses, and the card stands under it; else (no target, or one not on screen) nothing dims and the card stands where the game puts it.

`import { TutorialScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/tutorial.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The step's title. |
| `text` | `string` | What to do. |
| `step` | `number` | The step, from 1. |
| `steps` | `number` | Steps in the lesson. |
| `waits?` | `boolean` | The step waits for the player to act (Next hidden). |
| `x` | `number` | The card's start edge, CSS px from the screen's start (the right, right to left). |
| `y` | `number` | The card's top, CSS px. |
| `texture` | `Texture \| 0` | The step's art (0: none). |
| `target?` | `string` | The `name` of the element the step points at, anywhere in the UI (name a `View` around an `Icon`): the rest of the screen dims and takes every press, the element stays lit and takes its own, and the card stands under it in place of x and y. Empty: nothing is dimmed and the card stands at x and y. While no element of that name shows, nothing is dimmed and the card stands at x and y. |
| `highlight?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the highlight snippet). |
| `hint?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the hint snippet). |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Screen.frame`.

Fixtures: `default`, `spotlight`, `targetMissing`.

Actions: `confirm` (Next), `back` (Skip).
