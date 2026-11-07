// Rewards: pick one of N (docs/ui.md §2 genre patterns: pick
// one of N, reroll, banish). The rewards are cards dealt in face up; a
// press takes one, or selects it when the choice confirms or takes
// several. What is armed (banishing) is the component's own state
// (og.ui.memory); what the player took is answered for the game to apply.

import type * as Api from "@origingame/engine";
import {
  artSlot, btn, frame, glow, grow, keyHash, kit, lastInteraction, line, mix, over, paint, pips, px, rarityColor, rarityPips, role, rtl, shadow, st, still,
  flags, inkOn, spring, surface, symbol, text, token, u, vgrad, vstate, xy, type Mut, type Rgba,
} from "./core.ts";

/** A card's rules as words for a screen reader: the markup's tags left out. */
export function plainText(markup: string): string {
  return markup.replace(/\[\[/g, "\u0000").replace(/\[[^\]]*\]/g, "").replace(/\u0000/g, "[").replace(/\s+/g, " ").trim();
}

/** A small tag in a corner (New, Upgrade): ink on the highlight, outlined so it reads on any art; `up` leads it with an up arrow. */
export function cornerTag(id: string, s: string, tone: Api.ThemeColor | Rgba = "highlight", up = false): void {
  // A tier's own colour (a pull's New on a legendary card) takes the ink that reads on it.
  const fill = typeof tone === "string" ? role(tone) : tone;
  const ink = inkOn(fill);
  kit.open(id, { direction: "row", gap: u(3), cross: "center", padding: xy(u(8), u(3)), part: "Reward.tag", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: paint(fill) }], strokes: [line(1, token("shadow", 0.6), "outside")], effects: [shadow(2, 4, 0.45)] });
  if (up) symbol("Up", "arrowUp", u(12), ink);
  text("Text", s, { step: st(12), color: ink, nowrap: true, font: "display" });
  kit.close();
}
