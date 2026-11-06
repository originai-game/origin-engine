// The strategy kit's resource bar: read against StarCraft II's and
// Civilization's resource bars (amount, a signed rate, supply at its cap).
// Moved from the engine's hud/scores.ts (docs/ui.md §17).

import type * as Api from "@origingame/engine";
import { a11y, anchorOf, box, chip, close, fitLevel, hudRoom, open, edge, formatNumber, frame, hudPlace, icon, keyId, kit, leaf, lit, msToTicks, pop, pulse, px, role, rtl, sel, solid, spring, text, token, ts, u, stepSize } from "@origingame/engine/ui-parts/hud";
import type { ResourceBarDesc, ResourceItem } from "../types.ts";

export function resourceBar(id: string, desc: ResourceBarDesc = {}): void {
  const tk = frame().t;
  const anchor = anchorOf(desc, "top");
  hudPlace(id, anchor, (inAnchor) => {
    // Too wide for its room (a phone's screen; in the flow, the box it is laid
    // in as of last tick), it folds: first each rate under its amount, then
    // the items wrap onto a second line.
    const parent = inAnchor ? null : kit.bounds(kit.openPath());
    const room = parent && parent.width > 0 ? Math.min(hudRoom(), parent.width) : hudRoom();
    const fit = fitLevel(inAnchor ? `${id}Hud/${id}` : [kit.openPath(), id].filter(Boolean).join("/"), room, 2);
    const pad = fit > 0 ? tk.sm : tk.md;
    box(id, {
      direction: "row", gap: fit > 0 ? tk.md : tk.lg, cross: "center", wrap: fit > 1,
      ...(fit > 1 ? { width: px(room), align: "center" } : {}),
      padding: { left: pad, right: pad, top: u(5), bottom: u(5) }, part: sel("ResourceBar", "frame", fit > 0 ? "compact" : null), partClass: "hud",
    }, chip(), () => {
      for (const r of desc.items ?? []) {
        const cap = r.capacity ?? null;
        const amount = r.amount ?? 0;
        const capped = cap !== null && amount >= cap;
        const near = cap !== null && !capped && amount >= cap * 0.85;
        const fmt: Api.NumberFormatDesc = r.format ?? { notation: "compact" };
        box(keyId(r.key ?? r.name ?? ""), { direction: "row", gap: u(6), cross: "center", part: sel("ResourceBar", "item", capped ? "capped" : near ? "near" : null) }, undefined, () => {
          if (r.icon) icon("Icon", r.icon, u(20), r.icon === "coin" || r.icon === "gold" ? lit(role("gold")) : token("ink"));
          // Folded, the amount and its rate stack.
          if (fit > 0) open("Values", { direction: "column", gap: 0, cross: "start" });
          // The amount counts to its value on the ui spring.
          const shown = spring("amount", amount);
          const e = edge("amount", amount);
          const s = e.since < msToTicks(300) && amount > e.prev ? pop(e.since, 0.1, 300) : 1;
          const ink = capped ? lit(role("danger")) : near ? lit(role("warn")) : token("ink");
          // The amount and its cap on one line, folded or not.
          if (cap !== null) open("Count", { direction: "row", gap: 0, cross: "end" });
          // Amount then cap, left to right in every direction: the row mirrors under right to left, so there the cap is
          // built first (an Arabic HUD read 800 / 640 for 640 of 800).
          const capText = (): void => {
            if (cap !== null) text("Cap", `${fit > 0 ? "/" : " / "}${formatNumber(cap, fmt)}`, { step: ts(15), font: "mono", color: capped || near ? ink : token("inkMuted"), nowrap: true, outline: 1 });
          };
          if (rtl()) capText();
          text("Amount", cap !== null ? formatNumber(Math.round(shown), fmt) : formatNumber(Math.abs(amount - shown) < 0.5 ? amount : Math.round(shown), fmt), {
            step: ts(15), font: "mono", color: ink, nowrap: true, outline: 1,
          }, { transform: { scaleX: s, scaleY: s, pivotY: 1 }, ...(capped ? { opacity: 0.6 + 0.4 * pulse(1.5) } : {}) });
          // A cap reads as amount / cap with the cap a step quieter, unless the cap is near or reached (folded, without the spaces).
          if (!rtl()) capText();
          if (cap !== null) close();
          const rate = r.rate ?? 0;
          if (desc.rates !== false && rate !== 0) {
            // The direction as a drawn mark (no font need carry ▲▼), then the amount a second.
            const rateInk = lit(rate > 0 ? role("success") : role("danger"));
            box("Rate", { direction: "row", gap: u(2), cross: "center", part: sel("ResourceBar", "rate", rate > 0 ? "up" : "down") }, undefined, () => {
              leaf("Mark", { width: px(u(7)), height: px(u(6)) }, { shape: { kind: "polygon", sides: 3, rotation: rate > 0 ? 0 : Math.PI }, fills: [solid(rateInk)] });
              text("Text", `${formatNumber(Math.abs(rate), { ...fmt, decimals: Math.abs(rate) < 100 && !Number.isInteger(rate) ? 1 : 0 })}${r.rateSuffix ?? "/s"}`, {
                step: ts(11), font: "mono", color: rateInk, nowrap: true, outline: 0.8,
              });
            });
          }
          if (fit > 0) close();
        }, { a11y: a11y("text", r.name ?? r.key ?? "", { value: `${formatNumber(amount, fmt)}${cap !== null ? ` of ${formatNumber(cap, fmt)}` : ""}${rate(r)}` }) });
      }
      const clock = desc.clock ?? null;
      if (clock !== null) {
        leaf("Rule", { width: px(1), height: px(Math.ceil(stepSize(ts(15)).line * 0.8)), part: sel("ResourceBar", "rule") }, { fills: [solid(token("line", 0.9))] });
        const t = Math.max(0, Math.floor(clock));
        const h = Math.floor(t / 3600);
        const mm = String(Math.floor((t % 3600) / 60)).padStart(h > 0 ? 2 : 1, "0");
        text("Clock", `${h > 0 ? `${h}:` : ""}${mm}:${String(t % 60).padStart(2, "0")}`, { step: ts(15), font: "mono", color: token("ink"), nowrap: true, outline: 1 }, { a11y: a11y("timer", "Game time") });
      }
    }, { a11y: a11y("list", "Resources") });
  });
}

function rate(r: ResourceItem): string {
  const v = r.rate ?? 0;
  return v === 0 ? "" : `, ${v > 0 ? "+" : ""}${v}${r.rateSuffix ?? "/s"}`;
}
