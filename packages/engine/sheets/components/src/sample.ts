// An isolated fixture for checking states without mounting the whole catalogue.
import { og, type UiReadable, type World } from "@origingame/engine";
import { FIXTURES, NAMES } from "./fixtures.ts";
import { BASE_FIXTURES, BASE_NAMES } from "./base-fixtures.ts";
import { META_FIXTURES, META_NAMES } from "./meta-fixtures.ts";

const LISTS = ["items", "pages", "shots", "bars", "ticks", "columns", "rows", "entries", "stars", "buffs", "debuffs", "stats", "compareStats", "hints", "steps", "rewards", "messages", "attachments"];
const MEMBERS = ["breadcrumbs", "carousel", "celebration", "chart", "dataGrid", "menu", "ribbon", "skeletonGroup", "spinner", "splitView", "starBar", "statusGrid", "tooltipCard", "actionHints", ...Object.keys(BASE_NAMES), ...Object.keys(META_NAMES)];

export function sample(world: World, args: readonly string[]): void {
  const member = args[0] || "tooltipCard";
  const state = args[1] || "default";
  const fixture = (META_FIXTURES[member] || BASE_FIXTURES[member] || FIXTURES[member])?.[state];
  if (!MEMBERS.includes(member) || !fixture) throw new Error(`Unknown panel or feedback fixture: ${member} ${state}`);
  const scope = og.ui.scope(world, `sample-${member}-${state}`);
  const inputs: Record<string, UiReadable<unknown>> = {
    member: scope.writable("member", member),
    caption: scope.writable("caption", `${META_NAMES[member] || BASE_NAMES[member] || NAMES[member]} · ${state}`),
    sample: scope.writable("sample", fixture),
  };
  for (const prop of LISTS) {
    const rows = (fixture[prop] || []) as Record<string, unknown>[];
    const key = rows.length && !("id" in rows[0]!) ? "action" : "id";
    inputs[`sample_${prop}`] = scope.collection(prop, rows, { key });
  }
  const mounted = og.ui.scope(world, "sheet").mount("sheet", "UngroupedSamplePage", inputs);
  if (!mounted.ok) throw new Error(mounted.reason);
}
