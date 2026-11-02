// Trees and pan-and-zoom graphs (docs/ui.md §11, Data): a file-like tree
// of expandable items, and a canvas of nodes and links for skill and tech
// trees, run maps and flowcharts.

import type * as Api from "@origingame/engine";
import {
  byText, chevron, target, wrapRow, clamp, fit, flags, frame, grow, kit, lastGesture, mark, over, overProps, px, rememberPath, role, rtl, spring, still, surface, text, token, xy,
} from "../containers/base.ts";
import { edge } from "../containers/panels.ts";

// ── tree ────────────────────────────────────────────────────────────────

/**
 * A tree: items (`id`, `parent`, `label`, `icon`, `badge`) shown as
 * indented rows under their parents; an item with children opens and
 * closes by its chevron or by a press of it (Enter, Space and the pad's A
 * on the focused row), and the arrows walk the rows. The visible rows are virtualised, so a
 * codex of thousands of entries costs a screenful. Answers the item
 * chosen and the expanded set, to keep in game state.
 */
export function tree(id: string, desc: Api.TreeDesc): Api.TreeResult {
  const { t, phone } = frame();
  const items = desc.items ?? [];
  let expanded = [...(desc.expanded ?? [])];
  const open = new Set(expanded);
  let chosen: string | null = null;
  let toggled: string | null = null;
  // The visible rows, depth first, under open parents only.
  const kids = new Map<string, Api.TreeItem[]>();
  for (const it of items) {
    const p = it.parent ?? "";
    let l = kids.get(p);
    if (!l) kids.set(p, (l = []));
    l.push(it);
  }
  const rows: Array<{ it: Api.TreeItem; depth: number; branch: boolean; last: boolean }> = [];
  const walk = (p: string, depth: number): void => {
    const l = kids.get(p) ?? [];
    l.forEach((it, k) => {
      const branch = (kids.get(it.id ?? "")?.length ?? 0) > 0;
      rows.push({ it, depth, branch, last: k === l.length - 1 });
      if (branch && open.has(it.id ?? "")) walk(it.id ?? "", depth + 1);
    });
  };
  walk("", 0);
  const rowH = wrapRow(desc.rowHeight ?? (phone ? 44 : 36));
  const indent = desc.indent ?? 20;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), height: desc.height ?? null, cross: "stretch", part: "Tree.frame", partClass: "panel" });
  try {
    const r = kit.openList("Rows", { count: rows.length, rowHeight: rowH, width: grow(), height: desc.height ? grow() : null, padding: xy(0, 0) }) as Api.ListRange;
    for (let v = r.first; v < r.last; v++) {
      const { it, depth, branch } = rows[v]!;
      const key = it.id ?? "";
      const isOpen = open.has(key);
      const sel = desc.selected === key;
      const resp = kit.listItem(v, sel) as Api.Response;
      try {
        kit.row("Line", { width: grow(), height: grow(), padding: { left: t.xs + depth * indent, top: 0, right: t.sm, bottom: 0 }, gap: t.xs, cross: "center", part: `Tree.item${flags(sel && "selected", branch && (isOpen ? "expanded" : "collapsed"))}`, partClass: "slot" }, () => {
          if (branch) {
            const tr = surface("Toggle", { part: "Tree.toggle", cls: "chip", fill: null, focusable: false, width: px(target(28)), height: px(target(28)), align: "center", cross: "center", semantics: { role: frame().touch ? "button" : "image", name: isOpen ? "Collapse" : "Expand" } }, () => {
              chevron("Chevron", "forward", 14, role("inkMuted"), { group: "expansion", current: isOpen ? "expanded" : "collapsed", turns: { collapsed: 0, expanded: 1 } });
            });
            if (tr.clicked) toggled = key;
          } else kit.spacer("Toggle", px(target(28)));
          if (it.icon) kit.icon("Icon", it.icon, { size: 18, tint: token(sel ? "primary" : "inkMuted"), label: "" });
          text("Label", it.label ?? "", { step: "sm", maxLines: 2, ellipsis: true, width: grow(), color: it.disabled ? "inkFaint" : "ink" });
          if (it.badge) kit.badge("Badge", it.badge, it.badgeTone ?? "neutral");
        });
      } finally {
        kit.close();
      }
      kit.semantics({ role: "treeitem", name: it.label ?? "", selected: sel, ...(branch ? { expanded: isOpen } : {}), value: `level ${depth + 1}`, disabled: it.disabled ?? false });
      if (resp.clicked && !it.disabled) {
        if (branch && desc.toggleOnPress !== false) toggled = key;
        chosen = key;
      }
    }
    kit.closeList();
    kit.semantics({ role: "tree", name: desc.label ?? "" });
  } finally {
    kit.close();
  }
  if (toggled !== null) {
    expanded = open.has(toggled) ? expanded.filter((k) => k !== toggled) : [...expanded, toggled];
    kit.feedback(open.has(toggled) ? "close" : "open", -1);
  } else if (chosen !== null) kit.feedback("select", -1);
  return { chosen, toggled, expanded };
}

// ── graph ───────────────────────────────────────────────────────────────

const STATE_INK: Record<string, Api.ThemeColor> = { locked: "inkFaint", available: "ink", active: "primary", done: "success", current: "highlight" };

/**
 * A pan-and-zoom graph: nodes at positions in the graph's own units and
 * links between them (straight, or curved as a run map's paths), for
 * skill and tech trees, run maps and flowcharts. A drag pans it (with
 * momentum, rubber at its edges), the wheel and two fingers scroll it, and
 * the zoom buttons (and a pinch) scale it about the view's centre between
 * `minZoom` and `maxZoom`; keys and pads walk the nodes, the view
 * following the focus. Each node's state (locked, available, active,
 * done, current) is shown by its ring and a mark as well as its colour;
 * links from a done node are drawn solid, others dashed. Answers the node
 * pressed and the zoom.
 */
export function nodeGraph(id: string, desc: Api.NodeGraphDesc): Api.NodeGraphResult {
  const { t, phone } = frame();
  const nodes = desc.nodes ?? [];
  const links = desc.links ?? [];
  const size = desc.nodeSize ?? (phone ? 52 : 56);
  const minZ = desc.minZoom ?? 0.5;
  const maxZ = desc.maxZoom ?? 2;
  let zoom = clamp(desc.zoom ?? 1, minZ, maxZ);
  let chosen: string | null = null;
  const key = `graph:${id}`;
  // The graph's extent in its own units.
  let x1 = 0;
  let y1 = 0;
  for (const n of nodes) {
    x1 = Math.max(x1, n.x ?? 0);
    y1 = Math.max(y1, n.y ?? 0);
  }
  const pinch = lastGesture(`${key}/Canvas`);
  const pinchHeld = kit.memory(`${id}~pinch`, 1);
  if (pinch && pinch.scale !== 1 && !pinch.held && pinchHeld !== 1) zoom = clamp(zoom * pinchHeld, minZ, maxZ);
  kit.remember(`${id}~pinch`, pinch?.held ? pinch.scale : 1);
  const live = pinch?.held ? clamp(zoom * pinch.scale, minZ, maxZ) : zoom;
  const s = size * Math.max(0.75, Math.min(1.25, live));
  // Room round the nodes' centres for a node and its caption (two lines under it), at the text size.
  const capW = byText(120);
  const capH = byText(2 * 18);
  const labelled = desc.labels !== false;
  // Each caption's width: its text's at the caption size (about 0.56 em an average letter), up to
  // capW, where it wraps. Estimated, not measured: a caption panned out of view has no bounds.
  const em = 13 * Math.max(1, frame().theme.textScale ?? 1);
  const capWs = nodes.map((n) => (n.label && labelled ? Math.min(capW, Math.ceil(n.label.length * em * 0.56) + 4) : 0));
  // The least scale at which no two nodes overlap (kNode) and no two captions either (kCap): a
  // pair is clear when it is far enough apart on either axis, measured at this text size.
  let kNode = 0;
  let kCap = 0;
  if (nodes.length <= 400) {
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const dx = Math.abs((nodes[i]!.x ?? 0) - (nodes[j]!.x ?? 0));
        const dy = Math.abs((nodes[j]!.y ?? 0) - (nodes[i]!.y ?? 0));
        const need = (nx: number, ny: number) => Math.min(dx > 0 ? nx / dx : Infinity, dy > 0 ? ny / dy : Infinity);
        const a = need(s + t.xs, s + t.xs);
        const b = need(Math.max(s, (capWs[i]! + capWs[j]!) / 2) + t.sm, s + 4 + (capWs[i]! > 0 ? capH : 0) + t.xs);
        if (Number.isFinite(a)) kNode = Math.max(kNode, a);
        if (Number.isFinite(b)) kCap = Math.max(kCap, b);
      }
    }
  }
  const padX = Math.max(s / 2, ...capWs.map((w) => w / 2)) + t.sm;
  const padTop = s / 2 + t.sm;
  const padBottom = s / 2 + 4 + capH + t.sm;
  // Zoom 1 fits the whole graph in its view (as laid out last tick): it opens with every node in
  // sight, and zooming in from there pans. Never tighter than its captions can stand apart: past
  // that the graph pans instead, and zoomed out further the captions hide (as maps drop labels).
  const view = kit.bounds(`${id}/Window/View`) ?? kit.bounds(id);
  const fitK = view ? Math.min(1, x1 > 0 ? Math.max(0.1, (view.width - 2 * padX) / x1) : 1, y1 > 0 ? Math.max(0.1, (view.height - padTop - padBottom) / y1) : 1) : 1;
  const k = Math.max(kNode, live * Math.max(fitK, labelled ? kCap : kNode));
  const captions = labelled && k >= kCap - 1e-6;
  const w0 = x1 * k + 2 * padX;
  const h0 = y1 * k + padTop + padBottom;
  // Smaller than its view, it sits in the middle.
  const W = Math.max(w0, view?.width ?? 0);
  const H = Math.max(h0, view?.height ?? 0);
  const ox = (W - w0) / 2;
  const oy = (H - h0) / 2;
  const at = (n: Api.GraphNode) => ({ x: ox + padX + (n.x ?? 0) * k, y: oy + padTop + (n.y ?? 0) * k });
  const byId = new Map(nodes.map((n) => [n.id ?? "", n] as const));
  // The pan: the graph's own offset in its window (a scroll container that clips but never scrolls
  // by itself, the canvas placed over it), so it opens centred on the selected node (else the
  // root: the first node no link leads to) and shows by a fade at each side where more lies.
  const vw = view?.width ?? 0;
  const vh = view?.height ?? 0;
  const maxX = Math.max(0, W - vw);
  const maxY = Math.max(0, H - vh);
  let panX = kit.memory("~panX", 0);
  let panY = kit.memory("~panY", 0);
  // A zoom keeps the view's middle where it was.
  const kWas = kit.memory("~k", 0);
  if (kWas > 0 && Math.abs(kWas - k) > 1e-6 && vw > 0) {
    panX = (panX + vw / 2 - ox - padX) * (k / kWas) + ox + padX - vw / 2;
    panY = (panY + vh / 2 - oy - padTop) * (k / kWas) + oy + padTop - vh / 2;
  }
  const centreOn = (n: Api.GraphNode | undefined) => {
    if (!n || vw <= 0) return;
    const p = at(n);
    panX = p.x - vw / 2;
    panY = p.y - vh / 2;
  };
  // Until the player pans it (a drag, the focus moving off screen), it stays centred there, as
  // its size settles and the text size changes.
  let free = kit.memory("~free", 0) > 0;
  if (view && !free) {
    const led = new Set(links.map((l) => l.to ?? ""));
    centreOn(byId.get(desc.selected ?? "") ?? nodes.find((n) => !led.has(n.id ?? "")) ?? nodes[0]);
  }
  const ges = lastGesture(`${key}/Canvas`);
  const along = (d: number) => (rtl() ? d : -d);
  if (ges && !ges.dragging && kit.memory("~held", 0) !== 0) {
    panX += along(kit.memory("~dx", 0) + ges.velocityX * 0.15);
    panY -= kit.memory("~dy", 0) + ges.velocityY * 0.15;
    free = true;
  }
  kit.remember("~held", ges?.dragging ? 1 : 0);
  kit.remember("~dx", ges?.dragging ? ges.offsetX : 0);
  kit.remember("~dy", ges?.dragging ? ges.offsetY : 0);
  panX = clamp(panX, 0, maxX);
  panY = clamp(panY, 0, maxY);
  let focusedNode = -1;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), height: desc.height ?? px(phone ? 360 : 420), cross: "stretch", radius: t.panel, background: token("well"), part: "Graph.frame", partClass: "slot" });
  try {
    kit.setDraw({
      clip: "shape",
      fills: [
        { kind: "solid", color: token("well") },
        { kind: "gradient", type: "radial", from: { x: 0.5, y: 0.45 }, to: { x: 0.75, y: 0.75 }, stops: [{ offset: 0, color: token("sheen", 0.12) }, { offset: 1, color: token("sheen", 0) }] },
      ],
    });
    // The window the graph pans in, its fades over its cut sides; the zoom controls in a bar under
    // it, so no node ever sits beneath them however the graph is panned or zoomed.
    kit.open("Window", { direction: "column", width: grow(), height: grow(), cross: "stretch" });
    kit.openScroll("View", { width: grow(), height: grow(), axes: "both" });
    try {
      kit.open("Canvas", { width: px(W), height: px(H) });
      // Links under the nodes: one path element each.
      links.forEach((l, k) => {
        const a = byId.get(l.from ?? "");
        const b = byId.get(l.to ?? "");
        if (!a || !b) return;
        const pa = at(a);
        const pb = at(b);
        const x0 = Math.min(pa.x, pb.x);
        const y0 = Math.min(pa.y, pb.y);
        const lw = Math.max(1, Math.abs(pb.x - pa.x));
        const lh = Math.max(1, Math.abs(pb.y - pa.y));
        const p0 = { x: pa.x - x0, y: pa.y - y0 };
        const p1 = { x: pb.x - x0, y: pb.y - y0 };
        const done = a.state === "done" || l.active;
        const curve = desc.curved ?? true;
        const pts = curve ? [p0, { x: p0.x, y: (p0.y + p1.y) / 2 }, { x: p1.x, y: (p0.y + p1.y) / 2 }, p1] : [p0, p1];
        kit.open(kit.idi("Link", k), { width: px(lw), height: px(lh) });
        kit.setDraw({
          shape: { kind: "path", points: pts, curves: curve, width: done ? 3 : 2, cap: "round", join: "round" },
          fills: [{ kind: "solid", color: token(done ? "primary" : "lineStrong", done ? 1 : 0.8) }],
        });
        kit.close();
        over({ insets: { start: rtl() ? W - x0 - lw : x0, top: y0 }, inert: true, z: 0 });
      });
      nodes.forEach((n, k) => {
        const p = at(n);
        const state = n.state ?? "available";
        const locked = state === "locked";
        const ink = STATE_INK[state] ?? "ink";
        const r = surface(kit.idi("Node", k), {
          part: "Graph.node",
          cls: "avatar",
          flags: [state, n.kind ?? "", "round", desc.selected === n.id ? "selected" : ""],
          selected: desc.selected === n.id,
          fill: role(locked ? "panel" : "panelRaised"),
          disabled: n.disabled ?? false,
          width: px(s),
          height: px(s),
          align: "center",
          cross: "center",
          radius: n.shape === "square" ? frame().t.control : s / 2,
          semantics: { role: "button", name: n.label ?? n.id ?? "", value: state, selected: desc.selected === n.id },
          layer: { insets: { start: (rtl() ? W - p.x : p.x) - s / 2, top: p.y - s / 2 }, z: 1 },
        }, () => {
          kit.setDraw({
            shape: n.shape === "hex" ? { kind: "polygon", sides: 6, rounding: 4, rotation: Math.PI / 6 } : n.shape === "diamond" ? { kind: "polygon", sides: 4, rounding: 4 } : { kind: n.shape === "square" ? "rect" : "ellipse" },
            fills: [{ kind: "solid", color: token(locked ? "panel" : "panelRaised") }],
            strokes: [{ width: state === "current" || desc.selected === n.id ? 3 : 2, align: "inside", paint: { kind: "solid", color: token(desc.selected === n.id ? "focus" : ink) } }],
            effects: state === "current" || state === "active" ? [{ kind: "outerGlow", blur: 14, color: token(ink, 0.6) }] : [],
          });
          if (n.icon) kit.icon("Icon", n.icon, { size: s * 0.46, tint: token(locked ? "inkFaint" : ink), label: "" });
          else text("Label", (n.label ?? "").slice(0, 2), { step: "sm", color: locked ? "inkFaint" : "ink", nowrap: true });
          // State in shape too (XAG 103): a lock's bar, a tick, a dot for current.
          if (locked || state === "done" || state === "current") {
            kit.open("State", { width: px(18), height: px(18), radius: 9, align: "center", cross: "center", background: token(locked ? "panelRaised" : state === "done" ? "success" : "highlight") });
            mark("Mark", locked ? "minus" : state === "done" ? "tick" : "dot", locked ? 10 : 12, role(locked ? "inkMuted" : "onPrimary"), 2);
            kit.close();
            over({ insets: { end: -2, bottom: -2 }, inert: true, z: 2 });
          }
        });
        if (n.label && captions) {
          const cw = capWs[k]!;
          // On a backing of the graph's ground: a link running under a caption never strikes it through.
          kit.column(kit.idi("Caption", k), { width: fit(cw, capW), cross: "center", padding: xy(2, 0), radius: 4, background: token("well", 0.92) }, () => {
            text("Text", n.label ?? "", { step: "caption", color: locked ? "inkFaint" : "inkMuted", align: "center", width: grow(), maxLines: 2 });
          });
          over({ insets: { start: (rtl() ? W - p.x : p.x) - cw / 2, top: p.y + s / 2 + 4 }, inert: true, z: 1 });
        }
        if (r.clicked) chosen = n.id ?? null;
        if (r.focused) focusedNode = k;
      });
      kit.close();
      const liveX = ges?.dragging ? clamp(panX + along(ges.offsetX), -24, maxX + 24) : panX;
      const liveY = ges?.dragging ? clamp(panY - ges.offsetY, -24, maxY + 24) : panY;
      const shownX = still() || ges?.dragging ? liveX : kit.animate("~shownX", liveX, spring("ui"));
      const shownY = still() || ges?.dragging ? liveY : kit.animate("~shownY", liveY, spring("ui"));
      kit.props({
        ...overProps({ insets: { start: 0, top: 0 } }),
        transform: { x: rtl() ? shownX : -shownX, y: -shownY },
        pinch: { minScale: minZ / zoom, maxScale: maxZ / zoom },
        drag: { axis: "both", returnHome: true, rubber: 0.55, threshold: 6, spring: { kind: "instant" } },
      });
      rememberPath(`${key}/Canvas`, kit.lastPath());
    } finally {
      kit.close();
    }
    // The fades: over the graph's sides where more of it lies beyond the window.
    const fade = 28;
    edge("FadeStart", true, kit.animate("~fs", panX > 0.5 ? 1 : 0, spring("ui")), true, fade, "well");
    edge("FadeEnd", false, kit.animate("~fe", panX < maxX - 0.5 ? 1 : 0, spring("ui")), true, fade, "well");
    edge("FadeTop", true, kit.animate("~ft", panY > 0.5 ? 1 : 0, spring("ui")), false, fade, "well");
    edge("FadeBottom", false, kit.animate("~fb", panY < maxY - 0.5 ? 1 : 0, spring("ui")), false, fade, "well");
    kit.close();
    // A toolbar of its own under the window, ruled off from it, so the window's lower edge reads as
    // the edge it is.
    kit.row("Bar", { width: grow(), align: "end", padding: xy(t.sm, t.xs), background: token("panel") }, () => {
      kit.row("Zoom", { gap: 2, padding: xy(2, 2), radius: t.control, background: token("panelRaised"), part: "Graph.zoom", partClass: "hud" }, () => {
        const out = surface("Out", { part: "Graph.zoomButton", cls: "button", fill: null, disabled: zoom <= minZ + 1e-3, width: px(target(40)), height: px(target(40)), align: "center", cross: "center", semantics: { role: "button", name: desc.zoomOutLabel || "Zoom out" } }, () => mark("Minus", "minus", 16, role("ink")));
        text("Level", `${Math.round(live * 100)}%`, { step: "caption", font: "mono", color: "inkMuted", nowrap: true });
        const inn = surface("In", { part: "Graph.zoomButton", cls: "button", fill: null, disabled: zoom >= maxZ - 1e-3, width: px(target(40)), height: px(target(40)), align: "center", cross: "center", semantics: { role: "button", name: desc.zoomInLabel || "Zoom in" } }, () => mark("Plus", "plus", 16, role("ink")));
        if (out.clicked) zoom = clamp(zoom / 1.25, minZ, maxZ);
        if (inn.clicked) zoom = clamp(zoom * 1.25, minZ, maxZ);
      });
    });
  } finally {
    kit.close();
  }
  kit.semantics({ role: "group", name: desc.label ?? "", value: `${Math.round(zoom * 100)}%` });
  // A node the focus moves to (keys, a pad) is brought into view.
  if (focusedNode >= 0 && focusedNode !== Math.round(kit.memory("~focused", -1))) {
    const p = at(nodes[focusedNode]!);
    const m = s / 2 + t.lg;
    if (p.x - m < panX) panX = p.x - m;
    else if (p.x + m > panX + vw) panX = p.x + m - vw;
    if (p.y - m < panY) panY = p.y - m;
    else if (p.y + m + capH > panY + vh) panY = p.y + m + capH - vh;
    free = true;
    panX = clamp(panX, 0, maxX);
    panY = clamp(panY, 0, maxY);
  }
  kit.remember("~free", free ? 1 : 0);
  kit.remember("~focused", focusedNode);
  kit.remember("~panX", panX);
  kit.remember("~panY", panY);
  kit.remember("~k", k);
  if (chosen !== null) kit.feedback("select", -1);
  return { chosen, zoom, changed: zoom !== (desc.zoom ?? 1) };
}
