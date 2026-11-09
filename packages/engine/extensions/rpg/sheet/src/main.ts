// The rpg kit's review sheet (docs/ui.md §6.2): each page composes the
// kit's documents, installed as a game installs them (ui-kits/rpg/), from
// the fixtures in fixtures.ts, dressed by a style pack.
//
// Commands: style:<pack>, page:<inventory|journal>.
import { og } from "@origingame/engine";
import * as F from "./fixtures.ts";

const KIT = ["inventory", "equipment", "journal", "tracker", "dialogue", "party", "turn-order", "world-map", "compass"];
const ART = ["hero", "mage", "rogue", "cleric", "speaker", "map"] as const;
const art = {} as F.Art;

export default og.game({
  start(world) {
    for (const doc of KIT) {
      const loaded = og.load.document(world, `ui-kits/rpg/${doc}.svelte`);
      if (!loaded.ok) throw new Error(loaded.reason);
    }
    const sheet = og.load.document(world, "ui/rpg-sheet.svelte");
    if (!sheet.ok) throw new Error(sheet.reason);
    for (const name of ART) {
      const t = og.load.texture(world, `art/${name}.png`);
      if (!t.ok) throw new Error(t.reason);
      art[name] = t.value as unknown as number;
    }
  },
  ui(world) {
    const scope = og.ui.scope(world, "sheet");
    // A component's props as one store; its lists as the keyed collections
    // `<store>.<prop>` that `{$store.prop}` reads.
    const props = (name: string, value: Record<string, unknown>) => ({
      [name]: scope.writable(name, value),
      ...Object.fromEntries(Object.entries(value).filter(([, v]) => Array.isArray(v))
        .map(([k, v]) => [`${name}.${k}`, scope.collection(`${name}.${k}`, v as Array<{ id: string }>)])),
    });
    const mounted = scope.mount("sheet", "rpgSheet", {
      page: scope.writable("page", "inventory"),
      ...props("inv", F.inventory()),
      ...props("eq", F.equipment(art)),
      ...props("jr", F.journal()),
      ...props("tr", F.tracker()),
      ...props("dl", F.dialogue(art)),
      ...props("pa", F.party(art)),
      ...props("to", F.turns(art)),
      ...props("mp", F.map(art)),
      ...props("cp", F.compass()),
    });
    if (!mounted.ok) throw new Error(mounted.reason);
  },
  commands: {
    style(world, args) {
      const loaded = og.load.style(world, `@styles/${args[0]}/style.ogt`);
      if (!loaded.ok) throw new Error(loaded.reason);
      og.ui.configure(world, { style: loaded.value });
    },
    page(world, args) {
      og.ui.scope(world, "sheet").writable("page", "inventory").set(args[0] === "journal" ? "journal" : "inventory");
    },
  },
  commandHelp: {
    style: { usage: "<pack>", doc: "Dress the sheet with a style pack." },
    page: { usage: "<inventory|journal>", doc: "The page shown." },
  },
});
