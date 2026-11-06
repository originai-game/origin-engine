import type { AbilitySlotDesc, Anchor, ChartKind, ChartSeries, MinimapViewDesc, NumberFormat, NumberFormatDesc, Paint, Texture, Vector2 } from "@origingame/engine";
// The strategy kit's data types, moved from the engine's generated og.ui
// declarations (docs/ui.md §17). Edit them with the components that read them.



/**
 * An RTS command card.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CommandCardDesc {
  /**
   * The commands in reading order, each one's key its id; an empty key leaves its cell empty.
   *
   * @default []
   */
  readonly commands?: readonly AbilitySlotDesc[];
  /**
   * Columns; 0 is 5.
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * A cell's side; 0 is 56 (48 on a phone).
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
 * One page of an RTS command card: the unit's own commands, or a sub-card (Build, Advanced build) a command opens.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CommandPage {
  /**
   * Its key: a command whose key is a page's key opens that page.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its commands in reading order (empty keys leave cells empty). A command's `action` names the UI action (og.ui.defineAction, on physical key codes) whose press fires it, and its glyph is the hotkey the card prints.
   *
   * @default []
   */
  readonly commands?: readonly AbilitySlotDesc[];
}

/**
 * A control group's tab over the selection.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ControlGroup {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What its tab says (1).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Units in it; 0 draws an empty tab.
   *
   * @default 0
   */
  readonly count?: number;
}

/**
 * An RTS drag selection.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DragSelectDesc {
  /**
   * It follows the pointer: a press on the scene and a drag past the threshold draws the box.
   *
   * @default true
   */
  readonly enabled?: boolean;
  /**
   * Where the box starts, in the surface's pixels, instead of the press (a scripted or pad selection).
   *
   * @default null
   */
  readonly from?: Vector2 | null;
  /**
   * Where it ends, with from.
   *
   * @default null
   */
  readonly to?: Vector2 | null;
  /**
   * How far a press travels before it is a drag, not a click.
   *
   * @default 6
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly threshold?: number;
  /**
   * Its colour; null is the style's highlight.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * Element paths (HUD roots such as ConsoleHud) a press inside which is theirs, never the start of a box.
   *
   * @default []
   */
  readonly exclude?: readonly string[];
}

/** What a drag selection answered. */
export interface DragSelectResult {
  /** The box is drawn this tick. */
  readonly dragging: boolean;
  /** The box was let go this tick: select what lies inside x, y, width and height. */
  readonly released: boolean;
  /**
   * The box's left.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Its top.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Its width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Its height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
}

/**
 * A building's production queue.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ProductionQueueDesc {
  /**
   * In build order, the one being built first.
   *
   * @default []
   */
  readonly items?: readonly QueueItem[];
  /**
   * Slots shown (empty ones dim); 0 is 5.
   *
   * @default 0
   */
  readonly slots?: number;
  /**
   * Production is held (no supply, no power): the bar and its time in warn.
   *
   * @default false
   */
  readonly paused?: boolean;
  /**
   * What the held bar says.
   *
   * @default "Paused"
   */
  readonly pausedLabel?: string;
  /**
   * A queued slot's side; 0 is 40 (36 on a phone).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * Opens its own anchored group there; null builds it in place (inside a selection panel).
   *
   * @default null
   */
  readonly anchor?: Anchor | null;
}

/** What a production queue answered. */
export interface ProductionQueueResult {
  /** The item pressed this tick to cancel it (its key). */
  readonly cancelled: string | null;
}

/**
 * Something in a production queue.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface QueueItem {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * What it makes (Vanguard Marine).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * An icon of the atlas when it has no art.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Its art (art kind portrait or ability, 1:1).
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Seconds left; only the first item counts down.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly remaining?: number;
  /**
   * Its build time; the first item's bar shows remaining over it.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly duration?: number;
  /**
   * How many of it are queued in this slot; over 1 shows a small count.
   *
   * @default 0
   */
  readonly count?: number;
}

/**
 * A resource bar.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ResourceBarDesc {
  /**
   * The resources.
   *
   * @default []
   */
  readonly items?: readonly ResourceItem[];
  /**
   * Show rates (the default).
   *
   * @default true
   */
  readonly rates?: boolean;
  /**
   * A game clock at the plate's end, in mono (12:34), after a hairline; null is none.
   *
   * @default null
   * @unit s (seconds of simulation time)
   */
  readonly clock?: number | null;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "top"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A resource on a resource bar.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ResourceItem {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its icon.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * What it is (what a screen reader says).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * How much.
   *
   * @default 0
   */
  readonly amount?: number;
  /**
   * Its rate (per second or per turn): + in success, − in danger with a mark.
   *
   * @default 0
   */
  readonly rate?: number;
  /**
   * After the rate.
   *
   * @default "/s"
   */
  readonly rateSuffix?: string;
  /**
   * A cap: shown as amount / capacity, warn near it, danger at it (supply).
   *
   * @default null
   */
  readonly capacity?: number | null;
  /**
   * How its numbers are written; the default is compact.
   *
   * @default {}
   */
  readonly format?: NumberFormatDesc;
}

/**
 * The RTS command layer: the console to draw, the command pages, the control groups' keys and a command awaiting its target.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RtsCommandsDesc {
  /**
   * The console (strategy.rtsConsole): its map, its selection with the control groups' tabs, and its command card (whose commands the current page replaces).
   *
   * @default {}
   */
  readonly console?: RtsConsoleDesc;
  /**
   * The command pages; the first is the root.
   *
   * @default []
   */
  readonly pages?: readonly CommandPage[];
  /**
   * The page shown; empty is the root.
   *
   * @default ""
   */
  readonly page?: string;
  /**
   * The UI actions recalling control groups, in the order of the selection's groups (Digit1 … Digit0 bound by the game): with Ctrl held one sets the group to the selection, with Shift held adds the selection to it, and tapped twice quickly focuses it.
   *
   * @default []
   */
  readonly groupActions?: readonly string[];
  /**
   * A command waiting for its target (its key): its slot is lit, a prompt says so, and Back cancels it.
   *
   * @default ""
   */
  readonly targeting?: string;
  /**
   * Two taps of a group's key within this long focus it.
   *
   * @default 350
   * @unit ms (milliseconds)
   */
  readonly doubleTapMs?: number;
  /**
   * The targeting prompt ({command} is its name).
   *
   * @default "{command}: choose a target"
   */
  readonly targetLabel?: string;
  /**
   * The prompt's cancel word, after the back glyph.
   *
   * @default "Cancel"
   */
  readonly cancelLabel?: string;
  /**
   * The prompt's hint for queueing, after the Shift glyph.
   *
   * @default "Queue"
   */
  readonly queueLabel?: string;
  /**
   * The Back command a sub-page ends with.
   *
   * @default "Back"
   */
  readonly backLabel?: string;
}

/** What the player asked for this tick, for the game to carry out. */
export interface RtsCommandsResult {
  /**
   * A command fired (pressed, or its action): its key. A command naming a page opens the page instead.
   */
  readonly command: string | null;
  /** Shift was held as it fired (the order goes to the end of the units' queues). */
  readonly queued: boolean;
  /**
   * A command that cannot fire now (locked, disabled, unaffordable or cooling down) was asked for: its key (the game says why: not enough gold).
   */
  readonly refused: string | null;
  /** A page was opened, or Back (the cell or the action) went to the root: its key. */
  readonly page: string | null;
  /** Back with a command waiting for its target: cancel it. */
  readonly cancelTarget: boolean;
  /** A control group's key or tab: select it. */
  readonly recall: string | null;
  /** A control group recalled twice quickly: centre the camera on it. */
  readonly focus: string | null;
  /** Ctrl and a group's key: the group becomes the selection. */
  readonly assign: string | null;
  /** Shift and a group's key: the selection joins the group. */
  readonly add: string | null;
  /** A unit's tile pressed: focus it (or select only it). */
  readonly clicked: string | null;
  /** Ctrl and a unit's tile: select every unit of its type in the selection. */
  readonly selectType: string | null;
  /** Shift and a unit's tile: take it out of the selection. */
  readonly deselect: string | null;
  /** The minimap was pressed. */
  readonly mapClicked: boolean;
  /** A production queue item pressed: cancel it. */
  readonly cancelled: string | null;
}

/**
 * An RTS bottom console.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RtsConsoleDesc {
  /**
   * The minimap zone at its start; null is none.
   *
   * @default null
   */
  readonly map?: MinimapViewDesc | null;
  /**
   * The selection and unit card zones (its anchor is ignored).
   *
   * @default {}
   */
  readonly selection?: SelectionPanelDesc;
  /**
   * The command card zone at its end (its anchor is ignored).
   *
   * @default {}
   */
  readonly commands?: CommandCardDesc;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "bottom"
   */
  readonly anchor?: Anchor | null;
}

/** What an RTS console answered. */
export interface RtsConsoleResult {
  /** The minimap was pressed this tick (MinimapResult.clicked). */
  readonly mapClicked: boolean;
  /** A unit's tile pressed this tick (its key). */
  readonly clicked: string | null;
  /** A control group's tab pressed this tick (its key). */
  readonly group: string | null;
  /** A queue item pressed this tick (its key). */
  readonly cancelled: string | null;
  /** A command pressed and released this tick (its key). */
  readonly command: string | null;
  /** The commands held. */
  readonly pressed: readonly string[];
}

/**
 * A unit or building in an RTS selection.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SelectedUnit {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name (Vanguard Marine).
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its type line under the name (Infantry).
   *
   * @default ""
   */
  readonly kind?: string;
  /**
   * An icon of the atlas when it has no art.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Its portrait (art kind portrait, 1:1); null draws the icon on a gradient in a hue of its own.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its health.
   *
   * @default 0
   * @range 0..
   */
  readonly health?: number;
  /**
   * Its most health; 0 hides the bar.
   *
   * @default 0
   * @range 0..
   */
  readonly healthMax?: number;
  /**
   * Its shield, over the health as a thin blue bar.
   *
   * @default 0
   * @range 0..
   */
  readonly shield?: number;
  /**
   * Its most shield; 0 is none.
   *
   * @default 0
   * @range 0..
   */
  readonly shieldMax?: number;
  /**
   * Its energy, a thin violet bar under the health.
   *
   * @default 0
   * @range 0..
   */
  readonly energy?: number;
  /**
   * Its most energy; 0 is none.
   *
   * @default 0
   * @range 0..
   */
  readonly energyMax?: number;
  /**
   * Something it builds or researches, 0 to 1: a thin gold bar along the tile's foot; null is none.
   *
   * @default null
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly progress?: number | null;
}

/**
 * An RTS selection panel.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SelectionPanelDesc {
  /**
   * The selection, in its order.
   *
   * @default []
   */
  readonly units?: readonly SelectedUnit[];
  /**
   * The unit the detail card shows (its key); empty is the first. Its tile is outlined.
   *
   * @default ""
   */
  readonly focus?: string;
  /**
   * Control group tabs over the panel; empty draws none.
   *
   * @default []
   */
  readonly groups?: readonly ControlGroup[];
  /**
   * The current group's key.
   *
   * @default ""
   */
  readonly group?: string;
  /**
   * The focused unit's stats in the detail card.
   *
   * @default []
   */
  readonly stats?: readonly UnitStat[];
  /**
   * Its attributes, a quiet line at the card's foot (Biological, Light).
   *
   * @default []
   */
  readonly tags?: readonly string[];
  /**
   * A building's queue while it has items (a building selected): beside its card, the item being built large with its name over a progress bar and the time left, the rest as small slots under it; on a phone a row of small slots under the card's line.
   *
   * @default null
   */
  readonly queue?: ProductionQueueDesc | null;
  /**
   * Tiles per row; 0 fits the panel (6 on a desktop, 4 on a phone).
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * Rows before the selection pages; 0 is 2.
   *
   * @default 0
   */
  readonly rows?: number;
  /**
   * The page shown, from 0.
   *
   * @default 0
   */
  readonly page?: number;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "bottom"
   */
  readonly anchor?: Anchor | null;
}

/** What a selection panel answered. */
export interface SelectionPanelResult {
  /** A unit's tile pressed this tick (its key): the game focuses it, or selects only it. */
  readonly clicked: string | null;
  /** A control group's tab pressed this tick (its key). */
  readonly group: string | null;
  /** A queue item pressed this tick (its key). */
  readonly cancelled: string | null;
}

/**
 * A line of the focused unit's stats.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface UnitStat {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * An icon of the atlas before the label; empty is none.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * What it is (Armour).
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its value as the game words it (8 - 10).
   *
   * @default ""
   */
  readonly value?: string;
}

/**
 * A wave on strategy.wavePreview.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface WaveEntry {
  /**
   * Its key (its probe id).
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its number.
   *
   * @default 0
   */
  readonly number?: number;
  /**
   * What comes (Crawlers), for a screen reader and the tooltip.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its enemy's art (art kind unitPortrait, 1:1).
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * An icon where there is no art.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * How many come; 0 prints none.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * A boss wave: its tile in the danger hue.
   *
   * @default false
   */
  readonly boss?: boolean;
}

/**
 * The waves to come.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface WavePreviewDesc {
  /**
   * The waves, the next first.
   *
   * @default []
   */
  readonly waves?: readonly WaveEntry[];
  /**
   * Its caption.
   *
   * @default "Wave preview"
   */
  readonly title?: string;
  /**
   * The wave under way, lit by the selected-tile rule; 0 is none.
   *
   * @default 0
   */
  readonly current?: number;
  /**
   * The most rows shown; the rest fold into a last row of dots. 0 is 5.
   *
   * @default 0
   */
  readonly max?: number;
  /**
   * Opens its own anchored group there; null builds it in place.
   *
   * @default "right"
   */
  readonly anchor?: Anchor | null;
}

/**
 * A management ledger: the headline figures, the lines by section with their budgets, and the history graph.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LedgerDesc {
  /**
   * The screen's title (Budget); empty shows none (the game titles it).
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The periods or views; empty shows no tabs.
   *
   * @default []
   */
  readonly tabs?: readonly LedgerTab[];
  /**
   * The selected tab's key.
   *
   * @default ""
   */
  readonly tab?: string;
  /**
   * The headline figures, start to end.
   *
   * @default []
   */
  readonly figures?: readonly LedgerFigure[];
  /**
   * The sections, top to bottom.
   *
   * @default []
   */
  readonly sections?: readonly LedgerSection[];
  /**
   * The lines, each in its section, in the game's order (the ledger sorts them by `sort`).
   *
   * @default []
   */
  readonly lines?: readonly LedgerLine[];
  /**
   * The column the lines are sorted by within each section: name, amount, change, or empty for the game's order.
   *
   * @default ""
   */
  readonly sort?: string;
  /**
   * The sort runs high to low.
   *
   * @default false
   */
  readonly descending?: boolean;
  /**
   * The sliders' low end, in percent.
   *
   * @default 50
   */
  readonly fundingMin?: number;
  /**
   * The sliders' high end, in percent.
   *
   * @default 150
   */
  readonly fundingMax?: number;
  /**
   * The sliders' step, in percent.
   *
   * @default 5
   */
  readonly fundingStep?: number;
  /**
   * The history's categories, oldest first (the months).
   *
   * @default []
   */
  readonly periods?: readonly string[];
  /**
   * The history's series over the periods (income, expenses, balance); empty shows no graph.
   *
   * @default []
   */
  readonly history?: readonly ChartSeries[];
  /**
   * How the history is drawn (a line by default).
   *
   * @default "line"
   */
  readonly graph?: ChartKind;
  /**
   * A section whose lines' shares are drawn as a donut beside the history; empty shows none.
   *
   * @default ""
   */
  readonly breakdown?: string;
  /**
   * The money's sign, written before amounts ($, ¥); empty writes bare numbers.
   *
   * @default ""
   */
  readonly currency?: string;
  /**
   * The name column's heading.
   *
   * @default "Name"
   */
  readonly nameLabel?: string;
  /**
   * The amount column's heading.
   *
   * @default "Amount"
   */
  readonly amountLabel?: string;
  /**
   * The change column's heading.
   *
   * @default "Change"
   */
  readonly changeLabel?: string;
  /**
   * The slider column's heading.
   *
   * @default "Budget"
   */
  readonly fundingLabel?: string;
  /**
   * Said before a section's total, and the net's caption.
   *
   * @default "Total"
   */
  readonly totalLabel?: string;
  /**
   * The net line's caption at the ledger's foot.
   *
   * @default "Net"
   */
  readonly netLabel?: string;
  /**
   * The graph's heading.
   *
   * @default "History"
   */
  readonly historyLabel?: string;
  /**
   * The donut's heading.
   *
   * @default "Breakdown"
   */
  readonly breakdownLabel?: string;
  /**
   * The donut's line for the smaller lines it folds together when the legend would not fit its card.
   *
   * @default "Other"
   */
  readonly otherLabel?: string;
  /**
   * A locked slider, to a screen reader.
   *
   * @default "Locked"
   */
  readonly lockedLabel?: string;
}

/**
 * One headline figure over the ledger (Balance, Income, Expenses, Population).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LedgerFigure {
  /**
   * Its key: its tile's element id.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its name, the tile's caption.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its value this period.
   *
   * @default 0
   */
  readonly value?: number;
  /**
   * Its change since the last period, shown signed with an arrow; 0 shows none.
   *
   * @default 0
   */
  readonly delta?: number;
  /**
   * How the value and the change are written (grouped by default).
   *
   * @default "grouped"
   */
  readonly format?: NumberFormat;
  /**
   * A unit after the value (a currency's sign is the game's, in the label or the unit).
   *
   * @default ""
   */
  readonly unit?: string;
  /**
   * Its art (art kind itemIcon) on the tile; null shows none.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * A fall is good news (expenses, debt, crime): the change's colour turns over.
   *
   * @default false
   */
  readonly lowerIsBetter?: boolean;
}

/** A line's slider was moved. */
export interface LedgerFunding {
  /** The line. */
  readonly key: string;
  /** Its new value, in percent. */
  readonly value: number;
}

/**
 * One line of the ledger: a tax, a service, a department, a contract.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LedgerLine {
  /**
   * Its key: its row's element id, so its probe path is <id>/Body/Ledger/List/<key>.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * The section it stands in.
   *
   * @default ""
   */
  readonly section?: string;
  /**
   * Its name, the row's reading.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its art (art kind itemIcon) at the row's start; null draws the style's placeholder mark.
   *
   * @default null
   */
  readonly image?: Texture | null;
  /**
   * Its amount this period (positive; the section says which way it runs).
   *
   * @default 0
   */
  readonly amount?: number;
  /**
   * Its amount last period, for its change; a negative value shows none.
   *
   * @default -1
   */
  readonly previous?: number;
  /**
   * Its budget or tax rate as a slider in percent; a negative value shows no slider (a fixed line).
   *
   * @default -1
   */
  readonly funding?: number;
  /**
   * A problem with it (Underfunded: crime rising), shown with a warning mark; empty shows none.
   *
   * @default ""
   */
  readonly warning?: string;
  /**
   * Its slider cannot move yet (a policy not unlocked): shown inert with a lock.
   *
   * @default false
   */
  readonly locked?: boolean;
}

/** What the player asked for this tick, for the game to apply. */
export interface LedgerResult {
  /** Another tab was pressed. */
  readonly tab: string | null;
  /**
   * A column's heading was pressed: sort by it (the same column again turns the order over).
   */
  readonly sort: string | null;
  /** The order asked for with `sort`. */
  readonly descending: boolean;
  /** A line's slider was moved. */
  readonly funding: LedgerFunding | null;
  /** A line was pressed (the game may open its detail or its overlay). */
  readonly selected: string | null;
}

/**
 * A section of the ledger's lines (Income, Expenses; Revenue, Upkeep).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LedgerSection {
  /**
   * Its key: the lines' `section`.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its heading, with its total.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its lines cost money: their total is taken from the net, and its change's colour turns over.
   *
   * @default false
   */
  readonly expense?: boolean;
}

/**
 * One of the ledger's periods or views (Month, Quarter, Year).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LedgerTab {
  /**
   * Its key.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Its word.
   *
   * @default ""
   */
  readonly label?: string;
}
