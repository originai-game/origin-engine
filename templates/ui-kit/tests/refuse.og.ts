// The shop refusing a purchase it cannot afford: the Crown (2,400) with
// 1,250 coins. Its Buy reads "Not enough" and still answers a press: it
// shakes (og.ui.shake: three decaying swings, then rest where it was),
// plays the error sound on the ui bus, spends nothing and reports the
// refusal (ShopResult.refused). Ways it could fail: the press is ignored
// (a disabled button with no skin shake), the button never moves or never
// comes back, the refusal is silent, or the price is taken anyway.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, tap, ui } from "./lib.ts";

const SAMPLES = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] as const;

export default playtest({
  name: "the shop refuses an item the balance cannot buy: Buy shakes and sounds",
  commands: [
    "0.3:show:shop",
    tap(1.0, "Items/Cell#2"),
    "1.4:dump_ui:before.json",
    tap(1.5, "Detail/Buy"),
    "1.62:dump_pulse:pressed.json",
    ...SAMPLES.map((k) => `${(1.62 + k * 0.04).toFixed(2)}:dump_ui:shake-${k}.json`),
    "2.4:dump_ui:after.json",
    "2.4:dump_entity:demo:refused.json",
    "2.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const d = demo(run, "refused.json");
    t.equal(d.screen, SCREEN.shop, "the shop stays open");
    t.equal([d.balance, d.owned, d.bought, d.refused], [1250, 0, 0, 1], "nothing was bought and one refusal was reported");
    const buy = (f: string) => ui(run, f).find((e) => e.id === "Buy" && e.parent === "Detail");
    const rest = buy("before.json");
    t.ok(rest !== undefined && rest.text === "Not enough", `the Crown's Buy says it cannot be afforded (${rest?.text})`);
    const xs = SAMPLES.map((k) => buy(`shake-${k}.json`)?.x ?? NaN);
    const moved = Math.max(...xs.map((x) => Math.abs(x - rest!.x)));
    t.ok(moved > 1, `the button shook sideways after the press (up to ${moved.toFixed(2)} px: ${xs.map((x) => x.toFixed(1)).join(", ")})`);
    t.ok(Math.abs(buy("after.json")!.x - rest!.x) < 0.01, "and came to rest where it was");
    const pulse = run.json<{ voices: Array<[number, number, number]> }>("pressed.json");
    t.ok(pulse.voices.some((v) => v[1] === 3), `the error sound plays on the ui bus (voices ${JSON.stringify(pulse.voices)})`);
  },
});
