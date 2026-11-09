// A style pack's core-control state sheet (docs/ui.md §6.2): `og ui sheet
// <pack>` installs the pack here and renders each page with it, at desktop
// and phone, 100 % and 150 % text. The core default draws it without one.
//
// Hover, pressed and focus-visible are forced on their cells every tick
// (og.ui.forceState), so a frame shows every state at once.
//
// Commands: style:<pack>, page:<controls|fields|surfaces|overlays>.
import { og, defineSystem } from "@origingame/engine";

type Forced = "hover" | "pressed" | "focus";
const FORCED: ReadonlyArray<readonly [Forced, string]> = [["hover", "hover"], ["pressed", "pressed"], ["focus", "focused"]];
/** Each row's cells (assets/ui/sheet.svelte) by page, as the probe paths name them. */
const CONTROLS = ["primary", "secondary", "quiet", "ghost", "danger", "checkbox", "switch", "radio"];
const FIELDS = ["tab", "field", "select", "stepper", "range"];
const box = (page: string, row: string, c: Forced): [string, string, Forced] => [row, `${page}/${row}/${row}-boxes/${row}-${c}-box/${row}-${c}`, c];
const cells = (): Array<[string, string, Forced]> => [
  ...CONTROLS.flatMap((row) => FORCED.map(([c]) => box("StyleSheet", row, c))),
  ...FIELDS.flatMap((row) => FORCED.map(([c]) => box("FieldSheet", row, c))),
];
/** Where a row's control takes the pointer and the focus, below its cell; the cell itself when absent. */
const PARTS: Record<string, Partial<Record<Forced, readonly string[]>>> = {
  radio: { hover: ["Options/Option#0"], pressed: ["Options/Option#0"], focus: ["Options/Option#0"] },
  select: { hover: ["Trigger"], pressed: ["Trigger"], focus: ["Trigger"] },
  stepper: { hover: ["Frame/Inc"], pressed: ["Frame/Inc"], focus: ["Frame/Value"] },
  range: { hover: ["Track", "Track/Knob"], pressed: ["Track", "Track/Knob"], focus: ["Track", "Track/Knob"] },
  tab: { hover: ["Strip/Track/Item#0"], pressed: ["Strip/Track/Item#0"], focus: ["Strip/Track/Item#0"] },
};
const STATE = new Map(FORCED);
const TARGETS = cells().flatMap(([row, at, c]) =>
  (PARTS[row]?.[c] ?? [""]).map((part) => [`sheet:sheet/${part ? `${at}/${part}` : at}`, STATE.get(c)!] as const));

const ForceStates = defineSystem("ForceStates", { reads: [] }, () => {
  for (const [path, state] of TARGETS) og.ui.forceState(path, "interaction", state);
});

export default og.game({
  systems: [ForceStates],
  start(world) {
    const font = og.load.font(world, "fonts/InstrumentSans-Regular.ttf");
    const icons = og.load.texture(world, "icons/ink-icons.png");
    if (!font.ok) throw new Error(font.reason);
    if (!icons.ok) throw new Error(icons.reason);
    og.ui.configure(world, {
      font: font.value,
      icons: { texture: icons.value, columns: 8, rows: 4, names: [
        "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem", "key",
        "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back", "bag",
        "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
      ] },
    });
    const loaded = og.load.document(world, "ui/sheet.svelte");
    if (!loaded.ok) throw new Error(loaded.reason);
  },
  ui(world) {
    const scope = og.ui.scope(world, "sheet");
    const mounted = scope.mount("sheet", "styleSheet", { page: scope.writable("page", "controls") });
    if (!mounted.ok) throw new Error(mounted.reason);
  },
  commands: {
    style(world, args) {
      const loaded = og.load.style(world, `styles/${args[0]}`);
      if (!loaded.ok) throw new Error(loaded.reason);
      og.ui.configure(world, { style: loaded.value });
    },
    page(world, args) {
      const page = ["fields", "surfaces", "overlays"].includes(args[0] ?? "") ? args[0]! : "controls";
      og.ui.scope(world, "sheet").writable("page", "controls").set(page);
    },
  },
  commandHelp: {
    style: { usage: "<pack>", doc: "Dress the sheet with an installed style pack." },
    page: { usage: "<controls|fields|surfaces|overlays>", doc: "The page shown." },
  },
});
