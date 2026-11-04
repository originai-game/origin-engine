// @origingame/engine/ui-parts/screen: the helpers the engine's screen
// patterns are written with (surfaces, buttons, symbols, art slots, price
// tags, a phone's room, the component's memory), public for kits
// (docs/ui.md §17.5). A kit's screen component imports these instead of
// engine internals; each is built from og.ui's public members and keeps no
// state outside the UI's memory, so it behaves the same on the web and
// native hosts.
//
// This file only names them: the implementations stay with the common
// components that share them. A name here is a contract within a minor
// version; a rename or a changed signature is a breaking changelog entry.

export {
  accent, artSlot, btn, chevron, clamp, eyebrow, fill, firstRoom, fit, flags, floating, frame, glow, grouped, grow, hex, inside, keyHash, keyOf, kit,
  lastInteraction, line, mix, type Mut, over, overProps, paint, pct, pips, pointer, priceTag, px, radial, rarityColor, rarityPips, recallPath,
  rememberPath, rememberRoom, type Rgba, role, rtl, safeHeight, safeWidth, scrimFill, shadow, solid, spring, st, still, surface, symbol,
  type SymbolKind, text, timerChip, type TimeWords, token, u, vgrad, vstate, xy
} from "../ui/patterns/core.ts";
export { tps } from "../ui/feedback/core.ts";
export { cornerTag, plainText } from "../ui/patterns/rewards.ts";
export { STAR_GOLD } from "../ui/patterns/stars.ts";
