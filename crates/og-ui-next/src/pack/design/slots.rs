//! Default pack slot class definitions (`Component.slot[.variant]`).

/// The slots a pack dresses (`variants` keys), each with what it is and
/// the default pack's own dressing. The default pack declares each, so a
/// component's `packClass` on one resolves under every pack; a slot no
/// layer names stays an unknown class. A HUD plate's default is a nearly
/// opaque ground with its ink, so its text reads over any scene: a
/// component's translucent or cleared fill under it is overruled. A row
/// and a table head are opaque grounds with their inks for the same
/// reason: the lists, tables and feeds that sit over the scene.
pub const SLOTS: &[(&str, &str, &str)] = &[
    (
        "Press.feedback",
        "the squash a control that answers a press shows under it, unless its own classes move it under active (its default is the motion tokens': Timing::press)",
        "",
    ),
    (
        "Screen.frame",
        "a full-screen built-in's root: how it enters and leaves (Timing::screen)",
        "",
    ),
    (
        "Modal.frame",
        "a dialog or popup: how it pops in and leaves (Timing::modal)",
        "",
    ),
    (
        "List.cascade",
        "a list whose rows enter one after another when it shows or gains rows (Timing::cascade)",
        "",
    ),
    (
        "List.item",
        "a row of a list with List.cascade: how it enters in its turn",
        "animate-rise",
    ),
    (
        "Reward.pop",
        "a reward or a reveal arriving: how it pops in (in its turn under List.cascade)",
        "animate-pop",
    ),
    (
        "Tab.indicator",
        "the mark under a selected tab or segment, sliding from the one it replaces (Timing::indicator)",
        "",
    ),
    (
        "Bar.fill",
        "a bar's or meter's fill: how it eases to a new value (Timing::bar)",
        "",
    ),
    (
        "Bar.trail",
        "the trail behind a bar's fill: what was just lost, draining after the fill (Timing::trail)",
        "",
    ),
    (
        "Attention.cue",
        "what a control plays once when it turns from disabled to enabled",
        "animate-nudge",
    ),
    (
        "Count.roll",
        "a number that rolls to a new value: on the Text that holds it, it counts up or down on the count row (Timing::count)",
        "",
    ),
    (
        "Modal.scrim",
        "the scrim behind a dialog: what dims the screen and how it fades in and out, apart from the dialog's pop (Timing::scrim)",
        "",
    ),
    (
        "Focus.ring",
        "the ring a control shows while a keyboard or pad focuses it, unless its own classes style focus-visible",
        "outline-regular outline-focus -outline-offset-2",
    ),
    (
        "Button.plate",
        "a button's body: rune-kit's neutral variant, and the states every variant shares (hover, pressed, a toggle's aria-pressed, selected, disabled, aria-busy)",
        "rounded-button text-fg bg-raised-lo shadow-raise inset-shadow-sunk inset-ring-hairline inset-ring-border-strong transition duration-control hover:bg-none hover:bg-raised-hover-lo hover:shadow-plate hover:inset-ring-well-rim active:bg-none active:bg-raised-pressed aria-pressed:bg-none aria-pressed:bg-border-strong aria-pressed:inset-ring-regular aria-pressed:inset-ring-accent selected:bg-none selected:bg-border-strong selected:inset-ring-regular selected:inset-ring-accent disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:shadow-none disabled:inset-ring-hairline disabled:inset-ring-border aria-busy:opacity-80",
    ),
    (
        "Button.plate.primary",
        "the main action (rune-kit's submit)",
        "text-accent-fg bg-linear-to-b from-sheen to-transparent to-48% bg-accent inset-shadow-none inset-ring-0 hover:bg-linear-to-b hover:from-sheen hover:to-lit hover:to-60% hover:bg-accent active:bg-linear-to-b active:from-shade active:to-transparent active:to-60% active:bg-accent",
    ),
    (
        "Button.plate.danger",
        "a destructive action",
        "text-danger-fg bg-linear-to-b from-sheen to-transparent to-48% bg-danger inset-shadow-none inset-ring-0 hover:bg-linear-to-b hover:from-sheen hover:to-lit hover:to-60% hover:bg-danger active:bg-linear-to-b active:from-shade active:to-transparent active:to-60% active:bg-danger",
    ),
    (
        "Button.plate.warning",
        "an action with a cost or a risk",
        "text-warn-fg bg-linear-to-b from-sheen to-transparent to-48% bg-warn inset-shadow-none inset-ring-0 hover:bg-linear-to-b hover:from-sheen hover:to-lit hover:to-60% hover:bg-warn active:bg-linear-to-b active:from-shade active:to-transparent active:to-60% active:bg-warn",
    ),
    (
        "Button.plate.contrast",
        "the inverse of the page: a light button on a dark page",
        "text-fg-inverse bg-none bg-fg shadow-raise inset-shadow-none inset-ring-0 hover:bg-none hover:bg-fg-muted active:bg-none active:bg-fg-subtle",
    ),
    (
        "Button.plate.outline",
        "a rim on the page, no fill",
        "text-fg bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-hover hover:shadow-none active:bg-none active:bg-pressed disabled:bg-none disabled:bg-transparent disabled:inset-ring-0 inset-ring-hairline inset-ring-border-strong hover:inset-ring-border-strong",
    ),
    (
        "Button.plate.ghost",
        "no plate until hovered or pressed",
        "text-fg bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-hover hover:shadow-none active:bg-none active:bg-pressed disabled:bg-none disabled:bg-transparent disabled:inset-ring-0",
    ),
    (
        "Button.plate.ghost-muted",
        "a ghost in the muted ink: a secondary action beside others",
        "text-fg-muted hover:text-fg bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-hover hover:shadow-none active:bg-none active:bg-pressed disabled:bg-none disabled:bg-transparent disabled:inset-ring-0",
    ),
    (
        "Button.plate.ghost-faint",
        "a ghost in the faint ink: the least of a set (Skip)",
        "text-fg-subtle hover:text-fg-muted bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-hover hover:shadow-none active:bg-none active:bg-pressed disabled:bg-none disabled:bg-transparent disabled:inset-ring-0",
    ),
    (
        "Button.plate.menu",
        "a front-end menu entry (MenuEntry: the main menu's, the pause menu's): type, not a box, dim until it is the current one, then in the ink of the MenuList.indicator gliding under it",
        "relative w-full justify-start py-2 px-4 rounded-button text-fg-muted bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-transparent hover:shadow-none hover:inset-ring-0 active:bg-none active:bg-transparent data-current:text-hud-accent-fg disabled:bg-none disabled:bg-transparent disabled:text-fg-subtle disabled:inset-ring-0",
    ),
    (
        "Button.plate.menu-danger",
        "a menu entry that ends something (Quit game): in the danger's ink while current",
        "relative w-full justify-start py-2 px-4 rounded-button text-fg-muted bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-transparent hover:shadow-none hover:inset-ring-0 active:bg-none active:bg-transparent data-current:text-danger-fg disabled:bg-none disabled:bg-transparent disabled:text-fg-subtle disabled:inset-ring-0",
    ),
    (
        "MenuList.indicator",
        "the bar behind a menu list's current entry (MenuList): it glides from the entry it leaves to the one hovered or focused (Timing::indicator)",
        "absolute inset-0 rounded-button bg-hud-accent pointer-events-none",
    ),
    (
        "MenuList.indicator.danger",
        "the bar behind a current entry that ends something (Quit game)",
        "bg-danger",
    ),
    (
        "MenuList.cursor",
        "the mark beside a menu list's current entry (MenuList): a pack's pointer, arrow or glint at the entry's leading edge, over the indicator; none in the default pack, whose indicator is the selection",
        "hidden",
    ),
    (
        "MenuList.cursor.danger",
        "the mark beside a current entry that ends something (Quit game)",
        "",
    ),
    (
        "MenuList.label",
        "a menu entry's words (MenuEntry): the main menu's size; the pack sets the face, case and treatment",
        "text-start text-3xl leading-tight font-semibold text-shadow-xs text-shadow-bg",
    ),
    (
        "MenuList.label.md",
        "a pause menu's entry words (MenuEntry size md)",
        "text-title compact:text-xl",
    ),
    (
        "MenuList.line",
        "the line under a menu entry's words (where the last save stands)",
        "text-start text-base font-medium opacity-quiet whitespace-nowrap overflow-hidden",
    ),
    (
        "MenuList.chevron",
        "the chevron at a major menu entry's trailing end, shown while it is the current one",
        "relative opacity-0 size-4 transition-opacity duration-control group-data-current/entry:opacity-100 rtl:-scale-x-100 rtl:scale-y-100",
    ),
    (
        "Button.spinner",
        "what a busy button shows before its label",
        "size-4 shrink-0 rounded-capsule border-regular border-current border-t-transparent animate-spin",
    ),
    (
        "Checkbox.indicator",
        "a checkbox's box; checked: and indeterminate: follow the checkbox",
        "flex items-center justify-center size-5 shrink-0 rounded-control bg-fill-strong inset-ring-regular inset-ring-border-strong text-accent-fg transition duration-instant group-hover/checkbox:inset-ring-accent group-active/checkbox:scale-press checked:bg-accent checked:inset-ring-0 indeterminate:bg-accent indeterminate:inset-ring-0 aria-invalid:inset-ring-danger disabled:opacity-50",
    ),
    (
        "Radio.indicator",
        "a radio's circle; checked: follows the radio",
        "flex items-center justify-center size-5 shrink-0 rounded-capsule bg-fill-strong inset-ring-regular inset-ring-border-strong transition duration-instant group-hover/radio:inset-ring-accent group-active/radio:scale-press checked:inset-ring-accent disabled:opacity-50",
    ),
    (
        "Radio.dot",
        "the dot inside a checked radio",
        "size-2.5 rounded-capsule bg-accent scale-0 transition-transform duration-instant group-checked/radio:scale-100",
    ),
    (
        "Switch.track",
        "a switch's track; checked: follows the switch",
        "relative h-6 w-11 shrink-0 rounded-capsule bg-fill-strong inset-ring-hairline inset-ring-border-strong transition duration-200 group-hover/switch:inset-ring-accent group-active/switch:scale-press checked:bg-accent checked:inset-ring-0 disabled:opacity-50",
    ),
    (
        "Switch.thumb",
        "the thumb that moves along a switch's track",
        "absolute top-0.5 left-0.5 size-5 rounded-capsule bg-knob shadow-raise transition-transform duration-200 ease-out group-active/switch:scale-95 checked:translate-x-5 checked:bg-accent-fg",
    ),
    (
        "Tab.item",
        "a tab of a tab list",
        "flex-row items-center justify-center gap-2 min-h-11 px-3 text-sm font-semibold text-fg-muted transition duration-instant hover:text-fg active:bg-pressed selected:text-fg disabled:text-fg-subtle",
    ),
    (
        "Checkbox.mark",
        "the check inside a checked checkbox",
        "size-3.5 text-accent-fg",
    ),
    (
        "Checkbox.dash",
        "the dash inside an indeterminate checkbox",
        "w-2.5 h-0.5 rounded-capsule bg-accent-fg",
    ),
    (
        "Tab.count",
        "a count beside a tab's label, on its own ground (the selected tab's is lit)",
        "min-w-5 px-1 rounded-capsule bg-layer-3 text-xs font-bold leading-snug tabular-nums text-center text-fg-muted group-selected/tab:bg-accent group-selected/tab:text-accent-fg",
    ),
    (
        "Tab.list",
        "a tab list's row",
        "flex-row items-stretch border-b border-border-muted",
    ),
    (
        "Tab.mark",
        "the selected tab's mark: an underline (Tab.indicator is how it slides)",
        "inset-x-2 bottom-0 h-0.75 rounded-capsule bg-accent",
    ),
    (
        "Toggle.item",
        "a toggle button, alone or in a toggle group; aria-pressed: is on",
        "flex-row items-center justify-center gap-2 min-h-11 min-w-11 px-3 rounded-control text-sm font-semibold text-fg-muted transition duration-instant hover:bg-hover hover:text-fg active:bg-pressed aria-pressed:bg-layer-4 aria-pressed:text-fg aria-pressed:shadow-raise disabled:text-fg-subtle",
    ),
    (
        "ToggleGroup.frame",
        "a toggle group's box: a segmented control's well",
        "flex-row items-stretch gap-0.5 p-0.5 rounded-field bg-layer-2 inset-ring-hairline inset-ring-border",
    ),
    (
        "Toolbar.frame",
        "a toolbar's bar",
        "flex-row items-center gap-1 p-1 rounded-field bg-layer-2 inset-ring-hairline inset-ring-border",
    ),
    (
        "Toolbar.separator",
        "the line between runs of a toolbar's items",
        "self-stretch w-px my-1.5 bg-border",
    ),
    (
        "Slider.track",
        "a slider's track",
        "relative grow h-1.5 rounded-capsule bg-track disabled:opacity-50",
    ),
    (
        "Slider.range",
        "the filled part of a slider's track, from its start to the thumb",
        "absolute left-0 top-0 h-full rounded-capsule bg-accent",
    ),
    (
        "Slider.thumb",
        "a slider's thumb",
        "absolute top-1/2 size-5 -translate-x-1/2 -translate-y-1/2 rounded-capsule bg-knob shadow-raise inset-ring-regular inset-ring-accent transition-transform duration-instant group-hover/slider:scale-105 group-active/slider:scale-110 disabled:opacity-50",
    ),
    (
        "NumberInput.frame",
        "a number input's box, around its field and steppers",
        "flex-row items-center rounded-field bg-well inset-ring-hairline inset-ring-border focus-within:inset-ring-regular focus-within:inset-ring-accent disabled:opacity-50",
    ),
    (
        "NumberInput.field",
        "a number input's text",
        "grow min-w-0 px-2 text-center text-base font-semibold tabular-nums text-fg",
    ),
    (
        "NumberInput.step",
        "a number input's decrement and increment buttons",
        "flex items-center justify-center size-11 shrink-0 rounded-control text-lg font-bold text-fg-muted transition duration-instant hover:bg-hover hover:text-fg active:bg-pressed disabled:text-fg-subtle",
    ),
    (
        "Dialog.frame",
        "a dialog's panel, over Modal.scrim (Modal.frame is how it pops in)",
        "flex-col gap-3 w-115 max-w-full max-h-[85vh] overflow-y-auto m-auto p-6 compact:p-4 rounded-panel bg-linear-to-b from-glass-hi to-glass-lo backdrop-blur-regular text-fg shadow-overlay inset-ring-hairline inset-ring-border",
    ),
    (
        "Dialog.title",
        "a dialog's heading",
        "font-display text-title font-bold leading-tight text-fg",
    ),
    (
        "Dialog.body",
        "what a dialog says under its heading",
        "text-base leading-normal text-fg-muted",
    ),
    (
        "Dialog.actions",
        "the row of a dialog's buttons",
        "flex-row flex-wrap justify-end gap-2 pt-2",
    ),
    (
        "Sheet.frame",
        "a sheet's panel at the bottom of the screen",
        "flex-col gap-3 w-full max-h-[85%] mt-auto px-4 pt-2 pb-[calc(env(hud-inset-bottom)+1.5rem)] rounded-t-sheet bg-linear-to-b from-glass-hi to-glass-lo backdrop-blur-regular text-fg shadow-overlay inset-ring-hairline inset-ring-border transition ease-out duration-base starting:translate-y-12 starting:opacity-0 exit:translate-y-12 exit:opacity-0 exit:duration-fast exit:ease-exit",
    ),
    (
        "Sheet.grabber",
        "the handle at the top of a sheet",
        "self-center w-10 h-1.5 my-1 rounded-capsule bg-border-strong",
    ),
    (
        "Scrollbar.track",
        "a scroll view's own bar over its edge; its thumb hides while nothing runs over",
        "absolute z-10 top-1 bottom-1 right-0.5 w-2.5 flex-col items-center pointer-coarse:w-1.5",
    ),
    (
        "Scrollbar.thumb",
        "the thumb of a scroll view's bar: as long as the share shown, dragged to scroll",
        "w-1 rounded-capsule bg-fg-subtle opacity-muted transition duration-instant hover:scale-x-150 hover:opacity-90 active:scale-x-150 active:opacity-100",
    ),
    (
        "Popover.frame",
        "a popover's panel by its trigger",
        "flex-col gap-2 min-w-48 max-w-80 p-3 rounded-card bg-linear-to-b from-plate-hi to-plate-lo backdrop-blur-thin text-fg shadow-float inset-ring-hairline inset-ring-border transition duration-instant ease-out starting:opacity-0 starting:scale-95 exit:opacity-0",
    ),
    (
        "Menu.frame",
        "a menu's popup by its trigger, around its items",
        "flex-col min-w-52 max-w-80 p-1 rounded-card bg-linear-to-b from-plate-hi to-plate-lo backdrop-blur-thin text-fg shadow-float inset-ring-hairline inset-ring-border transition duration-instant ease-out starting:opacity-0 starting:scale-95 exit:opacity-0 exit:duration-90",
    ),
    (
        "Menu.item",
        "an item of a menu; focus: is the one the keys are on",
        "flex-row items-center gap-2.5 min-h-11 px-3 rounded-control text-sm font-semibold text-fg transition duration-90 hover:bg-hover focus:bg-hover active:bg-pressed disabled:text-fg-subtle",
    ),
    (
        "Menu.item.danger",
        "a menu item that destroys or cannot be undone",
        "text-danger",
    ),
    (
        "Menu.indicator",
        "the check before a checkable menu item's text, shown while it is checked",
        "flex items-center justify-center size-4.5 shrink-0 text-accent opacity-0 checked:opacity-100",
    ),
    (
        "Menu.shortcut",
        "a menu item's hint after its text: a key or a count",
        "shrink-0 ml-auto pl-4 text-xs font-semibold text-fg-muted tabular-nums",
    ),
    (
        "Menu.separator",
        "a line between groups of a menu's items",
        "h-px my-1 mx-2 bg-border",
    ),
    (
        "Menu.label",
        "a group's heading in a menu",
        "px-3 pt-2 pb-1 text-xs font-bold uppercase tracking-label text-fg-muted",
    ),
    (
        "Tooltip.frame",
        "a tooltip's bubble",
        "max-w-64 px-2.5 py-1.5 rounded-control bg-fg text-fg-inverse text-xs font-semibold shadow-float",
    ),
    (
        "Toast.stack",
        "the column toasts stack in",
        "flex-col items-center gap-2 p-4 pt-[calc(env(safe-area-inset-top)+1rem)] pointer-events-none",
    ),
    (
        "Toast.frame",
        "a toast",
        "flex-row items-center gap-3 w-full max-w-96 min-h-12 px-4 py-2 rounded-card bg-linear-to-b from-plate-hi to-plate-lo backdrop-blur-thin text-fg shadow-float inset-ring-hairline inset-ring-border pointer-events-auto transition ease-out duration-base starting:opacity-0 starting:-translate-y-4 exit:opacity-0 exit:duration-instant",
    ),
    (
        "Toast.frame.success",
        "a toast saying something went well",
        "inset-ring-hairline inset-ring-success",
    ),
    (
        "Toast.frame.danger",
        "a toast saying something failed",
        "inset-ring-hairline inset-ring-danger",
    ),
    (
        "Select.trigger",
        "a select's button, showing its value",
        "flex-row items-center justify-between gap-2 min-h-11 px-3 rounded-field bg-well inset-ring-hairline inset-ring-border text-fg transition duration-instant hover:inset-ring-border-strong active:bg-pressed aria-expanded:inset-ring-accent aria-invalid:inset-ring-regular aria-invalid:inset-ring-danger disabled:text-fg-subtle disabled:opacity-50",
    ),
    (
        "Select.list",
        "a select's open list of options",
        "flex-col p-1 rounded-card bg-linear-to-b from-plate-hi to-plate-lo backdrop-blur-thin text-fg shadow-float inset-ring-hairline inset-ring-border transition duration-instant ease-out starting:opacity-0 starting:-translate-y-1 exit:opacity-0",
    ),
    (
        "Select.option",
        "an option of a select's list",
        "flex-row items-center gap-2 min-h-11 px-3 rounded-control text-fg hover:bg-hover active:bg-pressed selected:bg-layer-4 selected:font-semibold disabled:text-fg-subtle",
    ),
    (
        "Collapsible.trigger",
        "what a collapsible's or accordion item's header presses",
        "flex-row items-center justify-between gap-2 min-h-11 px-3 text-fg font-semibold hover:bg-hover active:bg-pressed disabled:text-fg-subtle disabled:opacity-50",
    ),
    (
        "Collapsible.chevron",
        "the chevron that turns as it opens",
        "size-4 text-fg-muted transition-transform duration-200 group-aria-expanded/collapsible:rotate-180",
    ),
    (
        "Collapsible.panel",
        "what a collapsible shows while open",
        "flex-col gap-2 px-3 pb-3 text-fg-muted transition ease-out duration-200 starting:opacity-0 starting:-translate-y-1 exit:opacity-0 exit:duration-instant",
    ),
    (
        "Accordion.item",
        "a section of an accordion",
        "flex-col border-b border-border-muted",
    ),
    (
        "Progress.track",
        "a progress bar's track; its indicator is Bar.fill",
        "w-full h-2 overflow-hidden rounded-capsule bg-fill-strong shadow-sunk",
    ),
    (
        "Progress.indeterminate",
        "a progress bar's indicator while its extent is unknown",
        "w-1/3 animate-pulse",
    ),
    (
        "Panel.frame",
        "a window or panel's frame (the first glass layer)",
        "rounded-panel bg-linear-to-b from-glass-hi to-glass-lo backdrop-blur-regular text-fg shadow-panel inset-ring-hairline inset-ring-border",
    ),
    (
        "Title.wordmark",
        "a game's name set in type where it has no logo (a title or main menu's wordmark): the pack's display face, size and treatment",
        "font-display text-display-2xl leading-tight font-bold text-fg text-shadow-xs text-shadow-bg",
    ),
    (
        "Panel.body",
        "a titled panel's inside (Panel): the room between its frame and its content, and the gap between its parts",
        "flex-col gap-4 p-6 compact:p-4",
    ),
    (
        "Panel.title",
        "a titled panel's heading (Panel): its words or the band they sit on",
        "font-display text-title font-bold leading-tight text-fg",
    ),
    (
        "Plate.frame",
        "a plate over the scene or inside a panel (the second glass layer)",
        "rounded-card bg-linear-to-b from-plate-hi to-plate-lo backdrop-blur-thin text-fg shadow-plate inset-ring-hairline inset-ring-border",
    ),
    (
        "Slot.frame",
        "an ability or item slot",
        "rounded-card bg-well text-fg shadow-sunk inset-ring-hairline inset-ring-border",
    ),
    (
        "Ability.box",
        "an ability's side on a HUD's ability bar (combat HUD design §4.1)",
        "size-13 compact:size-9",
    ),
    (
        "Ability.box.ultimate",
        "an ultimate's side",
        "size-21 compact:size-11",
    ),
    (
        "Ability.frame",
        "an ability's slot: its shape and rim",
        "rounded-card bg-linear-to-b from-rim-strong via-rim to-rim-soft transition duration-control",
    ),
    (
        "Ability.frame.ultimate",
        "an ultimate's slot, in place of Ability.frame: its rim is the charge ring",
        "rounded-full transition duration-control",
    ),
    (
        "Ability.frame.ready",
        "a slot ready to use: lit, and breathing while it waits",
        "from-hud-accent-hi to-hud-accent shadow-glow animate-breathe",
    ),
    (
        "Ability.frame.disabled",
        "a slot that cannot be used now: its art greyed (its figures keep their ink)",
        "grayscale",
    ),
    (
        "Ability.face",
        "the ground an ability's art sits on, inside its rim",
        "inset-0.5 rounded-block overflow-hidden bg-plate-hi",
    ),
    (
        "Ability.face.ultimate",
        "an ultimate's face: its inset is the charge ring's width",
        "inset-1/8 rounded-full",
    ),
    (
        "Ability.ring",
        "the track an ultimate's charge fills round",
        "nine-disc bg-track rounded-full",
    ),
    (
        "Ability.ring.ready",
        "an ultimate fully charged: its ring lit, and breathing while it waits",
        "shadow-glow animate-breathe",
    ),
    (
        "Ability.ring.fill",
        "the charge filling round an ultimate, clockwise from the top",
        "nine-disc bg-hud-accent",
    ),
    (
        "Ability.sweep",
        "the dark sweep over an ability's art while it cools down",
        "nine-tile bg-scrim opacity-sweep",
    ),
    (
        "Ability.sweep.ultimate",
        "an ultimate's cooldown sweep, on its round face",
        "nine-disc",
    ),
    (
        "Ability.count",
        "the seconds left over a cooldown's sweep",
        "text-title leading-none font-bold tabular-nums text-fg text-shadow-tag text-stroke-3 text-stroke-bg compact:text-base",
    ),
    (
        "Ability.chip.ult",
        "an ultimate's charge in figures while it charges (42%): its type and place",
        "text-caption leading-none font-bold tabular-nums text-fg bg-raised-pressed inset-ring-hairline inset-ring-border px-1.5 py-0.5 rounded-capsule",
    ),
    (
        "Ability.chip.ready",
        "the word an ultimate shows when it is full (Ready): its type, ground and place",
        "text-caption leading-none font-bold uppercase tracking-label text-hud-accent-fg bg-hud-accent px-1.5 py-0.5 rounded-capsule",
    ),
    (
        "Ability.chip.key",
        "the key cap under an ability",
        "py-0 px-1 text-xs leading-normal font-bold text-center text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key",
    ),
    (
        "Ability.icon",
        "an ability's glyph on its face",
        "size-1/2 text-fg text-shadow-tag",
    ),
    (
        "Ability.icon.disabled",
        "the glyph of a slot that cannot be used, locked or out of reach",
        "opacity-dim",
    ),
    (
        "Ability.tint",
        "the wash over an ability's art when it costs more than the player has or its target is out of range",
        "bg-linear-to-t from-danger to-transparent",
    ),
    (
        "Ability.slash",
        "the bar struck across a slot that cannot be used now",
        "w-11/10 h-1.25 bg-danger rounded-capsule -rotate-45",
    ),
    (
        "Ability.pip",
        "one rank of an ability under it, unlearnt",
        "w-2 h-1 rounded-capsule bg-track",
    ),
    ("Ability.pip.on", "a rank learnt", "bg-gold"),
    (
        "Ability.chip.cost",
        "an ability's cost at its foot while it is ready",
        "text-caption tabular-nums bg-surface-raised inset-ring-hairline inset-ring-border text-mana text-shadow-tag px-2 rounded-capsule",
    ),
    (
        "Ability.chip.short",
        "a cost the player cannot pay",
        "text-danger",
    ),
    (
        "Ability.chip.count",
        "how many of a quick item are left, at the slot's foot",
        "tabular-nums text-fg text-shadow-tag text-reading",
    ),
    (
        "Bar.frame",
        "a HUD bar's well: its height, track and rim (HudBar, a boss's health, StatBar)",
        "h-1.5 overflow-hidden rounded-capsule bg-track shadow-plate ring ring-border",
    ),
    (
        "Bar.frame.thick",
        "a bar thick enough to hold its figure (a boss's health, StatBar)",
        "h-3.5 compact:h-2.25",
    ),
    (
        "Bar.frame.low",
        "a bar the game called low or critical: its rim lit in danger, pulsing",
        "shadow-glow shadow-critical ring-critical animate-pulse",
    ),
    (
        "Bar.tone",
        "a bar's fill colour: health, unless a kind names another",
        "bg-linear-to-b from-sheen to-transparent to-42% bg-health shadow-glow shadow-health",
    ),
    ("Bar.tone.mana", "a mana bar's fill", "bg-mana shadow-mana"),
    (
        "Bar.tone.stamina",
        "a stamina bar's fill",
        "bg-gold shadow-gold",
    ),
    (
        "Bar.tone.shield",
        "a shield bar's fill",
        "bg-ally shadow-ally",
    ),
    (
        "Bar.tone.xp",
        "an experience bar's fill",
        "bg-linear-to-r from-xp to-xp shadow-xp",
    ),
    (
        "Bar.tone.boss",
        "a boss's health",
        "bg-linear-to-r from-legendary to-legendary shadow-late",
    ),
    (
        "Bar.tone.low",
        "a low bar's fill",
        "bg-critical shadow-critical",
    ),
    (
        "Bar.tone.invulnerable",
        "a fill that takes no damage now (an invulnerable boss)",
        "bg-none bg-fg-subtle shadow-none",
    ),
    (
        "Bar.ghost",
        "what was just lost, behind the fill",
        "bg-gold",
    ),
    (
        "Bar.tick",
        "the notch between a bar's cells",
        "w-0.75 bg-glass",
    ),
    (
        "Bar.value",
        "a bar's figure, over or beside it",
        "text-caption tabular-nums text-fg text-shadow-tag",
    ),
    ("Ring.box", "a progress ring's side", "size-14"),
    (
        "Ring.track",
        "the ground a progress ring fills round",
        "rounded-full bg-glass",
    ),
    (
        "Ring.rest",
        "the share of a progress ring still to fill, a ring of its own so it parts from the middle",
        "nine-ring bg-fg-subtle opacity-muted",
    ),
    (
        "Ring.fill",
        "a progress ring's fill, clockwise from the top",
        "nine-ring bg-accent transition-layout duration-slow ease-out",
    ),
    ("Ring.fill.success", "a ring done well", "bg-success"),
    ("Ring.fill.warn", "a ring in warning", "bg-warn"),
    ("Ring.fill.danger", "a ring in danger", "bg-danger"),
    ("Ring.fill.info", "an informing ring", "bg-info"),
    (
        "Ring.spin",
        "a ring turning with no known value",
        "rounded-full border-thick border-rim animate-spin",
    ),
    (
        "Ring.label",
        "the percent or words in a ring",
        "text-caption font-bold tabular-nums text-fg bg-raised-pressed px-1 rounded-capsule",
    ),
    ("Orb.box", "a resource orb's side", "size-28"),
    (
        "Orb.rim",
        "the orb's outer rim",
        "rounded-full bg-rim-strong shadow-plate ring ring-key-fg",
    ),
    (
        "Orb.well",
        "the glass the liquid fills",
        "inset-2.25 rounded-full overflow-hidden bg-boon-fg inset-shadow-sunk",
    ),
    ("Orb.well.mana", "a mana orb's glass", "bg-raised-pressed"),
    (
        "Orb.fill",
        "the liquid: health",
        "border-t border-t-boon bg-health",
    ),
    (
        "Orb.fill.mana",
        "a mana orb's liquid",
        "border-t-mana bg-mana",
    ),
    (
        "Orb.glass",
        "the highlight riding the liquid's surface",
        "rounded-full bg-sheen",
    ),
    (
        "Orb.value",
        "the orb's figure, on its own ground so it reads on any liquid",
        "text-display-sm font-bold tabular-nums text-fg bg-raised-pressed inset-ring-hairline inset-ring-border px-2 rounded-capsule",
    ),
    (
        "Orb.label",
        "the orb's name under it, on its own ground so it reads over the scene",
        "text-xs font-semibold tracking-label uppercase text-fg bg-raised-pressed px-2 rounded-capsule",
    ),
    ("Radial.box", "a radial meter's side", "size-12"),
    (
        "Radial.track",
        "a radial meter wheel's track",
        "rounded-full border border-rim ring ring-shade-deep bg-track",
    ),
    (
        "Radial.fill",
        "a radial meter wheel's fill",
        "nine-ring bg-success transition-layout duration-fast ease-out",
    ),
    ("Radial.fill.exhausted", "a spent radial meter", "bg-foe"),
    (
        "Radial.sheen",
        "the light over a radial meter wheel",
        "rounded-full bg-linear-to-b from-sheen to-transparent to-42%",
    ),
    (
        "Boss.phase",
        "a phase's mark on a boss's health: dark on any fill, its rim light",
        "bg-shade-deep ring-regular ring-fg",
    ),
    (
        "Boss.status",
        "an invulnerable or defeated boss's word, on its own ground",
        "text-xs font-semibold tracking-label uppercase text-fg bg-raised-pressed px-1.5 rounded-capsule",
    ),
    (
        "Boss.figure",
        "a boss's health in percent",
        "text-base leading-none font-bold tabular-nums text-fg bg-raised-pressed px-1.5 py-0.5 rounded-capsule compact:text-caption",
    ),
    (
        "Damage.number",
        "a damage number rising from a hit",
        "text-lg font-bold tabular-nums text-fg text-shadow-tag animate-float",
    ),
    (
        "Damage.number.crit",
        "a critical hit's number: larger, in gold",
        "text-3xl text-gold",
    ),
    ("Damage.number.heal", "a heal's number", "text-success"),
    (
        "Damage.number.shield",
        "damage the shield took",
        "text-info",
    ),
    ("Damage.number.weak", "a hit on a weakness", "text-warn"),
    (
        "Damage.number.miss",
        "a miss or an immune hit",
        "text-base text-fg-muted",
    ),
    (
        "Hit.ring",
        "the ring the hit direction arcs sit on, round the view's centre",
        "size-48 compact:size-36",
    ),
    (
        "Hit.arc",
        "one hit's direction arc, fading as its hit goes",
        "stroke-foe stroke-arc transition duration-arc exit:opacity-0",
    ),
    (
        "Hit.arc.shield",
        "an arc for a hit the shield took",
        "stroke-ally",
    ),
    ("Hit.arc.critical", "a critical hit's arc", "stroke-arc-lg"),
    (
        "Hit.vignette",
        "the screen's edges flashing as a hit lands (HitFeedback)",
        "inset-shadow-vignette opacity-0 transition-opacity duration-hit ease-out starting:opacity-flash",
    ),
    (
        "Hit.vignette.critical",
        "the edges pulsing while the game says the player is critical",
        "opacity-muted animate-pulse",
    ),
    (
        "Combo.plate",
        "the combo readout's plate",
        "gap-y-1 p-2 rounded-card bg-linear-to-b from-plate-hi to-plate-lo backdrop-blur-thin text-fg shadow-plate inset-ring-hairline inset-ring-border",
    ),
    (
        "Combo.count",
        "the combo count, popping as it rises",
        "text-2xl font-bold tabular-nums text-fg text-shadow-tag",
    ),
    (
        "Combo.multiplier",
        "the combo's multiplier",
        "text-lg font-bold tabular-nums text-gold text-shadow-tag",
    ),
    (
        "Feed.row",
        "a row of a killfeed or pickup feed over the scene",
        "gap-x-1.5 rounded-control bg-glass text-fg inset-ring-hairline inset-ring-border transition duration-base ease-out starting:opacity-0 starting:translate-x-4 exit:opacity-0",
    ),
    (
        "Feed.row.self",
        "a row the player took part in",
        "inset-shadow-bar inset-shadow-hud-accent",
    ),
    (
        "Feed.name",
        "a name in a feed row",
        "text-sm font-semibold text-shadow-tag",
    ),
    (
        "Feed.weapon",
        "the weapon between a feed row's names; it gives way before the names do",
        "text-xs font-semibold tracking-label uppercase text-fg-muted text-shadow-tag",
    ),
    (
        "Feed.plate",
        "a pickup feed's plate",
        "gap-y-1 p-2 rounded-card bg-linear-to-b from-plate-hi to-plate-lo backdrop-blur-thin text-fg shadow-plate inset-ring-hairline inset-ring-border",
    ),
    (
        "Celebrate.title",
        "a level-up's big word, on its own dark band so it reads over the burst",
        "text-title font-bold tracking-label uppercase text-gold bg-raised-pressed px-3 rounded-capsule animate-stamp",
    ),
    (
        "Celebrate.halo",
        "the ring of light behind a level-up",
        "size-66 compact:size-23.75 rounded-full border-thick border-gold shadow-glow-lg shadow-gold inset-shadow-glow inset-shadow-gold bg-glass animate-stamp",
    ),
    (
        "Touch.base",
        "a touch stick's ring, the stick zone's box",
        "size-40 compact:size-28 rounded-full inset-ring-regular inset-ring-border-strong bg-tile shadow-plate opacity-soft active:opacity-100 active:bg-tile-pressed disabled:opacity-dim",
    ),
    (
        "Touch.knob",
        "a touch stick's knob, following the thumb",
        "size-2/5 rounded-full nine-disc bg-stick shadow-float",
    ),
    (
        "Touch.pad",
        "a touch d-pad's cross, the d-pad zone's box",
        "size-40 compact:size-28 bg-tile active:scale-press disabled:opacity-dim",
    ),
    (
        "Touch.button",
        "a touch action button",
        "size-16 rounded-capsule inset-ring-regular inset-ring-border-strong bg-raised-hi text-fg shadow-float transition duration-fast active:scale-press disabled:opacity-dim",
    ),
    (
        "Touch.button.primary",
        "the main touch action (attack, fire, jump): larger, in the accent",
        "size-20 inset-ring-0 bg-accent text-accent-fg",
    ),
    (
        "Touch.charge",
        "a touch action's charge, filling round its rim clockwise from the top",
        "nine-ring bg-gold",
    ),
    (
        "Touch.charge.ready",
        "a touch action's charge when it is full: lit and breathing",
        "shadow-glow shadow-gold animate-breathe rounded-full",
    ),
    (
        "Touch.actions",
        "the touch action cluster: its gap and how wide it wraps",
        "gap-3 max-w-56",
    ),
    (
        "Touch.bar",
        "the touch controls' row: its padding and the gap between zones",
        "gap-6 p-4 compact:gap-4 compact:p-3",
    ),
    ("Touch.icon", "a touch action's glyph", "size-8"),
    (
        "Pips.units",
        "a pips row's units: the gap between them",
        "gap-0.5",
    ),
    (
        "Pips.pip",
        "one pip's box and its empty socket's ink",
        "size-6 text-fg-subtle",
    ),
    (
        "Pips.fill",
        "a pip's filled part: its ink, a life's red unless a tone names another",
        "text-danger transition-layout duration-base",
    ),
    ("Pips.fill.success", "a success-toned pip", "text-success"),
    ("Pips.fill.warn", "a warn-toned pip", "text-warn"),
    ("Pips.fill.info", "an info-toned pip", "text-info"),
    ("Pips.fill.primary", "a primary-toned pip", "text-accent"),
    (
        "Pips.label",
        "a pips row's name",
        "text-caption text-fg-muted",
    ),
    (
        "Counter.value",
        "a counter's number: tabular so a count does not jitter",
        "tabular-nums",
    ),
    (
        "Hero.frame",
        "a hero action's large button (End turn, Ultimate): its face, edge and the lift it stands on",
        "min-h-12 w-full max-w-50 py-3 px-4 gap-x-3 rounded-card bg-linear-to-b from-sheen to-transparent to-48% bg-accent text-accent-fg shadow-key inset-ring-hairline inset-ring-lit transition duration-fast hover:-translate-y-px active:translate-y-0.5 disabled:translate-y-0 disabled:opacity-dim",
    ),
    (
        "Hero.frame.premium",
        "a premium hero action",
        "bg-linear-to-b from-sheen to-transparent to-48% bg-gold text-gold-fg",
    ),
    (
        "Hero.frame.secondary",
        "a secondary hero action",
        "bg-none bg-raised-lo text-fg inset-ring-hairline inset-ring-border-strong",
    ),
    (
        "Hero.frame.danger",
        "a destructive hero action",
        "bg-linear-to-b from-sheen to-transparent to-48% bg-danger text-danger-fg",
    ),
    (
        "Hero.frame.ready",
        "a hero action whose charge is full: lit, and breathing while it waits",
        "shadow-glow animate-breathe",
    ),
    (
        "Hero.label",
        "a hero action's words",
        "text-lg uppercase text-center",
    ),
    (
        "Hero.charge",
        "the track a hero action's charge fills along its foot",
        "absolute inset-x-2 bottom-1 h-1 rounded-capsule bg-scrim overflow-hidden",
    ),
    (
        "Hero.charge.fill",
        "the charge filling it",
        "h-full rounded-capsule bg-fg-inverse",
    ),
    (
        "Hold.fill",
        "a hold-to-confirm button's progress along its foot",
        "h-0.5 bg-fg",
    ),
    (
        "Hold.fill.danger",
        "a destructive hold's progress",
        "bg-danger-fg",
    ),
    (
        "Hold.fill.primary",
        "a primary hold's progress",
        "bg-boon-fg",
    ),
    (
        "Ability.chip.charges",
        "the charges or items an ability holds, on a tag at its corner",
        "py-0 px-1.25 text-xs font-bold tabular-nums text-fg rounded-control bg-layer-3 inset-ring-hairline inset-ring-border",
    ),
    (
        "Chip.frame",
        "a tag, badge or chip",
        "rounded-control bg-layer-3 text-fg inset-ring-hairline inset-ring-border",
    ),
    (
        "Field.frame",
        "a text field or input's box (an Input's own, by default): its focus, invalid value, read-only and disabled states",
        "min-h-11 px-3 py-2 rounded-field bg-well text-base text-fg inset-ring-hairline inset-ring-border placeholder:text-fg-subtle data-readonly:bg-transparent aria-invalid:inset-ring-regular aria-invalid:inset-ring-danger focus:inset-ring-regular focus:inset-ring-accent disabled:opacity-50 disabled:inset-ring-border",
    ),
    (
        "Field.area",
        "a multi-line text field's box (an Input with multiline or rows, by default), in the same states",
        "min-h-11 px-3 py-2 rounded-field bg-well text-base text-fg inset-ring-hairline inset-ring-border placeholder:text-fg-subtle data-readonly:bg-transparent aria-invalid:inset-ring-regular aria-invalid:inset-ring-danger focus:inset-ring-regular focus:inset-ring-accent disabled:opacity-50 disabled:inset-ring-border",
    ),
    (
        "Field.label",
        "a labelled field's label (TextField, TextArea)",
        "text-sm font-semibold text-fg",
    ),
    (
        "Field.hint",
        "the description or character count under a field",
        "text-xs text-fg-muted",
    ),
    (
        "Field.error",
        "what is wrong with a field's value, under it",
        "text-xs font-semibold text-danger",
    ),
    (
        "Badge.frame",
        "a status tag: words or a count (Badge)",
        "min-w-5 h-5 px-1.5 rounded-capsule text-2xs font-bold uppercase tracking-label whitespace-nowrap bg-layer-3 text-fg inset-ring-hairline inset-ring-border",
    ),
    (
        "Badge.frame.accent",
        "an accent badge",
        "bg-accent text-accent-fg inset-ring-0",
    ),
    (
        "Badge.frame.success",
        "a success badge",
        "bg-success text-success-fg inset-ring-0",
    ),
    (
        "Badge.frame.warn",
        "a warning badge",
        "bg-warn text-warn-fg inset-ring-0",
    ),
    (
        "Badge.frame.danger",
        "a danger badge (unread, alert)",
        "bg-danger text-danger-fg inset-ring-0",
    ),
    (
        "Badge.frame.info",
        "an informational badge",
        "bg-info text-info-fg inset-ring-0",
    ),
    (
        "Badge.frame.gold",
        "a gold badge (a sale, a premium)",
        "bg-gold text-gold-fg inset-ring-0",
    ),
    (
        "Badge.dot",
        "a badge that is only a dot",
        "size-2.5 rounded-capsule bg-fg-muted ring-regular ring-bg",
    ),
    ("Badge.dot.accent", "an accent dot", "bg-accent"),
    ("Badge.dot.success", "a success dot", "bg-success"),
    ("Badge.dot.warn", "a warning dot", "bg-warn"),
    ("Badge.dot.danger", "a danger dot", "bg-danger"),
    ("Badge.dot.info", "an informational dot", "bg-info"),
    ("Badge.dot.gold", "a gold dot", "bg-gold"),
    ("Divider.line", "a divider's hairline", "bg-border"),
    (
        "Divider.label",
        "the words centred on a divider",
        "text-xs font-semibold uppercase tracking-label text-fg-muted",
    ),
    (
        "IconButton.frame",
        "what an icon-only button adds to its Button plate: the square and its radius",
        "rounded-button",
    ),
    (
        "RichText.keyword",
        "a glossary keyword inside rich text: its underline and focus",
        "rounded-control underline decoration-dotted underline-offset-2 transition duration-instant hover:bg-surface-hover focus-visible:bg-surface-hover focus-visible:inset-ring-regular focus-visible:inset-ring-accent",
    ),
    (
        "RichText.gloss",
        "the tooltip that explains a keyword",
        "flex-col gap-0.5 py-2 px-3 rounded-block bg-surface-overlay text-sm text-fg shadow-overlay inset-ring-hairline inset-ring-border",
    ),
    (
        "CastBar.frame",
        "a cast or channel bar's plate",
        "flex-col gap-1 py-1.5 px-2 rounded-card bg-glass backdrop-blur-regular text-fg inset-ring-hairline inset-ring-border",
    ),
    (
        "CastBar.track",
        "a cast bar's track",
        "relative h-2 rounded-capsule overflow-hidden bg-track shadow-sunk",
    ),
    (
        "CastBar.fill",
        "a cast bar's fill; data-locked: an uninterruptible cast, data-interrupted: a cast broken off",
        "absolute inset-y-0 left-0 rounded-capsule bg-linear-to-r from-hud-accent to-hud-accent-hi transition-[width] duration-instant data-locked:from-fg-muted data-locked:to-fg data-interrupted:from-danger data-interrupted:to-danger",
    ),
    (
        "BannerQueue.item",
        "a non-blocking banner (an achievement, a milestone, a quest unlocked)",
        "flex-row items-center gap-3 py-2.5 pl-2.5 pr-4 min-w-72 max-w-120 rounded-panel bg-glass backdrop-blur-regular text-fg shadow-overlay inset-ring-hairline inset-ring-border transition duration-base ease-out starting:opacity-0 starting:-translate-y-3 data-leaving:opacity-0 data-leaving:-translate-y-3",
    ),
    (
        "HudLayout.region",
        "one placed HUD region inside a HudLayout",
        "",
    ),
    (
        "Row.frame",
        "a row of a list, table or feed",
        "rounded-control bg-layer-3 text-fg",
    ),
    (
        "Row.frame.selected",
        "the chosen row, or the local player's",
        "rounded-control bg-layer-4 text-fg inset-shadow-bar",
    ),
    (
        "Row.frame.muted",
        "a row out of play (disconnected, locked, spent)",
        "rounded-control bg-layer-3 text-fg-muted",
    ),
    (
        "Head.frame",
        "a table or list's head row",
        "rounded-control bg-surface text-fg-muted",
    ),
];
