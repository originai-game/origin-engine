// The racing kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.

import type { Anchor, Paint, PickedColor, Rarity, Texture, Vector2 } from "@origingame/engine";

/**
 * An arcade racer's running drift score.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DriftScoreDesc {
  /**
   * The chain's score so far; 0 with nothing banked hides it.
   *
   * @default 0
   * @range 0..
   */
  readonly score?: number;
  /**
   * The chain's multiplier (x2, x3 …); 1 or less hides its badge.
   *
   * @default 0
   * @range 0..
   */
  readonly multiplier?: number;
  /**
   * og.ui.tick() when the chain was banked: the score ticks up into the total for 0.8 s and leaves, and the chain stays over until bankedAt is 0 again (a new chain); 0 is a running chain.
   *
   * @default 0
   */
  readonly bankedAt?: number;
  /**
   * og.ui.tick() when the chain broke (a crash): the score shakes and fades in danger for 0.8 s, and the chain stays over until failedAt is 0 again; 0 is none.
   *
   * @default 0
   */
  readonly failedAt?: number;
  /**
   * The banked total the chain ticks up into.
   *
   * @default 0
   */
  readonly total?: number;
  /**
   * A screen point (CSS px) it floats over, such as the car's; null builds it at its anchor.
   *
   * @default null
   */
  readonly at?: Vector2 | null;
  /**
   * The chain's caption.
   *
   * @default "Drift"
   */
  readonly label?: string;
  /**
   * Opens its own anchored group there when at is null.
   *
   * @default "top"
   */
  readonly anchor?: Anchor | null;
}

/**
 * One category of the garage's rail (Engine, Turbo, Tyres; Livery).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GarageCategory {
  /**
   * Its key: its rail entry's element id, so its probe path is <id>/Body/Rail/<key> (<id>/Rail/Chips/<key> on a phone in portrait).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name beside its art.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its art (art kind itemIcon), a part of the kind; null draws the style's placeholder mark.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * A line under the name or at the list's head (4 / 8 owned); empty shows none.
   *
   * @default ""
   */
  readonly count?: string;
  /**
   * The livery editor: the parts list becomes the car's layers and the side the selected layer's colour.
   *
   * @default false
   */
  readonly livery?: boolean;
}

/**
 * A garage: the categories, the selected category's parts, the car's figures, the livery's layers.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GarageDesc {
  /**
   * The screen's title (Garage); empty shows none (the game titles it).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The purse the parts are priced in.
   *
   * @default 0
   */
  readonly balance?: number;
  /**
   * The currency's art beside the balance and on prices; null draws a coin.
   *
   * @default null
   */
  readonly currencyImage?: Texture | null;
  /**
   * The rail, top to bottom.
   *
   * @default []
   */
  readonly categories?: readonly GarageCategory[];
  /**
   * The selected category's key.
   *
   * @default ""
   */
  readonly category?: string;
  /**
   * The selected category's parts.
   *
   * @default []
   */
  readonly parts?: readonly GaragePart[];
  /**
   * The part whose detail and preview are shown (the game's: it passes GarageResult.selected back).
   *
   * @default ""
   */
  readonly selected?: string;
  /**
   * The car (art kind vehicle, a cut-out) standing on its turntable in the middle; null leaves the middle to the game's own view (og.ui.sceneViewport built under the garage).
   *
   * @default null
   */
  readonly vehicle?: Texture | null;
  /**
   * The car's figures, each a bar of its value with the preview's change lit.
   *
   * @default []
   */
  readonly stats?: readonly GarageStat[];
  /**
   * The car's class letter (A, S1) in its badge; empty shows no badge.
   *
   * @default ""
   */
  readonly ratingClass?: string;
  /**
   * The car's rating beside the class (742).
   *
   * @default 0
   */
  readonly rating?: number;
  /**
   * The rating with the selected part fitted; 0 shows none.
   *
   * @default 0
   */
  readonly ratingPreview?: number;
  /**
   * The livery's layers, for a livery category.
   *
   * @default []
   */
  readonly layers?: readonly LiveryLayer[];
  /**
   * The selected layer's key.
   *
   * @default ""
   */
  readonly layer?: string;
  /**
   * The colour picker's swatches (`#rrggbb`).
   *
   * @default []
   */
  readonly swatches?: readonly string[];
  /**
   * The figures' heading.
   *
   * @default "Performance"
   */
  readonly performanceLabel?: string;
  /**
   * The rating's caption.
   *
   * @default "Class"
   */
  readonly classLabel?: string;
  /**
   * The legend's word for the value now.
   *
   * @default "Current"
   */
  readonly currentLabel?: string;
  /**
   * The legend's word for the change.
   *
   * @default "Preview"
   */
  readonly previewLabel?: string;
  /**
   * The action on an owned part.
   *
   * @default "Equip"
   */
  readonly equipLabel?: string;
  /**
   * An equipped part's word.
   *
   * @default "Equipped"
   */
  readonly equippedLabel?: string;
  /**
   * The action on a part to buy.
   *
   * @default "Buy"
   */
  readonly buyLabel?: string;
  /**
   * A locked part's word.
   *
   * @default "Locked"
   */
  readonly lockedLabel?: string;
  /**
   * A free part's price.
   *
   * @default "Free"
   */
  readonly freeLabel?: string;
  /**
   * Said after a price the purse cannot pay.
   *
   * @default "not enough"
   */
  readonly shortLabel?: string;
  /**
   * The layers list's heading.
   *
   * @default "Layers"
   */
  readonly layersLabel?: string;
  /**
   * The colour picker's heading.
   *
   * @default "Colour"
   */
  readonly colorLabel?: string;
  /**
   * The action adding a layer.
   *
   * @default "Add layer"
   */
  readonly addLayerLabel?: string;
  /**
   * The action deleting the selected layer.
   *
   * @default "Delete"
   */
  readonly deleteLabel?: string;
  /**
   * Moving a layer up, to a screen reader.
   *
   * @default "Move up"
   */
  readonly upLabel?: string;
  /**
   * Moving a layer down, to a screen reader.
   *
   * @default "Move down"
   */
  readonly downLabel?: string;
  /**
   * A layer's eye, to a screen reader.
   *
   * @default "Visible"
   */
  readonly visibleLabel?: string;
}

/**
 * One part on offer in the selected category, as the game holds it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GaragePart {
  /**
   * Its key: its row's element id, so its probe path is <id>/Body/Parts/List/<key> (<id>/Body/Parts/<key> on a phone in portrait).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name, the row's reading.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its art (art kind itemIcon) on the row's tile; null draws the style's placeholder.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its tier: the tile's rim and the detail's tier word.
   *
   * @default "none"
   */
  readonly rarity?: Rarity;
  /**
   * The tier's word in the detail (Rare); empty shows none.
   *
   * @default ""
   */
  readonly rarityLabel?: string;
  /**
   * Its upgrade level, as lit pips.
   *
   * @default 0
   */
  readonly tier?: number;
  /**
   * The pips in all; 0 is 3.
   *
   * @default 0
   */
  readonly tierMax?: number;
  /**
   * Its price in the garage's currency; 0 is free.
   *
   * @default 0
   */
  readonly price?: number;
  /**
   * Owned: no price; its action is Equip.
   *
   * @default false
   */
  readonly owned?: boolean;
  /**
   * On the car now: a tick and the equipped word; its action is inert.
   *
   * @default false
   */
  readonly equipped?: boolean;
  /**
   * Not available yet: dimmed with a lock and lockedHint in place of the price; pressing it is refused.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * Why it is locked (Reach driver level 30).
   *
   * @default ""
   */
  readonly lockedHint?: string;
  /**
   * The detail's paragraph.
   *
   * @default ""
   */
  readonly description?: string;
}

/** What the player asked for this tick, for the game to apply. */
export interface GarageResult {
  /** Another category was pressed: show its parts. */
  readonly category: string | null;
  /** A part was pressed or focused: show its detail and preview. */
  readonly selected: string | null;
  /** Equip was pressed on an owned part. */
  readonly equip: string | null;
  /** Buy was pressed on a part the purse pays (the game charges it and fits it). */
  readonly buy: string | null;
  /** A locked part, or one the purse cannot pay, was asked for: it shook. */
  readonly refused: string | null;
  /** A layer was pressed: select it. */
  readonly layer: string | null;
  /** The selected layer's colour was changed in the picker. */
  readonly layerColor: PickedColor | null;
  /** A layer's up or down was pressed. */
  readonly layerMove: LayerMove | null;
  /** A layer's eye was pressed: show or hide it. */
  readonly layerVisible: string | null;
  /** Add layer was pressed. */
  readonly addLayer: boolean;
  /** Delete was pressed on the selected layer. */
  readonly deleteLayer: string | null;
}

/**
 * One of the car's performance figures, now and with the selected part fitted.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GarageStat {
  /**
   * Its key.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name (Speed).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its value now.
   *
   * @default 0
   */
  readonly value?: number;
  /**
   * Its value with the selected part fitted; a negative value shows none.
   *
   * @default -1
   */
  readonly preview?: number;
  /**
   * The bar's whole; 0 is 10.
   *
   * @default 0
   * @range 0..
   */
  readonly max?: number;
}

/**
 * An arcade racer's held item.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ItemSlotDesc {
  /**
   * The item held; null is an empty slot.
   *
   * @default null
   */
  readonly item?: Texture | null;
  /**
   * The item's region of its texture, [u0, v0, u1, v1]; null is the whole texture.
   *
   * @default null
   */
  readonly uv?: [number, number, number, number] | null;
  /**
   * The next item, in the smaller slot; null hides that slot.
   *
   * @default null
   */
  readonly queued?: Texture | null;
  /**
   * The next item's region.
   *
   * @default null
   */
  readonly queuedUv?: [number, number, number, number] | null;
  /**
   * What a roulette cycles through after a pickup; empty spins the item itself.
   *
   * @default []
   */
  readonly roulette?: readonly Texture[];
  /**
   * og.ui.tick() of the pickup: from then it cycles the roulette, slowing, for rollMs, then lands on the item with a pop; 0 is no roulette.
   *
   * @default 0
   */
  readonly rollingAt?: number;
  /**
   * How long the roulette spins.
   *
   * @default 1500
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly rollMs?: number;
  /**
   * The main slot's side; 0 is 84 (64 on a phone).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * What a screen reader calls the item.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "topLeft"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A racing lap and split block.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LapTimerDesc {
  /**
   * The player's place, from 1; 0 hides the place.
   *
   * @default 0
   */
  readonly position?: number;
  /**
   * Cars in the race.
   *
   * @default 0
   */
  readonly positionTotal?: number;
  /**
   * The lap the player is on, from 1; 0 hides the lap.
   *
   * @default 0
   */
  readonly lap?: number;
  /**
   * Laps in the race; 0 is open-ended (no total, no final lap).
   *
   * @default 0
   */
  readonly lapTotal?: number;
  /**
   * The current lap's time so far.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly currentMs?: number;
  /**
   * The player's best lap; null is none yet.
   *
   * @default null
   * @unit ms (milliseconds)
   */
  readonly bestMs?: number | null;
  /**
   * The last lap's time; null is none yet.
   *
   * @default null
   * @unit ms (milliseconds)
   */
  readonly lastMs?: number | null;
  /**
   * The split at the last sector against the best lap, signed (negative: ahead, in success; positive: behind, in danger); null hides the split.
   *
   * @default null
   * @unit ms (milliseconds)
   */
  readonly deltaMs?: number | null;
  /**
   * The place's caption.
   *
   * @default "Position"
   */
  readonly positionLabel?: string;
  /**
   * The lap's caption.
   *
   * @default "Lap"
   */
  readonly lapLabel?: string;
  /**
   * The lap's caption on the final lap.
   *
   * @default "Final lap"
   */
  readonly finalLabel?: string;
  /**
   * The current lap's caption.
   *
   * @default "Current lap"
   */
  readonly currentLabel?: string;
  /**
   * The best lap's caption.
   *
   * @default "Best lap"
   */
  readonly bestLabel?: string;
  /**
   * The last lap's caption.
   *
   * @default "Last lap"
   */
  readonly lastLabel?: string;
  /**
   * The split's caption.
   *
   * @default "Split"
   */
  readonly splitLabel?: string;
  /**
   * This lap's sectors, as pips under the lap counter; empty shows none.
   *
   * @default []
   */
  readonly sectors?: readonly SectorState[];
  /**
   * The split is a new best: purple instead of success.
   *
   * @default false
   */
  readonly splitBest?: boolean;
  /**
   * No plate: its text sits on the scene with an outline and a shadow (the Need for Speed rule).
   *
   * @default false
   */
  readonly boxless?: boolean;
  /**
   * Which part it builds: two calls (head at the top left, times at the top centre) anchor them apart.
   *
   * @default "all"
   */
  readonly show?: LapTimerShow;
  /**
   * The arcade look's place: a huge ordinal (positionText) with a bouncy pop when it changes, over its total, in place of the place's counter.
   *
   * @default false
   */
  readonly placeBadge?: boolean;
  /**
   * The place as the game writes it in its language (1st, 1er, 1位) for placeBadge; empty is the number.
   *
   * @default ""
   */
  readonly positionText?: string;
  /**
   * The times as a racing HUD's top centre shows them: the running lap large with the split beside it and the best lap under it at caption size (no last lap, no table).
   *
   * @default false
   */
  readonly compact?: boolean;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "topLeft"
   */
  readonly anchor?: Anchor | null;
}

/**
 * Which part of the lap block a call builds, so the place and lap and the times can be anchored apart.
 * - `"all"`: The whole block.
 * - `"head"`: The place, the lap and its sector pips.
 * - `"times"`: The current, best and last laps and the split.
 */
export type LapTimerShow = "all" | "head" | "times";

/** A layer asked to move in the stack. */
export interface LayerMove {
  /** The layer. */
  readonly key: string;
  /** Up (−1, towards the top) or down (+1). */
  readonly by: number;
}

/**
 * One layer of the car's livery, topmost first.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LiveryLayer {
  /**
   * Its key: its row's element id, so its probe path is <id>/Body/Layers/List/<key> (<id>/Body/<key> on a phone in portrait).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name (Stripe, Number 7).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * What it is, the row's second line (Decal, Shape, Text).
   *
   * @default ""
   */
  readonly kind?: string;
  /**
   * Its colour: the row's swatch, and the picker's while it is selected.
   *
   * @default {}
   */
  readonly color?: PickedColor;
  /**
   * Shown on the car; a hidden layer's row is dimmed and its eye is shut.
   *
   * @default true
   */
  readonly visible?: boolean;
  /**
   * Not editable: no colour, no moves (a manufacturer's badge).
   *
   * @default false
   */
  readonly locked?: boolean;
}

/**
 * The race banner's moment.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RaceBannerDesc {
  /**
   * Which moment.
   *
   * @default "none"
   */
  readonly event?: RaceEvent;
  /**
   * og.ui.tick() when it started: a moment plays from then (a later tick plays it again), so a still at a tick is always the same.
   *
   * @default 0
   */
  readonly at?: number;
  /**
   * countdown: the numeral it starts at, and the start lights' count.
   *
   * @default 3
   * @range 1..9
   */
  readonly count?: number;
  /**
   * checkpoint: the split against the best, signed (negative: ahead, in success; positive: behind, in danger).
   *
   * @default 0
   * @unit ms (milliseconds)
   */
  readonly deltaMs?: number;
  /**
   * checkpoint: a new best split, in purple.
   *
   * @default false
   */
  readonly best?: boolean;
  /**
   * checkpoint: its number, from 1, for the label's {n}.
   *
   * @default 0
   */
  readonly index?: number;
  /**
   * finish: the player's place, from 1.
   *
   * @default 0
   */
  readonly place?: number;
  /**
   * finish: the place as the game writes it in its language (1st, 1er, 1位); empty is the number.
   *
   * @default ""
   */
  readonly placeText?: string;
  /**
   * finish: the race time, written m:ss.mmm under the place.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly timeMs?: number;
  /**
   * checkpoint and finalLap: the element (an id or path, as og.ui.bounds takes it) the moment sits under, centred, clear of its last-frame bounds: the lap timer's times. Empty: a checkpoint centre-top at a ninth of the height, the final lap at a fifth.
   *
   * @default ""
   */
  readonly below?: string;
  /**
   * The player drives the wrong way: a danger banner with a U-turn arrow, pulsing once a second, over every other moment while it holds.
   *
   * @default false
   */
  readonly wrongWay?: boolean;
  /**
   * The start's last word.
   *
   * @default "GO!"
   */
  readonly goLabel?: string;
  /**
   * The checkpoint's caption; {n} is its index.
   *
   * @default "Checkpoint {n}"
   */
  readonly checkpointLabel?: string;
  /**
   * The wrong way's words.
   *
   * @default "Wrong way"
   */
  readonly wrongWayLabel?: string;
  /**
   * The final lap's words.
   *
   * @default "Final lap"
   */
  readonly finalLapLabel?: string;
  /**
   * The finish's caption over the place.
   *
   * @default "Finish"
   */
  readonly finishLabel?: string;
}

/**
 * A race's moment the race banner shows.
 * - `"none"`: Nothing (a wrong way still shows).
 * - `"countdown"`: The start: count numerals one a second from `at`, a start light filling danger with each, then GO with every light green.
 * - `"go"`: GO alone, every light green, for a second.
 * - `"checkpoint"`: A checkpoint's split (deltaMs) centre-top under the lap time, held 1.5 s.
 * - `"finalLap"`: A thin gold band with the final lap's label sweeping across the top third, 2 s.
 * - `"finish"`: A checker wipe, then the place huge with the time under it, 2.5 s.
 */
export type RaceEvent = "none" | "countdown" | "go" | "checkpoint" | "finalLap" | "finish";

/**
 * A rear-view mirror.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RearViewDesc {
  /**
   * What the mirror shows: a render target's texture (og.render.renderTarget, RenderTargetInfo.texture) a CameraView looking back draws into, at about the mirror's aspect; null draws the style's dark glass.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * Flips it left to right, as a mirror shows the road behind.
   *
   * @default true
   */
  readonly mirror?: boolean;
  /**
   * Its width; 0 is a third of the HUD band's width (two fifths on a phone held sideways, two thirds upright), at most 560.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width?: number;
  /**
   * Its width over its height.
   *
   * @default 4.2
   * @range 1.5..
   */
  readonly aspect?: number;
  /**
   * What a screen reader calls it.
   *
   * @default "Rear view"
   */
  readonly label?: string;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "top"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A lap's sector as timing screens colour it.
 * - `"none"`: Not driven yet this lap.
 * - `"slower"`: Slower than the player's best (yellow, the warn role).
 * - `"personal"`: The player's best sector (green, the success role).
 * - `"best"`: The best sector of anyone (purple).
 */
export type SectorState = "none" | "slower" | "personal" | "best";

/**
 * A speedometer's look.
 * - `"dial"`: A dial with a needle and the tach round it (the default).
 * - `"digital"`: The speed in large digits over a tach bar.
 * - `"arcade"`: The arcade racing look (kart and street racers): the speed as a big bouncy numeral under a fat curved nitro tube (boost) cut into boostSegments, which flares when full.
 * - `"tach"`: A 240° tach arc driven by rpm with its numbers outside it, a shift-light row, and the speed as the largest numeral (no needle) with the gear in a badge beside it.
 */
export type SpeedoStyle = "dial" | "digital" | "arcade" | "tach";

/**
 * A speedometer and tachometer.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SpeedometerDesc {
  /**
   * The speed.
   *
   * @default 0
   */
  readonly speed?: number;
  /**
   * The dial's top.
   *
   * @default 300
   */
  readonly maxSpeed?: number;
  /**
   * The speed's unit.
   *
   * @default "km/h"
   */
  readonly unit?: string;
  /**
   * Engine speed, 0 to 1 of the tach.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly rpm?: number;
  /**
   * Where the red zone starts.
   *
   * @default 0.85
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly redline?: number;
  /**
   * The tach's top in revolutions per minute (tach: its numbers are thousands, every rpmStep).
   *
   * @default 8000
   * @range 0..
   */
  readonly maxRpm?: number;
  /**
   * The revs between the tach's numbered ticks.
   *
   * @default 1000
   * @range 0..
   */
  readonly rpmStep?: number;
  /**
   * arcade: the nitro tube's segments, each filled whole.
   *
   * @default 3
   * @range 1..8
   */
  readonly boostSegments?: number;
  /**
   * Where the shift lights are all lit and flash, 0 to 1 of the tach; 0 is the redline.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly shiftAt?: number;
  /**
   * The gear (N, R, 1 …); empty is none.
   *
   * @default ""
   */
  readonly gear?: string;
  /**
   * A boost or nitro gauge 0 to 1.
   *
   * @default null
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly boost?: number | null;
  /**
   * Its look.
   *
   * @default "dial"
   */
  readonly style?: SpeedoStyle;
  /**
   * Its diameter; 0 is 180 (132 on a phone).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "bottomRight"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A car in the standings, in race order.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StandingsEntry {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * The driver's name (K. Moreno).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its team or car colour, a stripe by its position; null is the team colour of its index.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * Seconds behind the leader, or with StandingsTowerDesc.relative behind the player (negative: ahead).
   *
   * @default 0
   * @unit s (seconds of simulation time)
   */
  readonly gap?: number;
  /**
   * Laps behind the leader; more than 0 shows +<n> L in place of the gap.
   *
   * @default 0
   */
  readonly lapsDown?: number;
  /**
   * Places gained since the start (positive) or lost (negative): a small arrow by its position; 0 is none.
   *
   * @default 0
   */
  readonly change?: number;
  /**
   * The player's car: its row lit.
   *
   * @default false
   */
  readonly player?: boolean;
  /**
   * In the pit lane.
   *
   * @default false
   */
  readonly pit?: boolean;
  /**
   * Retired: dimmed at the foot, its gap the out label.
   *
   * @default false
   */
  readonly out?: boolean;
  /**
   * Holds the fastest lap: a small purple chip by its name (StandingsTowerDesc.fastestLabel).
   *
   * @default false
   */
  readonly fastest?: boolean;
}

/**
 * A racing standings tower.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StandingsTowerDesc {
  /**
   * The cars in race order, the leader first.
   *
   * @default []
   */
  readonly entries?: readonly StandingsEntry[];
  /**
   * Gaps are to the player (signed, ahead negative) rather than to the leader.
   *
   * @default false
   */
  readonly relative?: boolean;
  /**
   * The most rows shown; 0 is 10 (5 on a phone). Past it the tower shows the leaders and the three cars around the player, a break between.
   *
   * @default 0
   */
  readonly rows?: number;
  /**
   * A relative window (the racing HUD's): only the player and this many cars ahead of it (at most 1 on a phone), with behind. With ahead or behind above 0 the gaps are intervals in neutral ink, the player's row takes the accent fill with no gap and no place (the lap block has it), and rows is not used. A window at the front or the back of the field keeps its length by showing more on the other side.
   *
   * @default 0
   */
  readonly ahead?: number;
  /**
   * The relative window's cars behind the player (at most 1 on a phone).
   *
   * @default 0
   */
  readonly behind?: number;
  /**
   * The leader's gap (with relative off).
   *
   * @default "Leader"
   */
  readonly leaderLabel?: string;
  /**
   * The gap of a car in the pit lane.
   *
   * @default "PIT"
   */
  readonly pitLabel?: string;
  /**
   * The gap of a retired car.
   *
   * @default "OUT"
   */
  readonly outLabel?: string;
  /**
   * The fastest lap's chip.
   *
   * @default "FL"
   */
  readonly fastestLabel?: string;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "left"
   */
  readonly anchor?: Anchor | null;
}
