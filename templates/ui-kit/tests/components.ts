// The component previews' scripts (src/components.ts): each screen in its
// states, a frame of each. The desktop and phone playtests share them; each
// state's frame is `<screen>-<state>.png`, compared with
// crates/og-ui/references/<tier>/kit-components[-phone]-<file>.

import { key, tap } from "./lib.ts";

export interface Script {
  commands: string;
  files: string[];
}

/** Builds a script: `at` advances the clock, `shot` takes a frame. */
export function builder(t0: number) {
  const steps: string[] = [];
  const files: string[] = [];
  let t = t0;
  return {
    /** A raw command now, then wait `after` seconds. */
    cmd(c: string, after = 0.1) {
      steps.push(`${t.toFixed(2)}:${c}`);
      t += after;
    },
    /** A helper's steps (tap, key) starting now, then wait `after`. */
    seq(f: (at: number) => string, after = 0.5) {
      steps.push(f(t));
      t += after;
    },
    shot(name: string, settle = 0.6) {
      t += settle;
      steps.push(`${t.toFixed(2)}:screenshot:${name}.png`);
      files.push(`${name}.png`);
      t += 0.1;
    },
    /** Scroll a page's body (a phone's single column) by `dy` CSS px at its centre. */
    scroll(page: string, dy: number) {
      steps.push(`${t.toFixed(2)}:touch_move_to:${page}/Body`);
      steps.push(`${(t + 0.05).toFixed(2)}:scroll:0,${dy}`);
      t += 0.4;
    },
    dump(name: string) {
      steps.push(`${t.toFixed(2)}:dump_ui:${name}.json`);
      t += 0.05;
    },
    end(): Script {
      steps.push(`${t.toFixed(2)}:quit`);
      return { commands: steps.join(";"), files };
    },
  };
}

const OVERLAYS = ["menu", "context", "popover", "dialog", "confirm", "sheet", "sideSheet", "radial"] as const;

/** A pad's bumper pressed: the tab row's bumpers and the action bar in Xbox's glyphs (RB stepped on), then PlayStation's (LB back, the drawn × ○ □ △), the tabs left as they were. */
function padBumpers(b: ReturnType<typeof builder>, name: string): void {
  for (const [device, button] of [["xbox", "rightBumper"], ["playstation", "leftBumper"]] as const) {
    b.cmd(`demo:device:${device}`);
    b.cmd(`gamepad:${button}`, 0.05);
    b.cmd("gamepad:-", 0.2);
    b.shot(`${name}-bumper-${device}`, 0.5);
  }
  b.cmd("gamepad:off");
  b.cmd("demo:device:keyboardMouse");
}

/** Every component screen in its states. `phone` keeps to what a finger does (no hover, no keys). */
export function componentsScript(phone: boolean): Script {
  const b = builder(0.3);
  if (phone) b.cmd("viewport:phone");

  // Containers: idle, a card hovered, pressed and focused, an accordion opened.
  b.cmd("show:containers");
  b.shot("containers-idle", 0.8);
  if (!phone) {
    b.cmd("touch_move_to:Pick#0", 0.1);
    b.shot("containers-hover", 0.4);
    b.cmd("touch_start", 0.05);
    b.shot("containers-press", 0.1);
    b.cmd("touch_end", 0.3);
    b.seq((at) => key(at, "Tab"), 0.2);
    b.shot("containers-focus", 0.3);
  }
  // On a phone the panels are one column: the accordion is below the cards.
  if (phone) b.scroll("Containers", 500);
  b.seq((at) => tap(at, "Section#1/Header"), 0.2);
  b.shot("containers-accordion", 0.6);
  b.dump("containers");

  // Navigation: idle, a bumper stepped the tabs, the carousel moved, the breadcrumbs' menu open.
  b.cmd("show:navigation");
  b.shot("navigation-idle", 0.8);
  if (!phone) {
    b.seq((at) => key(at, "KeyE"), 0.2);
    b.shot("navigation-bumper", 0.5);
    padBumpers(b, "navigation");
  }
  if (phone) {
    // A finger picks a tab (no bumpers under a finger); the carousel is further down.
    // The underline row's tab (the pill row has a Tab#1 too, already selected).
    b.seq((at) => tap(at, "TabsUnderline/Tabs/Bar/Clip/Strip/Tabs/Tab#1"), 0.2);
    b.shot("navigation-tab", 0.5);
    b.dump("navigation-tab");
  } else {
    b.seq((at) => tap(at, "Viewport/Next"), 0.2);
    b.shot("navigation-carousel", 0.7);
  }
  if (phone) b.scroll("Navigation", 500);
  b.seq((at) => tap(at, "Path/More"), 0.2);
  b.shot("navigation-crumbs-menu", 0.5);
  b.dump("navigation");
  b.seq((at) => key(at, "Escape"), 0.3);

  // Overlays: each open in turn, the menu's submenu, the wheel aimed two ways.
  b.cmd("show:overlays");
  b.shot("overlays-closed", 0.6);
  for (const o of OVERLAYS) {
    b.cmd(`demo:overlay:${o}`);
    b.shot(`overlays-${o}`, o === "sheet" || o === "sideSheet" ? 0.9 : 0.5);
    if (o === "menu") {
      b.seq((at) => tap(at, "Menu/Panel/Content/Item#4"), 0.2);
      b.shot("overlays-submenu", 0.5);
      b.dump("overlays-menu");
    }
    if (o === "radial") {
      b.cmd("demo:aim:-0.8,0.5");
      b.shot("overlays-radial-aimed", 0.4);
    }
    b.cmd("demo:overlay:none", 0.4);
  }

  // Data: rows, sorted by a header, then loading, empty and error.
  b.cmd("show:data");
  b.shot("data-rows", 0.8);
  if (!phone) {
    // A list row under the pointer: its hover state (§7.3), beside the selected row.
    b.cmd("touch_move_to:Mail/Rows/Row#3", 0.1);
    b.shot("data-list-hover", 0.4);
    b.cmd("touch_move_to:Data/Body", 0.1);
  }
  b.seq((at) => tap(at, "Header/Col#3"), 0.2);
  b.shot("data-sorted", 0.5);
  b.dump("data");
  for (const s of ["loading", "empty", "error"]) {
    b.cmd(`demo:data:${s}`);
    b.shot(`data-${s}`, 0.5);
  }
  b.cmd("demo:data:rows");

  // Charts: revealed, a bar hovered (its tooltip), then their states.
  b.cmd("show:charts");
  b.shot("charts", 1.2);
  // On a phone the bar chart is the third panel down.
  if (phone) b.scroll("Charts", 560);
  b.seq((at) => tap(at, "Matches/Plot/Area/Category#1"), 0.1);
  b.shot("charts-category", 0.4);
  b.dump("charts");
  for (const s of ["loading", "empty", "error"]) {
    b.cmd(`demo:data:${s}`);
    b.shot(`charts-${s}`, 0.5);
  }
  b.cmd("demo:data:rows");
  // The second page: stacked bars, the radar and the donut, then its empty and error states.
  b.cmd("show:chartsMore");
  b.shot("charts-more", 1.2);
  for (const s of ["empty", "error"]) {
    b.cmd(`demo:data:${s}`);
    b.shot(`charts-more-${s}`, 0.5);
  }
  b.cmd("demo:data:rows");

  // The graph and the calendar: idle, zoomed in, the next month.
  b.cmd("show:graph");
  b.shot("graph", 0.9);
  b.seq((at) => tap(at, "Zoom/In"), 0.2);
  b.shot("graph-zoom", 0.6);
  // On a phone the calendar is under the graph: scrolled into view before its month turns.
  if (phone) b.scroll("Graphs", 600);
  b.seq((at) => tap(at, "Login/Header/Next"), 0.2);
  b.shot("graph-month", 0.7);
  b.dump("graph");

  // At the player's largest text size (200 %, docs/ui.md §15.2): every screen grows with its
  // text and cuts none of it; the overlays open, the data in its states.
  b.cmd("demo:text:200");
  b.cmd("show:containers");
  b.shot("text200-containers", 0.8);
  b.cmd("show:navigation");
  b.shot("text200-navigation", 0.8);
  b.dump("navigation-200");
  if (!phone) padBumpers(b, "text200-navigation");
  if (phone) b.scroll("Navigation", 1100);
  b.seq((at) => tap(at, "Path/More"), 0.2);
  b.shot("text200-navigation-crumbs-menu", 0.5);
  b.seq((at) => key(at, "Escape"), 0.3);
  b.cmd("show:overlays");
  for (const o of ["menu", "popover", "confirm", "sheet", "radial"] as const) {
    b.cmd(`demo:overlay:${o}`);
    b.shot(`text200-overlays-${o}`, o === "sheet" ? 0.9 : 0.5);
    if (o === "menu") {
      b.seq((at) => tap(at, "Menu/Panel/Content/Item#4"), 0.2);
      b.shot("text200-overlays-submenu", 0.5);
    }
    b.cmd("demo:overlay:none", 0.4);
  }
  b.cmd("show:data");
  b.shot("text200-data-rows", 0.8);
  for (const s of ["empty", "error"]) {
    b.cmd(`demo:data:${s}`);
    b.shot(`text200-data-${s}`, 0.5);
    // The page's lower panels (the tree, the grid's and the stats' states), scrolled into view.
    b.scroll("Data", 600);
    b.shot(`text200-data-${s}-end`, 0.5);
    b.scroll("Data", -600);
  }
  b.cmd("demo:data:rows");
  b.cmd("show:charts");
  b.shot("text200-charts", 1.2);
  b.cmd("show:chartsMore");
  b.shot("text200-charts-more", 1.2);
  b.cmd("show:graph");
  b.shot("text200-graph", 0.9);
  b.cmd("demo:text:100");
  return b.end();
}
