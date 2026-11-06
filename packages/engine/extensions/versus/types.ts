// The versus kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Rarity, Texture } from "@origingame/engine";

/**
 * A character select: the roster, the players' cursors and the words.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CharacterSelectDesc {
  /**
   * The screen's title (Choose your fighter).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The roster in its order: the grid fills rows from the start.
   *
   * @default []
   */
  readonly roster?: readonly RosterEntry[];
  /**
   * The players' cursors: the local player's splash stands on the start side; with two players, the other's on the end side.
   *
   * @default []
   */
  readonly cursors?: readonly SelectCursor[];
  /**
   * Tiles a row; 0 fits as many 96 px tiles as the grid's width holds (four on a phone in portrait).
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * Time left to pick, shown as a timer over the grid; a negative value shows none.
   *
   * @default -1
   * @unit s (seconds of simulation time)
   */
  readonly secondsLeft?: number;
  /**
   * The local player's confirm.
   *
   * @default "Lock in"
   */
  readonly lockLabel?: string;
  /**
   * The confirm once locked in.
   *
   * @default "Locked in"
   */
  readonly lockedInLabel?: string;
  /**
   * A taken character's reason.
   *
   * @default "Taken"
   */
  readonly takenLabel?: string;
  /**
   * The random pick's tile; empty shows none.
   *
   * @default "Random"
   */
  readonly randomLabel?: string;
}

/** What the local player asked for this tick, for the game to apply. */
export interface CharacterSelectResult {
  /**
   * The roster key under the pointer or the focus: move the local player's cursor to it (a fighting game's cursor follows the stick).
   */
  readonly hovered: string | null;
  /** A character pressed or confirmed (its tile, or Lock in on the cursor's): lock it in. */
  readonly picked: string | null;
  /** The random tile was pressed: pick one for the player. */
  readonly random: boolean;
  /** A locked or taken character pressed: it shook. */
  readonly refused: string | null;
  /** Back while locked in: unlock the pick. */
  readonly cancelled: boolean;
}

/**
 * One chat channel (General, Party, Guild, Whispers, Trade): a tab, and an ink its messages wear.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ChatChannel {
  /**
   * Its key: its tab's element id (<id>/Tabs/Row/<key>), and the messages' `channel`.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name on its tab and in the tag before its messages where several channels show.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its ink: a theme colour's name (info, success) or `#rrggbb`; empty is the ink.
   *
   * @default ""
   */
  readonly color?: string;
  /**
   * Messages the player has not seen in it: a count on its tab.
   *
   * @default 0
   */
  readonly unread?: number;
  /**
   * Unread messages naming the player: its tab's count turns to the highlight.
   *
   * @default 0
   */
  readonly mentions?: number;
  /**
   * The tab that shows every channel's messages, each tagged with its channel.
   *
   * @default false
   */
  readonly all?: boolean;
  /**
   * No sending to it (System, Combat log): the input's channel skips it.
   *
   * @default false
   */
  readonly readOnly?: boolean;
}

/**
 * A chat window: the channels, the messages, the player's draft and the channel it goes to.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ChatDesc {
  /**
   * The window's title; empty shows none.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The tabs, start to end.
   *
   * @default []
   */
  readonly channels?: readonly ChatChannel[];
  /**
   * The tab shown.
   *
   * @default ""
   */
  readonly channel?: string;
  /**
   * The messages, oldest first (the game keeps its own limit).
   *
   * @default []
   */
  readonly messages?: readonly ChatMessage[];
  /**
   * The local player's name: a message mentioning @me is lit.
   *
   * @default ""
   */
  readonly me?: string;
  /**
   * Who a mention can name.
   *
   * @default []
   */
  readonly members?: readonly ChatMember[];
  /**
   * The input's text (the game's: it passes ChatResult.draft back).
   *
   * @default ""
   */
  readonly draft?: string;
  /**
   * The channel the input sends to; empty is the tab shown (or the first one that can be sent to).
   *
   * @default ""
   */
  readonly sendTo?: string;
  /**
   * The longest message.
   *
   * @default 255
   */
  readonly maxChars?: number;
  /**
   * The HUD's chat (World of Warcraft, Valorant): no panel, the newest lines over the game on a soft shade, fading after `fadeAfter` seconds unless the pointer is over it; the tabs and the input show while `typing`.
   *
   * @default false
   */
  readonly hud?: boolean;
  /**
   * With hud: the player is writing (the game opens the input with Enter and closes it on a send or Back).
   *
   * @default false
   */
  readonly typing?: boolean;
  /**
   * With hud: seconds a line stays before it fades.
   *
   * @default 10
   */
  readonly fadeAfter?: number;
  /**
   * The empty input's words.
   *
   * @default "Say something…"
   */
  readonly placeholder?: string;
  /**
   * The send button's word.
   *
   * @default "Send"
   */
  readonly sendLabel?: string;
  /**
   * The pill over the log when newer messages are below the view ({n} is how many).
   *
   * @default "{n} new"
   */
  readonly newLabel?: string;
  /**
   * The input's channel chip, to a screen reader.
   *
   * @default "Channel"
   */
  readonly channelLabel?: string;
}

/**
 * A link inside a message: an item, a player, a place or a quest, which a press answers by its key.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ChatLink {
  /**
   * Its key, answered in ChatResult.link.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its words, shown in brackets ([Ashbringer]).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * What it links (item, player, place, quest), to a screen reader.
   *
   * @default ""
   */
  readonly kind?: string;
  /**
   * An item's tier: the link's ink (common shows the accent).
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
}

/**
 * Someone a mention can name: the suggestions while an @ word is typed.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ChatMember {
  /**
   * Their key.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Their name as a mention writes it.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Their portrait (art kind portrait) in the suggestion; null shows none.
   *
   * @default null
   */
  readonly avatar?: Texture | null;
}

/**
 * One message, as the game holds it. Its words are shown as they were typed: no markup is read in them.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ChatMessage {
  /**
   * Its key: its line's element id, so its probe path is <id>/Log/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * The channel it was said in.
   *
   * @default ""
   */
  readonly channel?: string;
  /**
   * Who said it; empty for a system line.
   *
   * @default ""
   */
  readonly author?: string;
  /**
   * The author's key, answered in ChatResult.author when their name is pressed (a whisper, a profile).
   *
   * @default ""
   */
  readonly authorKey?: string;
  /**
   * The author's portrait (art kind portrait) at the line's start; null shows none.
   *
   * @default null
   */
  readonly avatar?: Texture | null;
  /**
   * The words. `{0}`, `{1}` stand for its links in order; `@Name` naming a member is a mention.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * Its links, by their place in the text.
   *
   * @default []
   */
  readonly links?: readonly ChatLink[];
  /**
   * When, as the game writes it (12:04); empty shows none.
   *
   * @default ""
   */
  readonly time?: string;
  /**
   * A notice from the game (Kael joined the party): muted, with no author.
   *
   * @default false
   */
  readonly system?: boolean;
  /**
   * The local player's own message.
   *
   * @default false
   */
  readonly own?: boolean;
}

/** What the player asked for this tick, for the game to apply. */
export interface ChatResult {
  /** Another tab was pressed. */
  readonly channel: string | null;
  /** The input's channel chip was pressed: the next channel that can be sent to. */
  readonly sendTo: string | null;
  /** The input's text changed (typed, or a mention suggestion inserted). */
  readonly draft: string | null;
  /** Enter or Send with words in the input (the game posts it and clears the draft). */
  readonly send: ChatSend | null;
  /** A link was pressed: its key. */
  readonly link: string | null;
  /** An author's name was pressed: their key. */
  readonly author: string | null;
  /** With hud and typing: Back was pressed (the game closes the input). */
  readonly closed: boolean;
}

/** A message the player sent. */
export interface ChatSend {
  /** The channel it goes to. */
  readonly channel: string;
  /** Its words, trimmed. */
  readonly text: string;
}

/**
 * A draft: two teams' picks and bans, the roster, the phase and its timer.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DraftDesc {
  /**
   * The two teams, the local player's first (it stands on the start side).
   *
   * @default []
   */
  readonly teams?: readonly DraftTeam[];
  /**
   * The roster: taken (picked by anyone) and banned characters are told by the teams' lists.
   *
   * @default []
   */
  readonly roster?: readonly RosterEntry[];
  /**
   * ban, pick or done: what the active slot does (Ban in danger, Lock in in the accent).
   *
   * @default ""
   */
  readonly phase?: string;
  /**
   * The banner over the roster (Ban phase · Blue bans 2).
   *
   * @default ""
   */
  readonly phaseLabel?: string;
  /**
   * Time left in this turn.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly secondsLeft?: number;
  /**
   * The turn's whole time, for the bar; 0 is 30.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly seconds?: number;
  /**
   * The local player acts now: the roster takes presses and the action is live.
   *
   * @default false
   */
  readonly yourTurn?: boolean;
  /**
   * The roster key the local player has hovered (its slot shows it dimmed, its tile is rimmed).
   *
   * @default ""
   */
  readonly hovered?: string;
  /**
   * Tiles a row; 0 fits 88 px tiles.
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * The pick action.
   *
   * @default "Lock in"
   */
  readonly lockLabel?: string;
  /**
   * The ban action.
   *
   * @default "Ban"
   */
  readonly banLabel?: string;
  /**
   * The action while it is not the local player's turn.
   *
   * @default "Waiting"
   */
  readonly waitingLabel?: string;
  /**
   * The bans row's label.
   *
   * @default "Bans"
   */
  readonly bansLabel?: string;
  /**
   * A banned character, to a screen reader.
   *
   * @default "Banned"
   */
  readonly bannedLabel?: string;
  /**
   * A picked character, to a screen reader.
   *
   * @default "Picked"
   */
  readonly pickedLabel?: string;
}

/** What the local player asked for this tick, for the game to apply. */
export interface DraftResult {
  /**
   * An open character pressed or focused in the local player's turn: hover it (its slot shows it).
   */
  readonly hovered: string | null;
  /**
   * The action, or confirm, on the hovered character: lock it in (a pick or a ban by the phase).
   */
  readonly locked: string | null;
  /** A banned or picked character pressed: it shook. */
  readonly refused: string | null;
}

/**
 * One pick of a team in a draft.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DraftSlot {
  /**
   * The player's name.
   *
   * @default ""
   */
  readonly player?: string;
  /**
   * The roster key picked or hovered (empty: none yet).
   *
   * @default ""
   */
  readonly entry?: string;
  /**
   * The pick is locked in: the slot shows the character whole; else a hover shows it dimmed.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * This slot picks now: lit, with the timer's light along it.
   *
   * @default false
   */
  readonly active?: boolean;
  /**
   * The local player's slot.
   *
   * @default false
   */
  readonly you?: boolean;
}

/**
 * One side of a draft.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DraftTeam {
  /**
   * The team's name (Blue side).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its team, for its colour.
   *
   * @default 0
   */
  readonly team?: number;
  /**
   * Its picks, in pick order.
   *
   * @default []
   */
  readonly picks?: readonly DraftSlot[];
  /**
   * Its bans, roster keys in ban order; an empty key is a ban still to come.
   *
   * @default []
   */
  readonly bans?: readonly string[];
}

/**
 * A party lobby: who is in it, the mode, matchmaking and the ready check.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LobbyDesc {
  /**
   * The screen's title (Play, Party, Lobby).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The party, the local player among them, in the game's order (the local player's card is drawn first).
   *
   * @default []
   */
  readonly members?: readonly LobbyMember[];
  /**
   * The party's size: the places past the members are empty seats, each an Invite; 0 is the members' count.
   *
   * @default 0
   */
  readonly slots?: number;
  /**
   * The mode the party queues for: a card at the foot's start, pressed to change it (LobbyResult.mode).
   *
   * @default {}
   */
  readonly mode?: LobbyMode;
  /**
   * Matchmaking: the foot's action becomes Cancel and a chip counts the time in queue.
   *
   * @default false
   */
  readonly searching?: boolean;
  /**
   * Seconds in queue.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly searchSeconds?: number;
  /**
   * The estimated wait, said beside the count; 0 says none.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly estimateSeconds?: number;
  /**
   * The local player may start matchmaking (the leader, with the party ready): the foot's action is Find match; else it is the ready toggle.
   *
   * @default false
   */
  readonly canStart?: boolean;
  /**
   * A match found: the ready check stands over the lobby until the game clears it.
   *
   * @default null
   */
  readonly readyCheck?: ReadyCheckView | null;
  /**
   * The ready word on a card, and the toggle once ready.
   *
   * @default "Ready"
   */
  readonly readyLabel?: string;
  /**
   * The word on a card not ready.
   *
   * @default "Not ready"
   */
  readonly notReadyLabel?: string;
  /**
   * The toggle while not ready.
   *
   * @default "Ready up"
   */
  readonly readyUpLabel?: string;
  /**
   * The leader's action.
   *
   * @default "Find match"
   */
  readonly startLabel?: string;
  /**
   * The action while searching.
   *
   * @default "Cancel"
   */
  readonly cancelLabel?: string;
  /**
   * The chip's word while searching.
   *
   * @default "Searching"
   */
  readonly searchingLabel?: string;
  /**
   * The estimate beside the count ({time}).
   *
   * @default "Est. {time}"
   */
  readonly estimateLabel?: string;
  /**
   * An empty seat's word.
   *
   * @default "Invite"
   */
  readonly inviteLabel?: string;
  /**
   * A level ({level}).
   *
   * @default "Lv {level}"
   */
  readonly levelLabel?: string;
  /**
   * The leader, to a screen reader.
   *
   * @default "Party leader"
   */
  readonly leaderLabel?: string;
  /**
   * The ready check's title.
   *
   * @default "Match found"
   */
  readonly matchFoundLabel?: string;
  /**
   * The ready check's accept.
   *
   * @default "Accept"
   */
  readonly acceptLabel?: string;
  /**
   * Accept once pressed.
   *
   * @default "Accepted"
   */
  readonly acceptedLabel?: string;
  /**
   * The ready check's decline.
   *
   * @default "Decline"
   */
  readonly declineLabel?: string;
  /**
   * The mode card's eyebrow.
   *
   * @default "Mode"
   */
  readonly modeLabel?: string;
}

/**
 * One player in the party, as the game's viewmodel holds it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LobbyMember {
  /**
   * Its key, stable while it is in the party: its element id, so its probe path is <id>/Lineup/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * The player's name: the card's reading, set in the display face.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The player's level; 0 shows none.
   *
   * @default 0
   */
  readonly level?: number;
  /**
   * A line under the name: a rank, a class, a club (empty: none).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * Its character or avatar (art kind character), filling the card; null draws the style's placeholder figure.
   *
   * @default null
   */
  readonly portrait?: Texture | null;
  /**
   * Ready: the card's foot lit in success with a tick and the ready word.
   *
   * @default false
   */
  readonly ready?: boolean;
  /**
   * The party's leader: a crown at the card's top start.
   *
   * @default false
   */
  readonly leader?: boolean;
  /**
   * The local player: its card stands larger and first, its foot reads the ready toggle's state.
   *
   * @default false
   */
  readonly you?: boolean;
  /**
   * A line in place of the ready word (In a match, Away); empty is ready or not ready.
   *
   * @default ""
   */
  readonly status?: string;
  /**
   * In a ready check: has accepted.
   *
   * @default false
   */
  readonly accepted?: boolean;
  /**
   * In a ready check: has declined.
   *
   * @default false
   */
  readonly declined?: boolean;
}

/**
 * The mode the party queues for.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LobbyMode {
  /**
   * Its name (Ranked, Duos, Grand Prix).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * A line under it (5v5 · Summoner's Rift; 4 players).
   *
   * @default ""
   */
  readonly detail?: string;
  /**
   * Its art (art kind thumbnail), 16:9; null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
}

/** What the lobby was asked this tick, for the game to do. */
export interface LobbyResult {
  /** The ready toggle was pressed: flip the local player's ready. */
  readonly ready: boolean;
  /** Find match was pressed. */
  readonly start: boolean;
  /** Cancel was pressed while searching. */
  readonly cancel: boolean;
  /** An empty seat was pressed: its place, from 0 at the first empty seat. */
  readonly invite: number | null;
  /** A member's card was pressed: the game's menu for it (profile, promote, kick). */
  readonly member: string | null;
  /** The mode card was pressed: open the mode select. */
  readonly mode: boolean;
  /** The ready check was accepted (its button, or confirm). */
  readonly accept: boolean;
  /** The ready check was declined (its button, or Back). */
  readonly decline: boolean;
}

/**
 * A match found, waiting for every player to accept.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ReadyCheckView {
  /**
   * Seconds left to accept.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly secondsLeft?: number;
  /**
   * The whole time to accept, for the ring; 0 is 10.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly seconds?: number;
}

/**
 * One character on a select screen's roster.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RosterEntry {
  /**
   * Its key: its tile's element id, so its probe path is <id>/Stage/Roster/Row#<r>/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name, large over its splash when a cursor is on it.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its role or class (Duelist, Grappler, Support): the eyebrow over the name.
   *
   * @default ""
   */
  readonly role?: string;
  /**
   * Its tile's art (art kind portrait), the face filling the tile; null draws the style's placeholder figure.
   *
   * @default null
   */
  readonly portrait?: Texture | null;
  /**
   * Its full art (art kind splash) standing large on its player's side; null takes the portrait.
   *
   * @default null
   */
  readonly splash?: Texture | null;
  /**
   * Not owned or not unlocked: the tile dark with a lock; a press is refused and says why.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * Why it is locked (Unlock at level 10).
   *
   * @default ""
   */
  readonly lockedHint?: string;
  /**
   * Picked by another player of the local player's team: the tile dimmed, a press refused.
   *
   * @default false
   */
  readonly taken?: boolean;
  /**
   * A line under the name on its splash (its play style).
   *
   * @default ""
   */
  readonly description?: string;
}

/**
 * One player's cursor on the roster.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SelectCursor {
  /**
   * The player's key (p1, p2).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * The player's name or slot (P1, Mira): its tag on the tile and over its splash.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The roster key its cursor is on (empty: none yet).
   *
   * @default ""
   */
  readonly entry?: string;
  /**
   * Locked in: its tile wears the tick and its splash its lock.
   *
   * @default false
   */
  readonly confirmed?: boolean;
  /**
   * Its team, for its colour (the style's team roles).
   *
   * @default 0
   */
  readonly team?: number;
  /**
   * The local player: keys, pads and the pointer move this cursor (SelectResult.hovered).
   *
   * @default false
   */
  readonly you?: boolean;
}

/**
 * A versus screen: two sides, the stage, the words.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface VersusDesc {
  /**
   * The start side (the local player's).
   *
   * @default {}
   */
  readonly left?: VersusSide;
  /**
   * The end side.
   *
   * @default {}
   */
  readonly right?: VersusSide;
  /**
   * The mark between them.
   *
   * @default "VS"
   */
  readonly versusLabel?: string;
  /**
   * The stage or mode under the mark (Round 1 · Skyreach Gate).
   *
   * @default ""
   */
  readonly stage?: string;
  /**
   * Loading progress, 0 to 1, as a bar along the foot; a negative value shows none.
   *
   * @default -1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly progress?: number;
  /**
   * The skip hint; empty takes no skip.
   *
   * @default "Skip"
   */
  readonly skipLabel?: string;
}

/** What the versus screen was asked this tick. */
export interface VersusResult {
  /** Confirm, Back or a press asked to go on. */
  readonly skipped: boolean;
}

/**
 * One side of a versus screen.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface VersusSide {
  /**
   * The player's or team's name, in the display face.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The character's name, over the player's.
   *
   * @default ""
   */
  readonly character?: string;
  /**
   * The character's full art (art kind splash), filling the side; null draws the style's placeholder figure.
   *
   * @default null
   */
  readonly splash?: Texture | null;
  /**
   * A line under the name (a rank, a record: 12W 3L).
   *
   * @default ""
   */
  readonly detail?: string;
  /**
   * Its team, for its colour.
   *
   * @default 0
   */
  readonly team?: number;
}
