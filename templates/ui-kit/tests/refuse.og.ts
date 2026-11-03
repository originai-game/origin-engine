// The store refusing a purchase it cannot afford: the Moonlit Tide Bundle
// (2,800 gems) with 2,400 gems. Its card says "not enough" and still answers
// a press: it shakes (three decaying swings, then rest where it was), plays
// the error sound on the ui bus, spends nothing and asks for no
// confirmation. Ways it could fail: the press is ignored (no shake), the
// card never moves or never comes back, the refusal is silent, the confirm
// opens, or the price is taken anyway.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, tap, ui } from "./lib.ts";

const SAMPLES = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] as const;
const TIDE = "Shop/Body/View/Content/Store/Sections/featured/Offers/tide";
const GEMS = "Shop/Body/View/Content/Store/Head/Wallet/gems/Amount/Amount";

export default playtest({
  name: "the store refuses an offer the purse cannot buy: its card shakes and sounds",
  commands: [
    "0.3:show:store",
    "0.8:dump_ui:before.json",
    tap(0.85, "Offers/tide"),
    "0.97:dump_pulse:pressed.json",
    ...SAMPLES.map((k) => `${(0.97 + k * 0.04).toFixed(2)}:dump_ui:shake-${k}.json`),
    "1.6:dump_ui:after.json",
    "1.6:dump_entity:demo:refused.json",
    "1.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(demo(run, "refused.json").screen, SCREEN.store, "the store stays open");
    const card = (f: string) => ui(run, f).find((e) => e.path === TIDE);
    const rest = card("before.json");
    t.ok(rest !== undefined && rest.name.includes("2,800 Gems, not enough"), `the bundle says it cannot be afforded (${rest?.name})`);
    const xs = SAMPLES.map((k) => card(`shake-${k}.json`)?.x ?? NaN);
    const moved = Math.max(...xs.map((x) => Math.abs(x - rest!.x)));
    t.ok(moved > 1, `the card shook sideways after the press (up to ${moved.toFixed(2)} px: ${xs.map((x) => x.toFixed(1)).join(", ")})`);
    t.ok(Math.abs(card("after.json")!.x - rest!.x) < 0.01, "and came to rest where it was");
    const after = ui(run, "after.json");
    t.equal(after.find((e) => e.path === GEMS)?.text, "2,400", "no gems were spent");
    t.equal(after.filter((e) => e.path.startsWith("Buy")).map((e) => e.path), [], "no purchase confirmation opened");
    const pulse = run.json<{ voices: Array<[number, number, number]> }>("pressed.json");
    t.ok(pulse.voices.some((v) => v[1] === 3), `the error sound plays on the ui bus (voices ${JSON.stringify(pulse.voices)})`);
  },
});
