// The component library's previews at desktop size (1280×720 CSS px at 1×):
// containers, navigation, the overlays, data views, charts, the graph and
// the calendar in their states (idle, hover, press, focus, open, sorted,
// loading, empty, error), each frame compared with
// crates/og-ui/references/<tier>/kit-components-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { componentsScript } from "./components.ts";
import { ui } from "./lib.ts";

const script = componentsScript(false);

export default playtest({
  name: "the component previews in their states at desktop size",
  size: "1280x720",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const containers = ui(run, "containers.json");
    t.ok(containers.some((e) => e.id === "Header" && e.states.includes("expanded") && e.name.startsWith("Rewards")), "the tapped accordion section opened and says so");
    const nav = ui(run, "navigation.json");
    t.ok(nav.some((e) => e.role === "tab" && e.name === "Skills, 3" && e.states.includes("selected")), "the E bumper stepped to the second tab");
    t.ok(nav.some((e) => e.role === "menuitem" && e.name === "Weapons"), "the breadcrumbs' collapsed crumbs are a menu");
    const menu = ui(run, "overlays-menu.json");
    t.ok(menu.some((e) => e.role === "menuitemradio" && e.name === "Rarity"), "the Sort by row opened its submenu");
    t.ok(menu.some((e) => e.role === "menuitemcheckbox" && e.states.includes("checked")), "a check entry says it is checked");
    const data = ui(run, "data.json");
    t.ok(data.some((e) => e.role === "columnheader" && e.name === "Wins" && e.value === "sorted descending"), "the Wins header sorted the table high to low");
    t.ok(data.filter((e) => e.role === "row").length < 30, "a table of 10,000 rows built only the rows in view");
    const charts = ui(run, "charts.json");
    t.ok(charts.some((e) => e.role === "figure" && e.name === "Matches"), "each chart is a named figure");
    t.ok(charts.some((e) => e.id === "Tooltip"), "the tapped category shows its tooltip");
    const graph = ui(run, "graph.json");
    t.ok(graph.some((e) => e.role === "grid" && e.name === "December 2026"), "Next turned the calendar to December");
    t.ok(graph.some((e) => e.id === "Level" && e.text === "125%"), "Zoom in scaled the graph to 125 %");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-components-");
  },
});
