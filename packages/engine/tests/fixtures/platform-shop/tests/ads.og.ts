// A rewarded ad through the fake: the first is skipped (no reward), the
// second watched to the end (10 gems). Each pauses the game while it shows
// and is answered after the resume.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "a skipped rewarded ad pays nothing and a completed one pays 10 gems",
  commands: "0.5:ad;1.5:ad;2.5:get:Wallet:wallet.json;2.6:quit",
  platform: { outcomes: { "ad.show": ["skipped"] } },
  check(run, t) {
    const w = run.json<{ value: { gems: number; skipped: number } }>("wallet.json").value;
    t.equal(w.skipped, 1, "the skipped ad was counted, not paid");
    t.equal(w.gems, 10, "the completed ad paid 10 gems");
  },
});
