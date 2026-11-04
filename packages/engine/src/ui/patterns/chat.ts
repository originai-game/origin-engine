// The chat window (docs/ui.md §11 genre patterns, Appendix B: guild,
// friends, chat; Appendix C: the chat window; Appendix D: chat links,
// mentions, channels). Read against World of Warcraft's and Final Fantasy
// XIV's chat frames and Valorant's HUD chat: channel tabs with unread
// counts, each line its time, its channel's tag, the author in the
// channel's ink and the words as typed, item links in their tier's ink,
// @mentions lit, the log following the newest line unless the player has
// scrolled up, the input with its channel chip and mention suggestions.
// The messages, the channels and what a send or a link does are the game's.

import type * as Api from "@origingame/engine";
import {
  firstRoom,
  artSlot, btn, clamp, flags, frame, grow, kit, keyHash, line, mix, over, paint, px, rarityColor, rememberRoom, role, safeWidth, st, surface, symbol, text,
  token, u, vgrad, xy, type Mut, type Rgba,
} from "./core.ts";
import { inputField } from "../controls/fields.ts";

/** Player words as markup that reads them literally (richText's `[[` is a bracket). */
const esc = (s: string): string => s.replace(/\[/g, "[[");
const hexOf = (c: Rgba): string => `#${[c.r, c.g, c.b].map((v) => Math.round(clamp(v, 0, 1) * 255).toString(16).padStart(2, "0")).join("")}`;
/** A channel's ink: a theme colour's name or `#rrggbb`; anything else is the ink. */
function inkOf(c: string | undefined): string {
  if (c && /^#[0-9a-fA-F]{6}$/.test(c)) return c.toLowerCase();
  const pal = frame().theme.palette as Record<string, string | undefined>;
  return hexOf(c && pal[c] ? role(c as Api.ThemeColor) : role("ink"));
}
const MENTION = /@([\p{L}\p{N}_]+)/u;

export function chat(id: string, desc: Api.ChatDesc): Api.ChatResult {
  const out: Mut<Api.ChatResult> = { channel: null, sendTo: null, draft: null, send: null, link: null, author: null, closed: false };
  const { t, phone, touch } = frame();
  const hud = !!desc.hud;
  const typing = !hud || !!desc.typing;
  const tick = kit.tick();
  const channels = desc.channels ?? [];
  const byKey = new Map(channels.map((c) => [c.key ?? "", c]));
  const shown = byKey.get(desc.channel ?? "") ?? channels[0] ?? null;
  const all = !!shown?.all;
  const msgs = (desc.messages ?? []).filter((m) => all || !shown || m.channel === shown.key);
  const me = (desc.me ?? "").toLocaleLowerCase();
  const members = desc.members ?? [];
  const known = new Set(members.map((m) => (m.name ?? "").toLocaleLowerCase()));
  const sendable = channels.filter((c) => !c.readOnly && !c.all);
  const target = byKey.get(desc.sendTo ?? "") ?? (shown && !shown.readOnly && !shown.all ? shown : sendable[0]) ?? null;
  const draft = desc.draft ?? "";
  const W = kit.memory(`~ch.w.${id}`, 0) || firstRoom().w;
  const narrow = W < u(560) * Math.min(1.5, Math.max(1, frame().theme.textScale ?? 1));

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(hud ? 6 : 10), padding: hud ? xy(0, 0) : xy(u(phone ? 10 : 14), u(phone ? 10 : 12)), part: `Chat.frame${flags(hud && "hud")}`, partClass: "panel" });
  if (!hud) kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.94) }, vgrad([[0, token("sheen", 0.05)], [1, token("sheen", 0)]])], strokes: [line(1, token("ink", 0.1))], effects: [{ kind: "dropShadow", offset: { x: 0, y: u(8) }, blur: u(28), spread: 0, color: token("shadow", 0.5) }] });
  const hovered = kit.memory("~ch.hover", 0) > 0;

  // The tabs: a chip per channel, its name in its ink while shown, its unread count (the highlight for a mention).
  if (typing && channels.length > 1) {
    if (desc.title && !hud) text("Title", desc.title, { step: phone ? "xl2" : "xl3", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow() });
    kit.openScroll("Tabs", { width: grow(), gap: 0, padding: xy(0, 0), axes: "horizontal", part: "Chat.tabs", partClass: "panel" } as never);
    kit.open("Row", { direction: "row", gap: u(4), cross: "center" });
    for (const c of channels) {
      const k = c.key ?? "";
      const on = !!shown && shown.key === k;
      const ink = inkOf(c.color);
      const unread = on ? 0 : c.unread ?? 0;
      const mention = !on && (c.mentions ?? 0) > 0;
      const r = surface(k, {
        part: `Chat.tab${flags(on && "selected", mention && "mention")}`,
        cls: "tab",
        fill: on ? mix(role("panelRaised"), role("ink"), 0.1) : hud ? { ...role("shadow"), a: 0.55 } : null,
        selected: on,
        selectedFill: mix(role("panelRaised"), role("ink"), 0.1),
        gap: u(6),
        cross: "center",
        padding: xy(u(12), u(touch ? 10 : 6)),
        radius: 999,
        semantics: { role: "tab", name: `${c.name ?? k}${unread > 0 ? `, ${unread} unread` : ""}${mention ? ", mentioned" : ""}`, selected: on },
      }, () => {
        kit.open("Dot", { width: px(u(8)), height: px(u(8)), radius: 999, background: paint(hexRgb(ink)) });
        kit.close();
        text("Text", c.name ?? k, { step: st(14), font: "display", nowrap: true, color: on ? "ink" : "inkMuted" });
        if (unread > 0) {
          kit.row("Count", { padding: xy(u(6), 0), cross: "center", radius: 999, background: mention ? token("highlight") : token("ink", 0.16), part: "Chat.count", partClass: "badge" }, () => {
            text("Text", unread > 99 ? "99+" : String(unread), { step: st(12), font: "mono", nowrap: true, color: mention ? "shadow" : "ink" });
          });
        }
      });
      if (r.clicked && !on) {
        out.channel = k;
        kit.feedback("tab", -1);
      }
    }
    kit.close();
    kit.close();
    kit.semantics({ role: "tablist", name: desc.title || "Chat" });
  }

  // The log: one wrapping line per message; it follows the newest line while the player is at its foot.
  const n = msgs.length;
  const lastN = kit.memory("~ch.n", -1);
  const foot = kit.memory("~ch.foot", 1) > 0;
  let pending = kit.memory("~ch.pend", 0) > 0;
  if (n !== lastN && (foot || lastN < 0)) pending = true;
  const visible = hud && !typing ? msgs.slice(-8) : msgs;
  kit.openScroll("Log", { width: grow(), height: grow(), gap: u(hud ? 2 : 4), padding: xy(0, u(2)), fade: hud ? 0 : 24, fadeInto: "panel", part: "Chat.log", partClass: "panel" } as never);
  let lastPath = "";
  for (const m of visible) {
    const k = m.key ?? "";
    const ch = byKey.get(m.channel ?? "") ?? null;
    const ink = inkOf(ch?.color);
    const seenKey = `~ch.s.${keyHash(k)}`;
    const seenAt = kit.memory(seenKey, -1);
    if (seenAt < 0) kit.remember(seenKey, tick);
    const age = seenAt < 0 ? 0 : tick - seenAt;
    const fadeTicks = (desc.fadeAfter ?? 10) * 60;
    const alpha = hud && !typing && !hovered ? clamp(1 - (age - fadeTicks) / 60, 0, 1) : 1;
    // The words: links and mentions as markup; the rest exactly as typed.
    let mentionsMe = false;
    const words = (m.text ?? "").split(/(\{\d+\}|@[\p{L}\p{N}_]+)/u).map((part) => {
      const li = /^\{(\d+)\}$/.exec(part);
      if (li) {
        const l = m.links?.[Number(li[1])];
        if (!l) return esc(part);
        const c = rarityColor(l.rarity);
        return `[link=L${l.key ?? ""}][color=${c && l.rarity !== "common" ? hexOf(c) : hexOf(role("primary"))}][[${esc(l.label ?? "")}][/color][/link]`;
      }
      const mm = MENTION.exec(part);
      if (mm && mm.index === 0 && mm[0] === part && (known.has(mm[1]!.toLocaleLowerCase()) || mm[1]!.toLocaleLowerCase() === me)) {
        const mine = me !== "" && mm[1]!.toLocaleLowerCase() === me;
        mentionsMe ||= mine;
        return mine ? `[b][color=highlight]${esc(part)}[/color][/b]` : `[color=info]${esc(part)}[/color]`;
      }
      return esc(part);
    }).join("");
    const head = [
      m.time ? `[font=mono][color=inkFaint]${esc(m.time)}[/color][/font] ` : "",
      all && ch && !m.system ? `[color=${ink}][[${esc(ch.name ?? "")}][/color] ` : "",
      m.author && !m.system ? `${m.authorKey ? `[link=U${m.authorKey}]` : ""}[b][color=${ink}]${esc(m.author)}[/color][/b]${m.authorKey ? "[/link]" : ""}: ` : "",
    ].join("");
    const markup = m.system ? `${head}[color=inkMuted]${words}[/color]` : `${head}${m.channel && ch?.color && !m.author ? `[color=${ink}]${words}[/color]` : words}`;
    // Each line a focus stop (what og.ui.scrollIntoView reaches, and how keys and a pad read back).
    kit.open(k, { direction: "row", width: grow(), gap: u(8), cross: "start", padding: xy(u(mentionsMe ? 8 : 4), u(hud ? 2 : 3)), interactive: true, focusable: true, radius: t.small, part: `Chat.line${flags(mentionsMe && "mention", m.own && "own", m.system && "system")}`, partClass: "listItem" });
    if (mentionsMe) {
      kit.setDraw({ shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] }, fills: [{ kind: "solid", color: token("highlight", 0.12) }] });
      kit.open("Edge", { width: px(u(3)), height: grow(), radius: 2, background: token("highlight") });
      kit.close();
      over({ insets: { start: 0, top: u(2), bottom: u(2) }, inert: true });
    } else if (hud) kit.setDraw({ shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] }, fills: [{ kind: "solid", color: token("shadow", 0.45) }] });
    if (alpha < 1) kit.props({ opacity: alpha });
    if (m.avatar !== undefined && m.avatar !== null) artSlot("Avatar", "Chat.avatar", { texture: m.avatar, kind: "portrait", width: px(u(phone ? 24 : 28)), height: px(u(phone ? 24 : 28)), radius: 999 });
    const rt = kit.richText("Text", markup, { step: st(phone ? 14 : 15), color: paint(role("ink")), width: grow() } as Api.RichTextDesc) as Api.RichTextResult;
    if (rt.link) {
      if (rt.link.startsWith("L")) out.link = rt.link.slice(1);
      else if (rt.link.startsWith("U")) out.author = rt.link.slice(1);
      kit.feedback("select", -1);
    }
    kit.close();
    lastPath = kit.lastPath();
    kit.semantics({ role: "listitem", name: `${m.time ? `${m.time} ` : ""}${all && ch ? `[${ch.name ?? ""}] ` : ""}${m.author ? `${m.author}: ` : ""}${(m.text ?? "").replace(/\{(\d+)\}/g, (_, i: string) => `[${m.links?.[Number(i)]?.label ?? ""}]`)}` });
  }
  kit.close();
  // One focus group: keys and a pad walk the messages (reading back scrolls the log), and it remembers where they were.
  kit.props({ focusGroup: { memory: true } });
  const logPath = kit.lastPath();
  kit.semantics({ role: "log", name: shown?.name ?? desc.title ?? "Chat" });
  const lb = kit.bounds(logPath);
  // The pointer over the log keeps a HUD chat's lines from fading (a fine pointer; a finger has none to rest).
  const pt = kit.framePointer();
  kit.remember("~ch.hover", lb && pt.inside && pt.x >= lb.x && pt.x <= lb.x + lb.width && pt.y >= lb.y && pt.y <= lb.y + lb.height ? 1 : 0);
  // Where the newest line stands (last tick's layout): at the foot, the log follows; scrolled up, new ones gather in the pill.
  const b = lastPath ? kit.bounds(lastPath) : null;
  const inView = !lastPath || (!!b && !!lb && b.height > 0 && b.y + b.height <= lb.y + lb.height + 2);
  if (pending && lastPath) {
    kit.scrollIntoView(lastPath, false);
    if (inView && n === lastN) pending = false;
  }
  const atFoot = pending || inView;
  let seen = kit.memory("~ch.seen", n);
  if (atFoot) seen = n;
  const fresh = Math.max(0, n - seen);
  if (fresh > 0 && typing) {
    const r = surface("New", {
      part: "Chat.new",
      cls: "chip",
      fill: role("primary"),
      washBy: 0.12,
      gap: u(6),
      cross: "center",
      padding: xy(u(12), u(touch ? 10 : 5)),
      radius: 999,
      semantics: { role: "button", name: (desc.newLabel ?? "{n} new").replace("{n}", String(fresh)) },
    }, () => {
      text("Text", (desc.newLabel ?? "{n} new").replace("{n}", String(fresh)), { step: st(13), font: "display", nowrap: true, color: "shadow" });
    });
    kit.props({ position: "absolute", insets: { centerX: true, bottom: u(phone ? 64 : 58) } } as never);
    if (r.clicked) {
      pending = true;
      kit.feedback("select", -1);
    }
  }
  kit.remember("~ch.n", n);
  kit.remember("~ch.pend", pending ? 1 : 0);
  kit.remember("~ch.foot", atFoot ? 1 : 0);
  kit.remember("~ch.seen", seen);

  if (typing) {
    // The mention suggestions over the input while an @word is typed.
    const q = /@([\p{L}\p{N}_]*)$/u.exec(draft);
    const offers = q ? members.filter((m) => {
      const nm = (m.name ?? "").toLocaleLowerCase();
      const w = q[1]!.toLocaleLowerCase();
      return nm.startsWith(w) && nm !== w;
    }).slice(0, 5) : [];
    if (offers.length > 0) {
      kit.column("Suggest", { width: grow(), gap: u(2), padding: xy(u(4), u(4)), part: "Chat.suggestions", partClass: "panel" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] }, fills: [{ kind: "solid", color: token("panelRaised", 0.98) }], strokes: [line(1, token("ink", 0.12))] });
        for (const m of offers) {
          const r = surface(m.key ?? "", { part: "Chat.suggestion", cls: "listItem", fill: null, washBy: 0.12, width: grow(), gap: u(8), cross: "center", padding: xy(u(8), u(touch ? 8 : 4)), radius: t.small, semantics: { role: "option", name: m.name ?? "" } }, () => {
            artSlot("Avatar", "Chat.avatar", { texture: m.avatar ?? null, kind: "portrait", width: px(u(24)), height: px(u(24)), radius: 999 });
            text("Name", `@${m.name ?? ""}`, { step: st(14), nowrap: true, ellipsis: true, width: grow() });
          });
          if (r.clicked) {
            out.draft = draft.replace(/@[\p{L}\p{N}_]*$/u, `@${m.name ?? ""} `);
            kit.feedback("select", -1);
          }
        }
      });
      kit.semantics({ role: "listbox", name: "Mentions" });
    }
    // The input: the channel chip in its ink, the field, Send. Large text on a narrow column stacks the field over the
    // chip and Send: side by side the field's placeholder alone outgrows the row.
    const stack = narrow && (frame().theme.textScale ?? 1) > 1.25;
    const channelChip = (): void => {
      if (!target) return;
      const ink = inkOf(target.color);
      const r = surface("Channel", {
        part: "Chat.channel",
        cls: "chip",
        fill: { ...hexRgb(ink), a: 0.16 },
        washBy: 0.12,
        gap: u(6),
        cross: "center",
        height: { kind: "fit", min: Math.max(touch ? 44 : 0, u(36)) },
        // Narrow, the chip gives the field its room: its name is cut short rather than pushing Send out.
        width: narrow && !stack ? ({ kind: "fit", max: Math.round(W * 0.26) } as Api.Size) : null,
        padding: xy(u(10), 0),
        radius: 999,
        disabled: sendable.length < 2,
        semantics: { role: "button", name: `${desc.channelLabel ?? "Channel"}: ${target.name ?? ""}` },
      }, () => {
        text("Text", target.name ?? "", { step: st(13), font: "display", nowrap: true, ellipsis: true, color: hexRgb(ink) });
      });
      if (r.clicked && sendable.length > 1) {
        const i = sendable.findIndex((c) => c.key === target.key);
        out.sendTo = sendable[(i + 1) % sendable.length]!.key ?? null;
        kit.feedback("select", -1);
      }
    };
    // Narrow for its text (a phone), Send is a round button with its arrow, named Send.
    const sendButton = (words: string): boolean => narrow
      ? surface("Send", { part: "Chat.send", cls: "button", fill: !words || !target ? mix(role("primary"), role("panel"), 0.6) : role("primary"), washBy: 0.14, disabled: !words || !target, width: px(Math.max(44, u(40))), height: px(Math.max(44, u(40))), align: "center", cross: "center", radius: 999, semantics: { role: "button", name: desc.sendLabel ?? "Send" } }, () => {
        symbol("Arrow", "arrowUp", u(18), role("shadow"), { weight: 2.5 });
      }).clicked
      : btn("Send", desc.sendLabel ?? "Send", { variant: "primary", disabled: !words || !target, sym: "chat" }).clicked;
    kit.open("Input", { direction: stack ? "column" : "row", width: grow(), gap: u(8), cross: stack ? "stretch" : "center", part: "Chat.input", partClass: "panel" });
    if (!stack) channelChip();
    const f = inputField("Field", draft, { placeholder: desc.placeholder ?? "Say something…", maxChars: desc.maxChars ?? 255, clearable: false, width: grow() });
    if (f.changed && out.draft === null) out.draft = f.value;
    const words = (f.changed ? f.value : draft).trim();
    let send = false;
    if (stack) {
      kit.row("Bar", { width: grow(), gap: u(8), cross: "center" }, () => {
        channelChip();
        kit.spacer("Gap", grow());
        send = sendButton(words);
      });
    } else send = sendButton(words);
    kit.close();
    if ((f.submitted || send) && words && target) {
      out.send = { channel: target.key ?? "", text: words };
      kit.feedback("confirm", -1);
    }
    if (hud && kit.back()) out.closed = true;
  }
  kit.close();
  rememberRoom("ch", id);
  return out;
}

function hexRgb(s: string): Rgba {
  const n = (k: number): number => parseInt(s.slice(1 + k, 3 + k), 16) / 255;
  return { r: n(0), g: n(2), b: n(4), a: 1 };
}
