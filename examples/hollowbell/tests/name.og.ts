// A new game names its bell-keeper: New game on the title opens the name
// dialog (a text field kept in Prefs.name, a str field of world state, the
// save slot's name); a Latin part typed, a Chinese part through an input
// method's composition and commit; every focusable control is named
// (the field by its label); Begin starts the run with the name, saved with
// the settings through og.platform.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, PLAY, script, tap, value } from "./lib.ts";

export default playtest({
  name: "New game's name dialog: typed and composed text, a named accessibility tree, Begin",
  commands: script(
    "0.9:get:Game:at-title.json",
    "0.9:dump_ui:title.json",
    tap("new", 1.0),
    "1.6:get:Game:naming.json",
    "1.6:dump_ui:dialog.json",
    tap("Name", 1.7),
    "1.9:type:Mei",
    "2.0:compose:xiao ling",
    "2.1:compose:小铃",
    "2.2:commit:小铃",
    "2.4:dump_ui:typed.json",
    "2.4:get:Prefs:typed-prefs.json",
    "2.5:screenshot:name.png",
    tap("Begin", 2.6),
    "3.2:get:Game:begun.json",
    "3.2:get:Prefs:name.json",
    "3.2:dump_platform:calls.json",
    "3.3:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = elements(run, "title.json");
    const cont = title.find((e) => e.id === "continue");
    t.ok(cont !== undefined && cont.states.includes("disabled"), "with no save, Continue is shown but inert");
    t.ok(game(run, "at-title.json").a11yUnnamed === 0, "the title's controls are named");
    t.ok(game(run, "naming.json").naming, "New game opened the name dialog");
    const field = elements(run, "dialog.json").find((e) => e.id === "Name");
    t.equal(field?.name, "Name", "the field is named by its label");
    t.equal(game(run, "naming.json").a11yUnnamed, 0, "every focusable control of the dialog has a name");
    t.equal(elements(run, "typed.json").find((e) => e.id === "Name")?.text, "Mei小铃", "the name was typed, its Chinese part through an input method");
    t.equal(value<{ name: string }>(run, "typed-prefs.json").name, "Mei小铃", "and kept in Prefs.name, a str field");
    t.equal(game(run, "begun.json").phase, PLAY, "Begin started the run");
    t.equal(value<{ name: string }>(run, "name.json").name, "Mei小铃", "with the name");
    const calls = run.json<{ calls: Array<{ method: string; args: Record<string, unknown> }> }>("calls.json").calls;
    t.ok(calls.some((c) => c.method === "store.set" && c.args["key"] === "og.settings" && JSON.stringify(c.args).includes("Mei小铃")), "saved with the settings through og.platform");
  },
});
