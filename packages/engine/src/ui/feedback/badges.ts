// og.ui.countBadge: the count, dot or word on the corner of an icon, a tab
// or a button (docs/ui.md §2 "badges"). Read against the mail and friend
// badges of Genshin Impact's and Fortnite's menus and the platforms' app
// badges: a pill straddling the corner, cut from what it sits on by a ring
// of the ground colour, 99+ past its max, a pop when the count rises.

import type * as Api from "@origingame/engine";
import { decimalNumber } from "../data/format.ts";
import { a11y, box, bp, edge, groups, inkOn, kit, lit, pop, role, sel, solid, sound, stroke, text, toneRole, token, xy , ts, stepSize } from "./core.ts";

export function countBadge(id: string, desc: Api.CountBadgeDesc = {}): void {
  const kind: Api.BadgeKind = desc.kind ?? "count";
  const count = desc.count ?? 0;
  const max = desc.max || 99;
  const label = kind === "count" ? (count > max ? `${decimalNumber(max, 0)}+` : decimalNumber(count, 0)) : kind === "label" ? (desc.text ?? "") : "";
  const parent = kit.openPath();
  // The rise is watched even while hidden, so the first count pops.
  const e = edge(`${id}~count`, count);
  if ((kind === "count" && count <= 0) || (kind === "label" && !label)) return;
  const phone = bp().phone;
  // A badge with text grows with the player's text size, so its number stays inside the pill.
  const h = kind === "dot" ? 10 : Math.max(phone ? 18 : 20, Math.ceil(stepSize(ts(13)).line));
  const tone: Api.Tone = desc.tone ?? "danger";
  const fill = role(toneRole(tone, "danger"));
  const rose = e.since < 60 && count > e.prev;
  sound("notification", e.changed && count > e.prev && desc.pulse !== false);
  const s = desc.pulse === false || !rose ? 1 : pop(e.since, 0.35, 380);
  const corner: Api.Anchor = desc.corner ?? "topRight";
  const o = -h / 2;
  const insets: Api.Edges = {
    ...(corner.startsWith("bottom") ? { bottom: o } : { top: o }),
    ...(corner.endsWith("Left") ? { start: o } : { end: o }),
  };
  box(id, {
    height: { kind: "fixed", px: h },
    width: kind === "dot" ? { kind: "fixed", px: h } : { kind: "fit", min: h },
    padding: xy(kind === "dot" ? 0 : label.length > 1 ? 5 : 0, 0),
    align: "center",
    cross: "center",
    part: sel("CountBadge", "base", kind, tone),
    partClass: "badge",
  }, {
    shape: { kind: "rect", radii: [h / 2, h / 2, h / 2, h / 2] },
    fills: [solid(lit(fill))],
    // The ring in the ground colour cuts it from what it sits on.
    strokes: [stroke(2, token("panel"), "outside")],
  }, () => {
    if (kind !== "dot") text("Text", label, { step: ts(13), font: "mono", nowrap: true, color: lit(inkOn(fill)) });
  }, {
    position: "absolute",
    insets,
    z: 3,
    overlaps: parent ? [parent] : [],
    transform: { scaleX: s, scaleY: s },
    a11y: a11y("status", desc.name || label || "New"),
    ...groups({ count: count > max ? "overflow" : count > 0 || kind !== "count" ? "some" : "none" }, { count: ["none", "some", "overflow"] }),
  });
}
