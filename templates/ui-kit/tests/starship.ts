// The starship screens' scripts (src/starship.ts): each screen in the
// scifi pack (their showcase), in fantasy and in kawaii (a light pack),
// then right to left and at 200 % text. The desktop and phone playtests
// share them; each frame is `<style>-<screen>.png`, compared with
// crates/og-ui/references/<tier>/kit-starship[-phone]-<file>.

import { builder, type Script } from "./patterns.ts";

export const STARSHIP_SCREENS = ["shipStatus", "turnOrder", "abilityCards", "sectorMap", "hangar"] as const;

/** Every starship screen in scifi, fantasy and kawaii, then scifi right to left and at 200 % text. */
export function starshipScript(phone: boolean): Script {
  const b = builder(0.3);
  if (phone) b.cmd("viewport:phone");
  b.cmd("pat:ship:reset");
  for (const style of ["scifi", "fantasy", "kawaii"]) {
    b.cmd(`style:${style}`);
    for (const s of STARSHIP_SCREENS) {
      b.cmd(`show:${s}`);
      b.shot(`${style}-${s}`, 0.8);
      b.dump(`${style}-${s}`);
    }
  }
  // Another class picked in the hangar: the list's selection moves and the centre and detail follow.
  b.cmd("style:scifi");
  b.cmd("show:hangar");
  b.cmd("pat:ship:class:2");
  b.shot("scifi-hangar-carriers", 0.6);
  b.dump("scifi-hangar-carriers");
  b.cmd("pat:ship:reset");
  b.cmd("locale:ar");
  for (const s of STARSHIP_SCREENS) {
    b.cmd(`show:${s}`);
    b.shot(`rtl-${s}`, 0.8);
  }
  b.cmd("locale:ja");
  for (const s of STARSHIP_SCREENS) {
    b.cmd(`show:${s}`);
    b.shot(`ja-${s}`, 0.8);
  }
  b.cmd("locale:en");
  b.cmd("demo:text:200");
  for (const s of STARSHIP_SCREENS) {
    b.cmd(`show:${s}`);
    b.shot(`text200-${s}`, 0.8);
  }
  b.cmd("demo:text:100");
  b.cmd("style:default");
  return b.end();
}
