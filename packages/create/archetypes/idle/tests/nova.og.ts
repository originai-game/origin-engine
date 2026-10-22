// The supernova and its card (the run's end): with 300,000 sparks made
// this run, N asks; Not yet keeps everything; the HUD's Supernova and
// Collapse burst the orb: sparks and generators gone, one stardust, the
// card with the run's sparks, time and the stardust; Ignite again starts
// the next run, every rate 10 % stronger.
import { playtest } from "@origingame/engine/playtest";
import { game, NOVA, PLAY, press, tap, text } from "./lib.ts";

export default playtest({
  name: "a supernova: asked, declined, collapsed, the run's card, the next run stronger",
  commands: [
    "0.3:play",
    "0.9:own:0:5",
    "1.0:sparks:300000",
    press("KeyN", 1.1),
    "1.4:get:Game:asked.json",
    tap("Later", 1.5),
    "1.8:get:Game:declined.json",
    tap("Nova/Supernova", 1.9),
    "2.3:dump_ui:confirm.json",
    tap("Collapse", 2.4),
    "2.55:get:Game:collapsed.json",
    "3.3:dump_ui:card.json",
    "3.35:screenshot:card.png",
    tap("Again", 3.5),
    "4.2:sparks:15",
    press("Digit1", 4.3),
    "4.5:get:Game:next.json",
    "4.6:dump_platform:platform.json",
    "4.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "asked.json").confirmNova, true, "N asked");
    const declined = game(run, "declined.json");
    t.ok(!declined.confirmNova && declined.owned0 === 5 && declined.dust === 0, "Not yet kept everything");
    t.ok(text(run, "confirm.json", "Body")?.includes("1 stardust") ?? false, "the question says what it gives");
    const c = game(run, "collapsed.json");
    t.equal(c.phase, NOVA, "Collapse ended the run");
    t.equal(c.dust, 1, "one stardust (the square root of 3)");
    t.ok(c.sparks === 0 && c.owned0 === 0 && c.runEarned === 0, "sparks and generators gone");
    t.equal(text(run, "card.json", "Value", "Made"), "300K", "the card shows the run's sparks");
    t.equal(text(run, "card.json", "Value", "Gain"), "+1", "and the stardust gained");
    t.equal(text(run, "card.json", "Value", "Bonus"), "+10%", "and what it is worth");
    const next = game(run, "next.json");
    t.equal(next.phase, PLAY, "Ignite again started the next run");
    t.ok(Math.abs(next.rate - 0.55) < 1e-6, `an ember now makes 0.55 a second (${next.rate})`);
    const p = run.json<{ calls: Array<{ method: string; args: Record<string, unknown> }> }>("platform.json");
    t.ok(p.calls.some((x) => x.method === "telemetry" && x.args["name"] === "orb.nova"), "the supernova was reported");
    t.ok(p.calls.some((x) => x.method === "store.set" && (x.args["value"] as { dust?: number } | undefined)?.dust === 1), "and saved at once");
  },
});
