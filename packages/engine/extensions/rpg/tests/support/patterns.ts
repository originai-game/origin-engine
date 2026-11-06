import { og } from "@origingame/engine";

import type * as Api from "@origingame/engine";

import { rpg, type ConversationDesc, type ConversationResult, type GridItem, type InventoryGridResult, type InventoryMove, type InventoryPick } from "@origingame/engine/extensions/rpg/index.ts";

import { run, storyRun, ok, centre, byPath } from "../../../../tests/support/patterns.ts";

// ── inventory grid ──────────────────────────────────────────────────────

export const BAG: GridItem[] = [
  { key: "sword", name: "Ashbringer", x: 0, y: 0, w: 1, h: 3, rarity: "legendary", tag: "weapon" },
  { key: "potions", name: "Potion", x: 2, y: 0, count: 6, maxStack: 20, tag: "potion" },
  { key: "potions2", name: "Potion", x: 3, y: 0, count: 4, maxStack: 20, tag: "potion" },
  { key: "ring", name: "Ring", x: 2, y: 1, tag: "ring", locked: true },
  { key: "helm", name: "Helm", x: 4, y: 0, w: 2, h: 2, tag: "helmet", rarity: "rare" },
];

/** A grid of 8×4 cells of 50 px over BAG, the game applying each move as it should. */
export function bag(rtl = false) {
  const items = BAG.map((i) => ({ ...i }));
  const moves: InventoryMove[] = [];
  const results: InventoryGridResult[] = [];
  let pick: InventoryPick | null = null;
  const engine = run(() => {
    const r = rpg.inventoryGrid("Bag", { columns: 8, rows: 4, cellSize: 50, items, label: "Backpack", ...(pick ? { pick } : {}) });
    pick = null;
    og.ui.button("Below", "Sort", {});
    results.push(r);
    if (r.move) {
      moves.push(r.move);
      const it = items.find((i) => i.key === r.move!.key)!;
      if (r.move.mergeInto) {
        const into = items.find((i) => i.key === r.move!.mergeInto)!;
        into.count = (into.count ?? 1) + r.move.count;
        if (r.move.split) it.count = (it.count ?? 1) - r.move.count;
        else items.splice(items.indexOf(it), 1);
      } else if (r.move.split) {
        it.count = (it.count ?? 1) - r.move.count;
        items.push({ ...it, key: `${it.key}-${moves.length}`, count: r.move.count, x: r.move.x, y: r.move.y });
      } else {
        Object.assign(it, { x: r.move.x, y: r.move.y, rotated: r.move.rotated });
      }
    }
  }, undefined, rtl);
  return { engine, items, moves, results, setPick: (p: InventoryPick) => (pick = p) };
}

/** The smith's conversation on screen, the game passing the answer on to og.story as it should. */
export function smith(desc: Partial<ConversationDesc> = {}, fake?: (v: Api.DialogueView) => Api.DialogueView) {
  let conv = -1;
  const results: ConversationResult[] = [];
  const engine = storyRun((w) => {
    if (conv < 0) conv = ok<number>(og.story.start(w, { dialogue: "smith" }));
    const v0 = og.story.view(w, conv);
    const v = v0 && fake ? fake(v0) : v0;
    const r = rpg.conversation("Talk", { view: v, ...desc });
    results.push(r);
    if (r.advanced) og.story.advance(w, conv);
    if (r.chosen !== null) og.story.choose(w, conv, r.chosen);
  });
  const tapEl = (end: string) => {
    engine.probe.commands(`0:touch_move:${centre(byPath(engine, end)!)};0.05:touch_start;0.1:touch_end`);
    engine.advance(10);
  };
  return { engine, results, view: () => og.story.view(engine.world, conv)!, tapEl };
}
