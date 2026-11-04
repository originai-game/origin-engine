// The backlog and barks (docs/ui.md §11 genre patterns, Appendix B: the VN
// text box's backlog), fed by og.story: og.story.history and og.story.barks
// hand over each line resolved and whole, and these draw them. A bark's
// first tick is the component's own state (og.ui.memory, so a snapshot
// carries it). The conversation moved into the RPG kit (docs/ui.md §17).
//
// Read against the visual novels (Ace Attorney, Persona: the backlog of
// lines with their speakers, a voiced line replayed).

import type * as Api from "@origingame/engine";
import { speechBubble } from "../world/tags.ts";
import { tps } from "../feedback/core.ts";
import {
  accent, artSlot, clamp, fill, flags, frame, glow, grow, hex, kit, keyHash, line, mix, over, paint, px, role, rtl, shadow, spring, st, still, surface,
  symbol, text, token, u, vgrad, vstate, xy, type Mut, type Rgba,
} from "./core.ts";

/**
 * A conversation's backlog (Ren'Py, Persona): see `fn backlog` in
 * crates/og-api/api/ui.ogt.
 */
export function backlog(id: string, desc: Api.BacklogDesc): Api.BacklogResult {
  const out: Mut<Api.BacklogResult> = { closed: false, replay: null };
  const { viewport: vp, phone } = frame();
  const W = vp.width - vp.safeArea.left - vp.safeArea.right;
  const H = vp.height - vp.safeArea.top - vp.safeArea.bottom;
  const lines = desc.lines ?? [];
  const ink = role("ink");
  const pw = phone ? W - u(16) : Math.min(u(860), W - u(96));
  const ph = phone ? H - u(16) : Math.min(H - u(80), u(860));
  kit.open(id, { direction: "column", width: grow(), height: grow(), align: "center", cross: "center", part: "Backlog.scrim", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow", 0.72) }] });
  kit.open("Panel", { direction: "column", width: px(pw), height: px(ph), padding: { left: u(phone ? 16 : 32), right: u(phone ? 16 : 32), top: u(phone ? 16 : 26), bottom: u(phone ? 12 : 22) }, gap: u(14), part: "Backlog.frame", partClass: "panel" });
  kit.setDraw({ shape: { kind: "rect", radii: [u(16), u(16), u(16), u(16)] }, fills: [{ kind: "solid", color: token("panel", 0.97) }], effects: [shadow(u(16), u(40), 0.6)] });
  kit.props({ ...(kit.memory("~b.open", 0) === 0 && !still() ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: u(30) }], transition: spring("panel") } } : {}) });
  if (kit.memory("~b.open", 0) === 0) kit.feedback("open", -1);
  kit.row("Head", { width: grow(), cross: "center", gap: u(12) }, () => {
    text("Title", desc.title ?? "Log", { step: phone ? "xl2" : "xl4", font: "display", heading: true, width: grow(), nowrap: true, ellipsis: true });
    const g = frame().touch ? null : (kit.frameGlyph("back") as Api.UiGlyph | null);
    const r = surface("Close", { part: "Backlog.close", cls: "button", fill: mix(role("panelRaised"), ink, 0.04), gap: u(8), padding: xy(u(14), u(8)), radius: u(8), semantics: { role: "button", name: desc.closeLabel ?? "Close" } }, () => {
      if (g) text("Glyph", g.text, { step: st(12), font: "mono", color: "inkMuted", nowrap: true });
      text("Text", desc.closeLabel ?? "Close", { step: st(15), nowrap: true });
    });
    if (r.clicked) out.closed = true;
  });
  if (kit.action("back")) out.closed = true;
  if (out.closed) kit.feedback("back", -1);
  kit.openScroll("List", { width: grow(), height: grow(), gap: u(18), padding: { left: 0, right: u(10), top: u(4), bottom: u(8) }, fade: 40, fadeInto: "panel" } as never);
  if (lines.length === 0) text("Empty", desc.emptyLabel ?? "Nothing has been said yet.", { step: st(16), color: "inkMuted" });
  let lastPath = "";
  lines.forEach((l) => {
    const narrator = !l.speaker.name;
    const voiced = l.voice !== "";
    kit.open(`L${l.seq}`, { direction: "row", width: grow(), gap: u(14), cross: "start", part: `Backlog.entry${flags(narrator && "narrator", l.seen && "seen")}`, partClass: "card" });
    kit.props({ states: [{ name: "voice", states: [vstate("silent"), vstate("voiced")], transitions: [] }], state: [{ group: "voice", state: voiced ? "voiced" : "silent" }] });
    kit.column("Words", { width: grow(), gap: u(4) }, () => {
      if (!narrator) text("Name", l.speaker.name, { uppercase: true, step: st(13), font: "display", color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
      text("Text", l.text, { step: st(phone ? 16 : 18), color: narrator ? "inkMuted" : "ink", width: grow() });
      if (l.chosen) {
        // The player's reply: set in from the start under a short rule, in the muted ink.
        kit.row("Reply", { gap: u(10), cross: "center", padding: { left: u(14), right: 0, top: u(4), bottom: 0 }, part: "Backlog.reply", partClass: "badge" }, () => {
          kit.open("Rule", { width: px(u(14)), height: px(2) });
          kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("ink", 0.4) }] });
          kit.close();
          text("Text", l.chosen!.text, { step: st(phone ? 15 : 16), color: mix(ink, role("inkMuted"), 0.3), width: grow() });
        });
      }
    });
    if (voiced) {
      const r = surface("Replay", { part: "Backlog.replay", cls: "button", fill: mix(role("panelRaised"), ink, 0.05), width: px(u(36)), height: px(u(36)), align: "center", cross: "center", radius: 999, semantics: { role: "button", name: `${desc.replayLabel ?? "Play voice"}: ${l.text}` } }, () => {
        symbol("Speaker", "speaker", u(16), ink);
      });
      if (r.clicked) {
        out.replay = l.seq;
        kit.feedback("select", -1);
      }
    }
    kit.close();
    lastPath = kit.lastPath();
    kit.semantics({ role: "listitem", name: `${narrator ? "" : `${l.speaker.name}: `}${l.text}${l.chosen ? ` → ${l.chosen.text}` : ""}` });
  });
  kit.close();
  kit.semantics({ role: "log", name: desc.title ?? "Log" });
  // It opens on the newest line (asked until it shows, then left to the player's scrolling).
  const newest = lines.length > 0 ? lines[lines.length - 1]!.seq + 1 : 0;
  if (lastPath && kit.memory("~b.open", 0) !== newest) {
    kit.scrollIntoView(lastPath, false);
    const b = kit.bounds(lastPath);
    const lb = kit.bounds(parentOfPath(lastPath));
    if (b && lb && b.y + b.height <= lb.y + lb.height + 1 && b.height > 0) kit.remember("~b.open", newest);
  } else if (!lastPath) kit.remember("~b.open", 1);
  // Closed, it opens on the newest line again next time (written in the panel's scope, where it is read).
  if (out.closed) kit.remember("~b.open", 0);
  kit.close();
  kit.close();
  kit.semantics({ role: "dialog", name: desc.title ?? "Log" });
  return out;
}

function parentOfPath(p: string): string {
  const k = p.lastIndexOf("/");
  return k < 0 ? "" : p.slice(0, k);
}

/**
 * Barks over their speakers: see `fn barks` in crates/og-api/api/ui.ogt.
 * Each is og.ui.speechBubble, typed from the tick its line began and timed
 * to leave as og.story's time for it runs out.
 */
export function barks(id: string, desc: Api.BarksDesc): void {
  const tick = kit.tick();
  const rate = tps();
  const shoutAt = desc.shoutPriority ?? 5;
  for (const b of desc.barks ?? []) {
    const e = b.speaker.entity || b.entity;
    const k = `~bk.${e}`;
    // The tick its line began, and how long og.story gave it then (a new seq is a new line).
    if (kit.memory(`${k}.seq`, -1) !== b.seq) {
      kit.remember(`${k}.seq`, b.seq);
      kit.remember(`${k}.at`, tick);
      kit.remember(`${k}.len`, tick + b.ticksLeft);
    }
    const at = kit.memory(`${k}.at`, tick);
    const end = kit.memory(`${k}.len`, tick + b.ticksLeft);
    const typeMs = (Array.from(b.text).length / 40) * 1000;
    // Its stay after typing ends so it starts to leave 300 ms before og.story's time for it is up.
    const stayMs = Math.max(400, ((end - at) / rate) * 1000 - typeMs - 300);
    speechBubble(`${id}-E${e}`, {
      target: { entity: e as never, point: { x: 0, y: 0, z: 0 } },
      height: desc.height ?? 2.2,
      text: b.text,
      speaker: b.speaker.name,
      at,
      durationMs: stayMs,
      shout: b.priority >= shoutAt,
      clamp: desc.clamp ?? false,
      maxWidth: desc.maxWidth ?? 0,
    });
  }
}
