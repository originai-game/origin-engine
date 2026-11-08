// @origingame/engine/ui-parts/hud: the helpers the engine's HUD components
// are written with (parts a style dresses, the frame's tokens and roles,
// fills, strokes and effects, motion by the tick, HUD placement in the safe
// area), public for kits (docs/ui.md §4). A kit's HUD component imports
// these instead of engine internals; each is built from og.ui's public
// members and keeps no state outside the UI's memory, so it behaves the
// same on the web and native hosts.
//
// This file only names them: the implementations stay with the common
// components that share them. A name here is a contract within a minor
// version; a rename or a changed signature is a breaking changelog entry.

export { BLACK, TAU, WHITE, a11y, absolute, anchorOf, apply, arc, artTile, box, bp, chip, claimAnnouncement, clamp01, close, countdown, drop, easeBack, easeOut, edge, ellipseShape, fillDraw, fitLevel, fixed, formatNumber, frame, glow, groups, hudPlace, hudRoom, hueOf, icon, inkOn, inner, innerGlow, keyId, kit, leaf, lit, luminance, minText, mix, msToTicks, now, open, paintRgba, pct, pop, pulse, px, radial, reduced, role, rtl, sel, shapeOf, shrunk, socketRim, solid, sound, spring, springTo, stepSize, stroke, teamColor, text, textScale, ticksToMs, token, trackDraw, ts, tsFit, type Props, type Rgba, u, ut, vgrad } from "../ui/hud/core.ts";
