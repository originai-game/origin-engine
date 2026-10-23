// Coin Duel alone (no room): the local keys drive Red. D walks it right
// through the coin at the centre (Red 1), Space dashes, and the HUD reads
// both scores. The frame at 2 s is what
// `og polish` reads for contrast, text size and the HUD's share.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "alone, the keys walk Red through the coin and dash it, and the HUD shows the scores",
  commands: [
    "0.2:key_down:KeyD",
    "1.2:expect:entity:fighter-0:within:7,0,-1,16,9,1",
    "1.2:key_up:KeyD",
    "1.3:key_down:Space",
    "1.4:key_up:Space",
    "1.5:expect:entity:fighter-1:within:12.5,4,-1,13.5,5,1",
    "2:screenshot:solo.png",
    "2:expect:ui:text:Red 1",
    "2:expect:ui:text:Blue 0",
    "2:expect:ui:text:Local: Red",
    "2.2:quit",
  ].join(";"),
});
