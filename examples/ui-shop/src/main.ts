// og-ui's documents, bindings and layer stack (docs/ui.md §8, §9, §10),
// driven from game code: assets/ui/hud.ogt is a HUD layer bound to the
// Wallet resource; assets/ui/shop.ogt is a menu layer bound to a viewmodel
// this file keeps, pushed over the HUD (it dims and pauses what is under
// it, takes the focus and Back, and gives both back when it pops). Code
// fills the shop's Preview slot and answers its Buy event; the shop's
// clicks and its slider write into the viewmodel on their own.
//
// Commands: shop (push the shop), close (pop it), buy.
//
//   og run examples/ui-shop --size 1280x720 --out film --commands "$(cat film.txt)"
//
// film.txt is the filmstrip the U3 report reads: the HUD alone, the shop
// pushed over it, an item selected, a purchase, the shop adjusted live
// with ui:set and saved back with ui:save, and Back returning to the HUD.
// Its ui:save rewrites the four edited lines of assets/ui/shop.ogt (the
// title, the panel's width and background, the gold's colour) and nothing
// else: `git diff` shows them, `git checkout` puts the sample back.

import { og, defineResource, defineSystem, f64, str } from "@origingame/engine";
import type { World } from "@origingame/engine";

/** What the HUD reads (a Capitalised root in hud.ogt: `Wallet.gold`). */
export const Wallet = defineResource("Wallet", {
  gold: f64(1250, { doc: "The player's gold." }),
  essence: str(24, "1.5e400", { doc: "An idle game's essence, past 1e308: a mantissa and an exponent." }),
});

type Item = { name: string; price: number; owned: boolean; tint: string };
type ShopVm = { gold: number; items: Item[]; selected: number; price: number; restock: number; volume: number };

const ITEMS: Item[] = [
  { name: "Ember blade", price: 1200, owned: false, tint: "#e0573d" },
  { name: "Tide shield", price: 800, owned: true, tint: "#3d86e0" },
  { name: "Grove charm", price: 450, owned: false, tint: "#45a861" },
];

const state = { loaded: false, buy: false };

function shopVm(world: World): ShopVm | null {
  return og.ui.screenVm(world, "shop") as ShopVm | null;
}

function open(world: World): void {
  const gold = world.resource(Wallet);
  const vm: ShopVm = {
    gold: gold.ok ? gold.value.gold : 0,
    items: ITEMS.map((i) => ({ ...i })),
    selected: 0,
    price: ITEMS[0]!.price,
    restock: 5400,
    volume: 0.6,
  };
  og.ui.push(world, "shop", vm);
}

/** The purchase: gold leaves the wallet, the item is owned. */
function buy(world: World, vm: ShopVm): void {
  const item = vm.items[vm.selected];
  const wallet = world.resource(Wallet);
  if (!item || item.owned || !wallet.ok || wallet.value.gold < item.price) return;
  wallet.value.gold -= item.price;
  item.owned = true;
}

export const Shop = defineSystem("Shop", { reads: [og.Input], writes: [Wallet] }, (_q, _time, world) => {
  if (!state.loaded) {
    for (const doc of ["ui/hud.ogt", "ui/shop.ogt"]) {
      const r = og.load.document(world, doc);
      if (!r.ok) throw new Error(r.reason);
    }
    og.ui.push(world, "hud");
    state.loaded = true;
  }
  // The HUD is built at the frame's end with the viewmodel it has (none:
  // it reads Wallet). Its Shop button is an event to code.
  for (const e of og.ui.docEvents(world)) if (e.kind === "emit" && e.name === "shop") open(world);
  if (!og.ui.layers(world).some((l) => l.screen === "shop")) return;
  const vm = shopVm(world);
  if (!vm) return;
  if (state.buy) buy(world, vm);
  state.buy = false;
  const wallet = world.resource(Wallet);
  vm.gold = wallet.ok ? wallet.value.gold : 0;
  vm.price = vm.items[vm.selected]?.price ?? 0;
  // Built here, so code fills its slot right after.
  const r = og.ui.screen("shop", vm);
  // The viewmodel was handed over above: the purchase lands next tick, before the build.
  for (const e of r.events) if (e.kind === "emit" && e.name === "buy") state.buy = true;
  if (og.ui.openSlot("shop/Shop/Preview")) {
    const item = vm.items[vm.selected];
    og.ui.row("Card", { gap: 12, cross: "center", padding: { left: 12, top: 8, right: 12, bottom: 8 }, radius: 10, background: { color: { r: 0.09, g: 0.11, b: 0.16 }, alpha: 1 }, width: { kind: "grow" } }, () => {
      og.ui.label("Pick", item ? `${item.name}${item.owned ? " (owned)" : ""}` : "Nothing selected", { step: "lg" });
      og.ui.label("Afford", vm.gold >= (item?.price ?? 0) ? "within your purse" : `${(item?.price ?? 0) - vm.gold} gold short`, { color: { token: "inkMuted" } });
    });
    og.ui.close();
  }
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Shop],
  resources: [Wallet],
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) {
      camera.value.orthographic = true;
      camera.value.height = 360;
      camera.value.background = { r: 0.12, g: 0.2, b: 0.16 };
    }
    const font = og.load.font(world, "fonts/ui.ttf");
    if (font.ok) og.ui.configure(world, { font: font.value });
  },
  commands: {
    shop: (world) => open(world),
    close: (world) => {
      og.ui.pop(world);
    },
    buy: () => {
      state.buy = true;
    },
  },
  commandHelp: {
    shop: { usage: "", doc: "Push the shop over the HUD with a fresh viewmodel." },
    close: { usage: "", doc: "Pop the top layer." },
    buy: { usage: "", doc: "Buy the selected item (as the Buy button does)." },
  },
});
