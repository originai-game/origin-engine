// The rpg kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Anchor, DialogueView, Direction, MeterKind, Paint, QuestView, Rarity, Size, StatusEffect, Texture, UiRect, UnitClass, Vector2, Vector3 } from "@origingame/engine";

/**
 * A cast or channel bar.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CastBarDesc {
  /**
   * What is cast.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its icon, drawn large on a gradient of its own hue when there is no image.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Its art (art kind ability, 1:1), filling the tile at the bar's start; null draws the icon.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * How far: a cast fills, a channel drains.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly progress?: number;
  /**
   * Seconds left (shown at its end).
   *
   * @default 0
   * @unit s (seconds of simulation time)
   */
  readonly remaining?: number;
  /**
   * A channel: the bar drains, with tick marks at ticks.
   *
   * @default false
   */
  readonly channel?: boolean;
  /**
   * A channel's ticks (marks along it).
   *
   * @default 0
   */
  readonly ticks?: number;
  /**
   * A shield on its icon and a grey border: it cannot be interrupted.
   *
   * @default false
   */
  readonly uninterruptible?: boolean;
  /**
   * Where it is.
   *
   * @default "casting"
   */
  readonly state?: CastState;
  /**
   * og.ui.tick() when it ended (success, interrupted, failed): it fades out after.
   *
   * @default 0
   */
  readonly stateAt?: number;
  /**
   * The latency zone at its end, a fraction of the bar (a cast can be queued in it), a pale tint of the fill.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly latency?: number;
  /**
   * The interrupted word.
   *
   * @default "Interrupted"
   */
  readonly interruptedLabel?: string;
  /**
   * The failed word.
   *
   * @default "Failed"
   */
  readonly failedLabel?: string;
  /**
   * Its width; null is 280.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default null
   */
  readonly anchor?: Anchor | null;
}

/**
 * Where a cast is.
 * - `"casting"`: Under way.
 * - `"success"`: Finished: the bar flashes and fades.
 * - `"interrupted"`: Interrupted: it turns danger, says so, and fades.
 * - `"failed"`: Failed (out of range, moved): the same as interrupted, with its own word.
 */
export type CastState = "casting" | "success" | "interrupted" | "failed";

/**
 * A conversation on screen, as og.story.view answers it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ConversationDesc {
  /**
   * What og.story.view answers this tick; null shows nothing (no conversation runs).
   *
   * @default null
   */
  readonly view?: DialogueView | null;
  /**
   * The speaker's portrait (art kind portrait): the texture the game loaded for view.speaker.portrait and its expression (og.load.texture). Null draws the style's placeholder figure for a speaker with a portrait path, and no portrait for the narrator or a speaker without one.
   *
   * @default null
   */
  readonly portrait?: Texture | null;
  /**
   * How it stands.
   *
   * @default "cinematic"
   */
  readonly layout?: ConversationLayout;
  /**
   * How fast the line types out; 0 shows it whole at once (also under reduced motion).
   *
   * @default 45
   * @range 0..
   */
  readonly charsPerSecond?: number;
  /**
   * Auto: a whole line advances by itself once its voice has ended (DialogueView.voiceLeft) and autoDelayMs has passed; choices always wait.
   *
   * @default false
   */
  readonly auto?: boolean;
  /**
   * The pause after a whole line before auto advances; 0 is 900 plus 30 per character.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly autoDelayMs?: number;
  /**
   * A check's stat in words, by its variable ({ "charm": "Persuasion" }); a stat it lacks is written as its variable name.
   *
   * @default null
   */
  readonly statNames?: unknown;
  /**
   * Skip (the back action); empty hides it.
   *
   * @default "Skip"
   */
  readonly skipLabel?: string;
  /**
   * Log (the details action); empty hides it.
   *
   * @default "Log"
   */
  readonly logLabel?: string;
  /**
   * Auto; empty hides it.
   *
   * @default "Auto"
   */
  readonly autoLabel?: string;
  /**
   * What a press does on a whole line, said to a screen reader and beside the confirm glyph.
   *
   * @default "Continue"
   */
  readonly continueLabel?: string;
  /**
   * A disabled choice's reason when the dialogue gives none.
   *
   * @default "Unavailable"
   */
  readonly unavailableLabel?: string;
  /**
   * An active check's tag before its choice ({stat}, {chance} as a percent).
   *
   * @default "{stat} · {chance}"
   */
  readonly checkLabel?: string;
  /**
   * An active check passed.
   *
   * @default "Success"
   */
  readonly passedLabel?: string;
  /**
   * An active check missed.
   *
   * @default "Failure"
   */
  readonly failedLabel?: string;
}

/**
 * How a conversation stands on the screen.
 * - `"cinematic"`: The speaker's portrait large at its side (SpeakerView.side), the line as a caption over a shade at the foot, the choices stacked on the other side: a CRPG's or an action game's conversation (Mass Effect, Baldur's Gate 3, Cyberpunk 2077).
 * - `"box"`: A visual novel's text box across the foot, the speaker's name on a plate over its top edge and the portrait standing behind it; the choices centred over the scene (Ace Attorney, Persona, Fate).
 */
export type ConversationLayout = "cinematic" | "box";

/** What the conversation's player did this tick: the game passes it on to og.story. */
export interface ConversationResult {
  /** Go on: og.story.advance (a press or confirm on a whole line with no choices, or auto). */
  readonly advanced: boolean;
  /** A choice taken: og.story.choose with this ChoiceView.index. */
  readonly chosen: number | null;
  /** Skip (or back) was pressed. */
  readonly skipped: boolean;
  /** Log was pressed: show the backlog (og.ui.backlog). */
  readonly log: boolean;
  /** Auto was pressed: flip ConversationDesc.auto. */
  readonly autoToggled: boolean;
  /** The whole line shows. */
  readonly revealed: boolean;
  /** The ChoiceView.index under the pointer or the focus. */
  readonly hovered: number | null;
}

/** What a set of frames answered. */
export interface FramesResult {
  /** The member pressed this tick (its key). */
  readonly clicked: string | null;
}

/**
 * An item of an inventory grid or a paperdoll slot, as the game's viewmodel holds it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GridItem {
  /**
   * Its key, stable while it lives: its element id (letters, digits, - and _), so its probe path is <id>/Cells/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What it is called: a screen reader's name and the tooltip's title.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * An icon of the style's set drawn on it; empty draws its art, else its kind's silhouette.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Its art (art kind item: 1×1, 1×2, 2×3, 2×4 cells), covering its cells and turning with it.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its column from the start edge, 0 first.
   *
   * @default 0
   */
  readonly x?: number;
  /**
   * Its row from the top, 0 first.
   *
   * @default 0
   */
  readonly y?: number;
  /**
   * Columns it covers lying unturned.
   *
   * @default 1
   * @range 1..
   */
  readonly w?: number;
  /**
   * Rows it covers lying unturned.
   *
   * @default 1
   * @range 1..
   */
  readonly h?: number;
  /**
   * It lies turned a quarter: it covers h columns and w rows, and its art turns with it.
   *
   * @default false
   */
  readonly rotated?: boolean;
  /**
   * How many are in its stack: shown from 2, or from 1 when it stacks.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * The most its stack holds; 0 and 1 do not stack.
   *
   * @default 0
   */
  readonly maxStack?: number;
  /**
   * Its tier: the tile's wash, its edge and its pips.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * What kind of thing it is (helmet, ring, potion): a paperdoll slot takes the tags it lists, and InventoryGridDesc.highlight lights every item of one.
   *
   * @default ""
   */
  readonly tag?: string;
  /**
   * It cannot be moved (a quest item, a bound one): a lock in its corner, and a drag or a pick-up of it shakes and is refused.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * Picked up since the player last looked: a dot in its corner (the game clears it on InventoryGridResult.hovered).
   *
   * @default false
   */
  readonly fresh?: boolean;
  /**
   * Worn or slotted elsewhere: a tick in its corner.
   *
   * @default false
   */
  readonly equipped?: boolean;
  /**
   * The player cannot use it (a class it does not fit, a level too high): a danger wash, and said so to a screen reader.
   *
   * @default false
   */
  readonly unusable?: boolean;
  /**
   * Durability or charge, 0 to 1: a thin bar along its foot, danger under a quarter.
   *
   * @default null
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly charge?: number | null;
}

/**
 * An item in the player's hand: dragged by the pointer or a finger, or picked up with confirm and moved with keys or a pad. A grid or paperdoll reports the one it holds; pass it to the others on the screen (their desc's carry) so the places that take it light up and a release over them lands it.
 */
export interface InventoryCarry {
  /** The item. */
  readonly key: string;
  /** The id of the grid or paperdoll it was picked up from. */
  readonly source: string;
  /** The paperdoll slot it was picked up from; empty when from a grid. */
  readonly slot: string;
  /** Its tag. */
  readonly tag: string;
  /** Columns it covers as carried (turned when rotated). */
  readonly w: number;
  /** Rows it covers as carried. */
  readonly h: number;
  /** It is carried turned. */
  readonly rotated: boolean;
  /** How many are carried: its whole stack, or the part split from it. */
  readonly count: number;
  /**
   * The pointer's x (for keys and pads, the centre of the place the focus is on).
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * The pointer's y.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Keys or a pad carry it: it lands where confirm is pressed, not where the pointer lets go.
   */
  readonly byFocus: boolean;
  /**
   * The pointer let it go this tick at x and y, outside the place that holds it: the grid or paperdoll under that point takes it (it is in no one's hand from the next tick).
   */
  readonly released: boolean;
}

/**
 * A grid inventory (a backpack, a stash, a chest): items of several cells, rotation, stacks and splitting, quick transfer between containers.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface InventoryGridDesc {
  /**
   * Its columns.
   *
   * @default 10
   * @range 1..
   */
  readonly columns?: number;
  /**
   * Its rows.
   *
   * @default 6
   * @range 1..
   */
  readonly rows?: number;
  /**
   * The items, each at its cell.
   *
   * @default []
   */
  readonly items?: readonly GridItem[];
  /**
   * A cell's side; 0 is 52 on a desktop and 48 on touch. Cells never go below 44 on a coarse pointer: a grid wider than its room scrolls.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cellSize?: number;
  /**
   * Cells past this many, in reading order, are locked (a bag not yet bought): drawn with a lock, taking nothing; 0 is every cell.
   *
   * @default 0
   */
  readonly capacity?: number;
  /**
   * The selected item's key: ringed in the focus ink (a details panel beside shows it).
   *
   * @default ""
   */
  readonly selected?: string;
  /**
   * A tag whose items are lit and the others dimmed (what a slot or a recipe takes); empty lights none.
   *
   * @default ""
   */
  readonly highlight?: string;
  /**
   * An item carried from another grid or a paperdoll: its footprint shows where it would land under the pointer (or the focus), and a release or confirm there answers move with its from.
   *
   * @default null
   */
  readonly carry?: InventoryCarry | null;
  /**
   * Put an item, or part of a stack, in the player's hand this tick (pass it once).
   *
   * @default null
   */
  readonly pick?: InventoryPick | null;
  /**
   * What a screen reader calls the grid (Backpack); it is also said with the cells used.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * What a screen reader calls an empty cell.
   *
   * @default "Empty"
   */
  readonly emptyLabel?: string;
  /**
   * What it calls a locked cell, and a locked item's state.
   *
   * @default "Locked"
   */
  readonly lockedLabel?: string;
  /**
   * What it says of an item the player cannot use.
   *
   * @default "Cannot use"
   */
  readonly unusableLabel?: string;
  /**
   * The touch button that turns the carried item.
   *
   * @default "Rotate"
   */
  readonly rotateLabel?: string;
}

/** What an inventory grid answered this tick. */
export interface InventoryGridResult {
  /**
   * The item pressed (a tap, a click, confirm on it while nothing is carried and it cannot be picked up): select it.
   */
  readonly selected: string | null;
  /** The item used: a double press or a double tap (drink the potion, equip the sword). */
  readonly used: string | null;
  /**
   * The item whose actions were asked for: a long press, or the details action (X, a pad's Y) on the focused item. Open its menu at menuAt (og.ui.contextMenu): split, drop, use.
   */
  readonly menu: string | null;
  /**
   * Where that menu opens: the long press's point, or the focused item's corner.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly menuAt: Vector2;
  /**
   * The item to send to the other container at once (quick transfer): a press with Shift held, or the details action held.
   */
  readonly quick: string | null;
  /**
   * A move to apply: an item placed, turned, merged, swapped or split, or one carried in from elsewhere.
   */
  readonly move: InventoryMove | null;
  /**
   * The item in the player's hand from this grid while it is carried (and on the tick it is let go outside the grid, with released): pass it to the paperdoll and the other grids on the screen.
   */
  readonly carried: InventoryCarry | null;
  /**
   * The item under the pointer or the focus: show its tooltip beside hoveredRect (og.ui.tooltipCard, og.ui.popover).
   */
  readonly hovered: string | null;
  /** Where that item is on screen. */
  readonly hoveredRect: UiRect | null;
  /**
   * A drop or a pick-up was refused this tick (no room, a locked cell or item): the place shook red and the error sound played.
   */
  readonly refused: boolean;
}

/**
 * A move the player made: apply it to the viewmodel (the item is drawn where the viewmodel says, so it stays where it was until you do).
 */
export interface InventoryMove {
  /** The item moved. */
  readonly key: string;
  /** The id of the grid or paperdoll it came from (this grid's own id for a move inside it). */
  readonly from: string;
  /** The paperdoll slot it came from; empty when from a grid. */
  readonly fromSlot: string;
  /** Its new column. */
  readonly x: number;
  /** Its new row. */
  readonly y: number;
  /** It now lies turned. */
  readonly rotated: boolean;
  /**
   * How many moved: the whole stack, or the part split from it (leave the rest where it was, and give the moved part a new key).
   */
  readonly count: number;
  /** Part of a stack moved. */
  readonly split: boolean;
  /**
   * It was dropped on a stack of the same tag with room: add count to that stack (up to its maxStack) instead of placing it; what does not fit stays where it came from.
   */
  readonly mergeInto: string | null;
  /**
   * It was dropped on one item it trades places with: that item goes where this one came from.
   */
  readonly swapWith: string | null;
}

/**
 * The game puts an item (or part of a stack) in the player's hand: after its own menu's Split 10, Take one, or a transfer it wants carried.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface InventoryPick {
  /**
   * The item.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * How many to carry; 0 is the whole stack.
   *
   * @default 0
   */
  readonly count?: number;
}

/**
 * A paperdoll: equipment slots around the character's figure, and its stats.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PaperdollDesc {
  /**
   * The slots, each placed around the figure.
   *
   * @default []
   */
  readonly slots?: readonly PaperdollSlot[];
  /**
   * The character's name over the figure; empty is none.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * A line under it (Level 42 Paladin); empty is none.
   *
   * @default ""
   */
  readonly subtitle?: string;
  /**
   * A headline number (812); empty is none.
   *
   * @default ""
   */
  readonly rating?: string;
  /**
   * What it is (Item power).
   *
   * @default ""
   */
  readonly ratingLabel?: string;
  /**
   * The stats beside the figure (under it on a phone in portrait).
   *
   * @default []
   */
  readonly stats?: readonly PaperdollStat[];
  /**
   * The stats' heading (Attributes); empty is none.
   *
   * @default ""
   */
  readonly statsTitle?: string;
  /**
   * The stats go under the figure, not beside it (a screen that puts the grids beside the doll); a phone in portrait always stacks them.
   *
   * @default false
   */
  readonly stack?: boolean;
  /**
   * An item carried from a grid: the slots that take its tag light up and the others dim; a release or confirm over one answers equip.
   *
   * @default null
   */
  readonly carry?: InventoryCarry | null;
  /**
   * The stats' deltas are shown (the game sets them from the carried or hovered item).
   *
   * @default false
   */
  readonly compare?: boolean;
  /**
   * The selected slot's key: ringed in the focus ink.
   *
   * @default ""
   */
  readonly selected?: string;
  /**
   * The figure's box, width over height; 0 is 0.62 (a standing character).
   *
   * @default 0
   * @range 0..
   */
  readonly figureAspect?: number;
  /**
   * What a screen reader adds for an empty slot.
   *
   * @default "Empty"
   */
  readonly emptyLabel?: string;
  /**
   * What it adds for a locked slot.
   *
   * @default "Locked"
   */
  readonly lockedLabel?: string;
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
}

/** An item the player put in a slot. */
export interface PaperdollEquip {
  /** The slot. */
  readonly slot: string;
  /** The item. */
  readonly key: string;
  /** The id of the grid (or this paperdoll) it came from. */
  readonly from: string;
  /** The slot it came from when moved between slots. */
  readonly fromSlot: string;
  /** The item that was in the slot: it goes where the new one came from. */
  readonly swapped: string | null;
}

/** What a paperdoll answered this tick. */
export interface PaperdollResult {
  /** An item placed in a slot: apply it. */
  readonly equip: PaperdollEquip | null;
  /** The slot pressed while nothing is carried. */
  readonly selected: string | null;
  /** The slot double pressed (take the item off at once). */
  readonly used: string | null;
  /**
   * The item dragged out of a slot (or picked up with confirm) while it is carried, and on the tick it is let go outside with released: pass it to the grids.
   */
  readonly carried: InventoryCarry | null;
  /** The slot under the pointer or the focus: show its item's tooltip beside hoveredRect. */
  readonly hovered: string | null;
  /** Where that slot is on screen. */
  readonly hoveredRect: UiRect | null;
  /**
   * The carried item was refused this tick (a slot that does not take its tag, a locked slot): the slot shook and the error sound played.
   */
  readonly refused: boolean;
}

/**
 * An equipment slot around the paperdoll's figure.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PaperdollSlot {
  /**
   * The slot's id (head, chest, mainHand): its element id.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name (Head): said by a screen reader, and shown under an empty slot on a desktop.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * The item tags it takes; empty takes any.
   *
   * @default []
   */
  readonly accept?: readonly string[];
  /**
   * What is worn there; null is empty (its w and h are not read: the item fills the slot).
   *
   * @default null
   */
  readonly item?: GridItem | null;
  /**
   * Where its centre sits across the doll's box, 0 its start edge to 1 its end.
   *
   * @default 0.5
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly x?: number;
  /**
   * Where its centre sits down the box, 0 the top to 1 the foot.
   *
   * @default 0.5
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly y?: number;
  /**
   * Its side; 0 is 64 on a desktop, 56 on a phone.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * Its width over its height (a two-handed weapon's tall slot is 0.5); 0 is square.
   *
   * @default 0
   * @range 0..
   */
  readonly aspect?: number;
  /**
   * The empty slot's silhouette, an icon of the style's set; without one (or one the set lacks) the kind its icon name, key or accepted tags say is drawn as a shape: helm, amulet, sword, shield, ring, glove, boot, armor or a relic's star, else a plain socket.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Not open yet: a lock, taking nothing.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * Why it is locked (Unlocks at level 20).
   *
   * @default ""
   */
  readonly lockedHint?: string;
}

/**
 * A line of the paperdoll's stats, with the change the carried or hovered item would make.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PaperdollStat {
  /**
   * What it is (Armor).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its value as shown (1,240).
   *
   * @default ""
   */
  readonly value?: string;
  /**
   * The change the compared item makes: positive is better, drawn with an up mark in success; negative with a down mark in danger; 0 shows none.
   *
   * @default 0
   */
  readonly delta?: number;
  /**
   * The change as shown (+48); empty writes delta.
   *
   * @default ""
   */
  readonly deltaText?: string;
  /**
   * A smaller value is better (a cooldown): flips the marks' meaning.
   *
   * @default false
   */
  readonly lowerIsBetter?: boolean;
}

/**
 * The party's frames.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PartyFramesDesc {
  /**
   * The members, each with its key.
   *
   * @default []
   */
  readonly members?: readonly UnitFrameDesc[];
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "left"
   */
  readonly anchor?: Anchor | null;
  /**
   * Between frames; 0 is the style's sm space.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
}

/**
 * A texture for an asset path a quest names (a reward's image), loaded by the game (og.load.texture).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface QuestArt {
  /**
   * The path, as QuestView names it.
   *
   * @default ""
   */
  readonly path?: string;
  /**
   * Its texture.
   *
   * @default 0
   */
  readonly texture?: Texture;
}

/**
 * The quest log, as og.story.quests answers it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface QuestLogDesc {
  /**
   * The quests, in og.story.quests' order (their start).
   *
   * @default []
   */
  readonly quests?: readonly QuestView[];
  /**
   * The quest shown in the detail (game state); empty or unknown shows the tracked one, else the first.
   *
   * @default ""
   */
  readonly selected?: string;
  /**
   * The tabs, in order; empty is one list of every quest.
   *
   * @default []
   */
  readonly tabs?: readonly QuestTab[];
  /**
   * The tab shown (game state); empty is the first.
   *
   * @default ""
   */
  readonly tab?: string;
  /**
   * Textures for the reward images the quests name; one it lacks draws the style's placeholder.
   *
   * @default []
   */
  readonly art?: readonly QuestArt[];
  /**
   * Its title.
   *
   * @default "Quests"
   */
  readonly title?: string;
  /**
   * Track.
   *
   * @default "Track"
   */
  readonly trackLabel?: string;
  /**
   * A tracked quest's toggle.
   *
   * @default "Tracked"
   */
  readonly trackedLabel?: string;
  /**
   * Show on map (a quest with a marker); empty hides it.
   *
   * @default "Show on map"
   */
  readonly mapLabel?: string;
  /**
   * The objectives' heading.
   *
   * @default "Objectives"
   */
  readonly objectivesLabel?: string;
  /**
   * The rewards' heading.
   *
   * @default "Rewards"
   */
  readonly rewardsLabel?: string;
  /**
   * An optional objective's tag.
   *
   * @default "Optional"
   */
  readonly optionalLabel?: string;
  /**
   * An objective not yet revealed.
   *
   * @default "???"
   */
  readonly hiddenLabel?: string;
  /**
   * Who gave it ({giver}).
   *
   * @default "From {giver}"
   */
  readonly giverLabel?: string;
  /**
   * A quest not looked at yet.
   *
   * @default "New"
   */
  readonly newLabel?: string;
  /**
   * The active quests' heading.
   *
   * @default "Active"
   */
  readonly activeLabel?: string;
  /**
   * The completed quests' heading and a completed quest's stamp.
   *
   * @default "Completed"
   */
  readonly completedLabel?: string;
  /**
   * A failed quest's stamp.
   *
   * @default "Failed"
   */
  readonly failedLabel?: string;
  /**
   * With no quests in the tab.
   *
   * @default "No quests yet."
   */
  readonly emptyLabel?: string;
  /**
   * On a phone, back from a quest to the list.
   *
   * @default "Back"
   */
  readonly backLabel?: string;
}

/** What the quest log's player did this tick. */
export interface QuestLogResult {
  /** A quest picked: show it (game state), and og.story.markSeen it. */
  readonly selected: string | null;
  /** Track pressed: og.story.track it, flipping QuestView.tracked. */
  readonly track: string | null;
  /** Show on map pressed: open the map on its marker. */
  readonly showOnMap: string | null;
  /** A tab picked: show it (game state). */
  readonly tab: string | null;
  /** Back was pressed at the list. */
  readonly closed: boolean;
}

/**
 * One tab of the quest log: a category of QuestView.category.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface QuestTab {
  /**
   * Its category (main, side, daily).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its words.
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * The HUD's quest tracker, fed by og.story.quests.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface QuestTrackerDesc {
  /**
   * The quests (og.story.quests): the tracked active ones show, and a tracked one that completes flashes Complete and leaves.
   *
   * @default []
   */
  readonly quests?: readonly QuestView[];
  /**
   * Where the player stands: each quest's distance to its marker. Null shows none.
   *
   * @default null
   */
  readonly from?: Vector3 | null;
  /**
   * The most shown; 0 is og.ui.objectiveTracker's.
   *
   * @default 0
   */
  readonly max?: number;
  /**
   * Its width; 0 is og.ui.objectiveTracker's.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width?: number;
  /**
   * The completed word.
   *
   * @default "Complete"
   */
  readonly completeLabel?: string;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "right"
   */
  readonly anchor?: Anchor | null;
}

/**
 * Raid frames: a grid of compact cells.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RaidFramesDesc {
  /**
   * Up to 40 members.
   *
   * @default []
   */
  readonly members?: readonly RaidMember[];
  /**
   * Members per column (groups of five, as raids are); 0 is 5.
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * A cell's width; 0 is 96.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cellWidth?: number;
  /**
   * A cell's height; 0 is 36.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cellHeight?: number;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "left"
   */
  readonly anchor?: Anchor | null;
}

/**
 * One cell of raid frames.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RaidMember {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Health.
   *
   * @default 0
   */
  readonly health?: number;
  /**
   * Its maximum.
   *
   * @default 100
   */
  readonly healthMax?: number;
  /**
   * An absorb shield.
   *
   * @default 0
   * @range 0..
   */
  readonly absorb?: number;
  /**
   * Healing on its way.
   *
   * @default 0
   * @range 0..
   */
  readonly incomingHeal?: number;
  /**
   * Its role.
   *
   * @default "none"
   */
  readonly role?: RaidRole;
  /**
   * Its class colour for the bar (a WoW raid's), null is the health colour.
   *
   * @default null
   */
  readonly classColor?: Paint | null;
  /**
   * Dead.
   *
   * @default false
   */
  readonly dead?: boolean;
  /**
   * Disconnected.
   *
   * @default false
   */
  readonly offline?: boolean;
  /**
   * Out of range: its bar and class stripe faded to 45 % and its name muted (the name keeps its outline).
   *
   * @default false
   */
  readonly outOfRange?: boolean;
  /**
   * It has the enemy's attention: a danger border.
   *
   * @default false
   */
  readonly aggro?: boolean;
  /**
   * A debuff a healer can dispel: its type (magic, curse, poison, disease) glows on the cell's border.
   *
   * @default ""
   */
  readonly dispel?: string;
  /**
   * The player's target.
   *
   * @default false
   */
  readonly selected?: boolean;
}

/**
 * A raid member's role.
 * - `"none"`: No role shown.
 * - `"tank"`: Tank: a shield mark.
 * - `"healer"`: Healer: a cross.
 * - `"damage"`: Damage: a sword.
 */
export type RaidRole = "none" | "tank" | "healer" | "damage";

/**
 * A turn order.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TurnOrderDesc {
  /**
   * The units in turn order, the current first.
   *
   * @default []
   */
  readonly units?: readonly TurnUnit[];
  /**
   * How it lays out: a strip of portraits, or a list of rows.
   *
   * @default "strip"
   */
  readonly style?: TurnOrderStyle;
  /**
   * Which way a strip runs (a list always runs down).
   *
   * @default "row"
   */
  readonly direction?: Direction;
  /**
   * A portrait's side; 0 is 52 (the current one larger). In a list, the portrait's height (its slot 3:2, the art fitted whole).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * The most shown; 0 is 8.
   *
   * @default 0
   */
  readonly max?: number;
  /**
   * The current one's word: shown under it in a strip, said with its name in both.
   *
   * @default "Now"
   */
  readonly nowLabel?: string;
  /**
   * What a screen reader says after an enemy's name in a list.
   *
   * @default "Hostile"
   */
  readonly hostileLabel?: string;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "top"
   */
  readonly anchor?: Anchor | null;
}

/**
 * How a turn order lays out its units.
 * - `"strip"`: Portraits in a strip, the current one larger and ringed with its word (the default).
 * - `"list"`: Rows of portrait and name read top to bottom (a tactics game's initiative list): the current one first in the selected row's rule, an enemy marked by a stripe in its side's ink on the start edge, no numbers.
 */
export type TurnOrderStyle = "strip" | "list";

/**
 * A unit in the turn order.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TurnUnit {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its portrait (art kind portrait, 1:1).
   *
   * @default null
   */
  readonly portrait?: Texture | null;
  /**
   * 0 the player's side (primary), else enemies (danger) by number.
   *
   * @default 0
   */
  readonly team?: number;
  /**
   * An active time gauge 0 to 1 under it (Final Fantasy's ATB).
   *
   * @default null
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly atb?: number | null;
  /**
   * Its turn now: larger, ringed, first.
   *
   * @default false
   */
  readonly current?: boolean;
  /**
   * Pushed back: a down mark.
   *
   * @default false
   */
  readonly delayed?: boolean;
  /**
   * Out of the order: greyed and crossed.
   *
   * @default false
   */
  readonly dead?: boolean;
}

/**
 * A unit's frame: portrait, name, bars.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface UnitFrameDesc {
  /**
   * Its key in party frames (and its probe id there).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its level (60, ??); empty is none.
   *
   * @default ""
   */
  readonly level?: string;
  /**
   * A line under the name (a guild, a class).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * Its portrait (art kind portrait, 1:1).
   *
   * @default null
   */
  readonly portrait?: Texture | null;
  /**
   * A class or role icon of the atlas.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Health.
   *
   * @default 0
   */
  readonly health?: number;
  /**
   * Its maximum.
   *
   * @default 100
   */
  readonly healthMax?: number;
  /**
   * An absorb shield over the health.
   *
   * @default 0
   * @range 0..
   */
  readonly shield?: number;
  /**
   * A resource bar's value (mana, rage, energy); null draws none.
   *
   * @default null
   */
  readonly resource?: number | null;
  /**
   * Its maximum.
   *
   * @default 100
   */
  readonly resourceMax?: number;
  /**
   * The resource's own colour instead of its kind's (rage red, energy yellow).
   *
   * @default null
   */
  readonly resourceColor?: Paint | null;
  /**
   * Its colour.
   *
   * @default "mana"
   */
  readonly resourceKind?: MeterKind;
  /**
   * What it is casting; empty is nothing.
   *
   * @default ""
   */
  readonly castName?: string;
  /**
   * How far the cast is, 0 to 1.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly castProgress?: number;
  /**
   * Buffs and debuffs, small under the bars.
   *
   * @default []
   */
  readonly effects?: readonly StatusEffect[];
  /**
   * An enemy: its name and bar in danger.
   *
   * @default false
   */
  readonly hostile?: boolean;
  /**
   * Elite, rare or boss.
   *
   * @default "normal"
   */
  readonly unitClass?: UnitClass;
  /**
   * The player's target: a highlight ring.
   *
   * @default false
   */
  readonly targeted?: boolean;
  /**
   * Dead: grey, a skull.
   *
   * @default false
   */
  readonly dead?: boolean;
  /**
   * Disconnected: the portrait and bar greyed.
   *
   * @default false
   */
  readonly offline?: boolean;
  /**
   * In combat: crossed swords by the level.
   *
   * @default false
   */
  readonly combat?: boolean;
  /**
   * A party's leader: a crown.
   *
   * @default false
   */
  readonly leader?: boolean;
  /**
   * How large.
   *
   * @default "large"
   */
  readonly size?: UnitFrameSize;
  /**
   * Mirrored: the portrait at the end (a target frame).
   *
   * @default false
   */
  readonly reverse?: boolean;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default null
   */
  readonly anchor?: Anchor | null;
}

/**
 * A unit frame's size.
 * - `"large"`: The player's or the target's frame (the default).
 * - `"medium"`: A party member's.
 * - `"small"`: A target-of-target's or a pet's.
 */
export type UnitFrameSize = "large" | "medium" | "small";
