// @origingame/engine/ui-parts/screen: the helpers the engine's screen
// patterns are written with (surfaces, buttons, symbols, art slots, price
// tags, a phone's room, the component's memory), public for kits
// (docs/ui.md §4). A kit's screen component imports these instead of
// engine internals; each is built from og.ui's public members and keeps no
// state outside the UI's memory, so it behaves the same on the web and
// native hosts.
//
// This file only names them: the implementations stay with the common
// components that share them. A name here is a contract within a minor
// version; a rename or a changed signature is a breaking changelog entry.

export { accent, artSlot, btn, chevron, clamp, coinFill, countdown, eyebrow, fill, firstRoom, fit, fitStep, flags, floating, floatingContent, frame, glow, grouped, grow, hex, inkFor, inkOn, inside, isPhone, keyHash, keyOf, kit, lastInteraction, line, mix, over, overProps, paint, parentOf, pct, pips, pointer, priceTag, px, radial, rarityColor, rarityPips, recallPath, rememberPath, rememberRoom, role, rtl, safeHeight, safeWidth, scrimFill, shadow, solid, spring, st, still, surface, symbol, teamColor, text, timerChip, timerRing, token, type Mut, type Rgba, type SymbolKind, type TimeWords, u, vgrad, vstate, xy } from "../ui/patterns/core.ts";
export { nameText, stepSize, tps } from "../ui/feedback/core.ts";
export { formatNumber } from "../ui/data/format.ts";
