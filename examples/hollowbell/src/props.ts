// The hollow's things, a small system each (a system's query is the
// components it names, so one kind of row per system):
// - Pickups: glimmers, chimes and hearts, each an og.Sprite drawing a layer
//   of the items' texture array (sprites/items.ogt), the layer stepped here
//   to spin and shimmer; taken when Wren's chest reaches them.
// - Lanterns: the checkpoints. Touching one lights it (its sprite's layer,
//   its 2D light), restores the hearts and saves the run (src/save.ts).
// - Lifts: the plank over the pit, a kinematic body moved by its velocity
//   (the controller rides it).
// - Stones: dynamic bodies the mallet knocks about.
// - Flickers: the torches' and crystals' 2D lights.
// - Bells and Moles: who is near, the bell's ring, the mole's talk.
// - Hazards: the pit's brambles.
// - Areas: which area holds Wren (a toast of its name) and the 2D ambient
//   light, eased towards the area's.
// - Backdrop: the sky sprite stays behind the camera.
// - Effects: one-shot emitters moved where Game.fx asks and restarted.

import { og, defineSystem } from "@origingame/engine";
import { Bell, Draw, Fx, FxKind, Flicker, Game, HEARTS, Item, Lantern, Layer, Lift, Mole, Phase, Pickup, Pool, Sound, Stone, Tuning } from "./components.ts";
import { fx, heal, hurtHero, pop, say } from "./feel.ts";
import { Area, BELL_CLIP, Hazard, MOLE_CLIP, Sky, must } from "./world.ts";

void Draw;

export const Pickups = defineSystem(
  "Pickups",
  { writes: [Game, Pickup, og.Transform, og.Sprite], after: ["Enemies"] },
  (q, _time, world) => {
    const g = must(world.resource(Game));
    const playing = g.phase === Phase.Play && !g.held;
    for (const a of q.archetypes()) {
      const pk = a.column(Pickup);
      const tr = a.column(og.Transform);
      const sp = a.column(og.Sprite);
      for (let i = 0; i < a.count; i++) {
        const kind = pk.kind[i]!;
        const index = pk.index[i]!;
        if (g.start) {
          // A new run or a continue: taken as the masks say.
          const taken =
            kind === Item.Glimmer ? ((index < 32 ? g.pickedLo >>> index : g.pickedHi >>> (index - 32)) & 1) === 1
            : kind === Item.Chime ? ((g.chimeMask >>> index) & 1) === 1
            : ((g.heartMask >>> index) & 1) === 1;
          pk.taken[i] = taken ? 1 : 0;
          const bossChime = kind === Item.Chime && index === 2;
          pk.hidden[i] = bossChime && !g.bossDown ? 1 : 0;
          sp.visible[i] = taken || pk.hidden[i] ? 0 : 1;
          sp.opacity[i] = 1;
          tr.scale[i * 3] = tr.scale[i * 3 + 1] = 1;
          continue;
        }
        // The gloam's chime appears where it fell.
        if (pk.hidden[i] && g.bossDown && !pk.taken[i]) {
          pk.hidden[i] = 0;
          tr.position[i * 3] = g.bossX;
          pk.baseY[i] = g.bossY + 1;
          sp.visible[i] = 1;
          fx(g, FxKind.Chime, g.bossX, g.bossY + 1);
        }
        if (pk.taken[i]) {
          if (sp.opacity[i]! <= 0.02) sp.visible[i] = 0;
          continue;
        }
        if (pk.hidden[i]) continue;
        // Spin and shimmer through the array's layers; bob.
        const s = g.time + pk.phase[i]!;
        sp.arrayLayer[i] = kind === Item.Glimmer ? Layer.Glimmer + (Math.floor(s * 8) % 4) : kind === Item.Chime ? Layer.Chime + (Math.floor(s * 6) % 4) : Layer.Heart;
        const x = tr.position[i * 3]!;
        const y = pk.baseY[i]! + Math.sin(s * 3) * 0.08;
        tr.position[i * 3 + 1] = y;
        if (!playing || g.respawn > 0) continue;
        if (Math.abs(g.heroX - x) > 0.75 || Math.abs(g.heroY + 0.7 - y) > 0.9) continue;
        if (kind === Item.Heart && g.hearts >= HEARTS) continue;
        // Taken: what was gained above it, a pop, a sound.
        pk.taken[i] = 1;
        const e = a.entity(i);
        og.tween.to(world, { entity: e, field: "scale", to: [1.6, 1.6, 1], duration: 0.15, ease: "backOut" });
        og.tween.to(world, { entity: e, component: og.Sprite, field: "opacity", to: [0], duration: 0.2, delay: 0.05, ease: "cubicIn" });
        if (kind === Item.Glimmer) {
          g.glimmers += 1;
          if (index < 32) g.pickedLo |= 1 << index;
          else g.pickedHi |= 1 << (index - 32);
          g.sfx |= Sound.Glimmer;
          pop(world, "+1", x, y + 0.6, "gold");
        } else if (kind === Item.Chime) {
          g.chimeMask |= 1 << index;
          g.chimes += 1;
          g.sfx |= Sound.Chime;
          fx(g, FxKind.Chime, x, y);
          og.camera.punch(world, 0.06);
          og.ui.toast(world, say(world, "toast.chime", { n: g.chimes }), { tone: "accent" });
        } else {
          g.heartMask |= 1 << index;
          g.hearts = Math.min(HEARTS, g.hearts + 1);
          g.sfx |= Sound.Glimmer;
          pop(world, "+1", x, y + 0.6, "taken");
          og.ui.toast(world, say(world, "toast.heart"), { tone: "success" });
        }
      }
    }
  },
);

export const Lanterns = defineSystem(
  "Lanterns",
  { reads: [og.Transform], writes: [Game, Lantern, og.Sprite, og.Light2D], after: ["Pickups"] },
  (q, _time, world) => {
    const g = must(world.resource(Game));
    for (const a of q.archetypes()) {
      const ln = a.column(Lantern);
      const tr = a.column(og.Transform);
      const sp = a.column(og.Sprite);
      const light = a.column(og.Light2D);
      for (let i = 0; i < a.count; i++) {
        const index = ln.index[i]!;
        const lit = ((g.litMask >>> index) & 1) === 1;
        const x = tr.position[i * 3]!;
        const y = tr.position[i * 3 + 1]!;
        if (g.phase === Phase.Play && !g.held && g.respawn === 0 && g.checkpoint !== index && Math.abs(g.heroX - x) < 0.8 && Math.abs(g.heroY - y) < 1.2) {
          g.litMask |= 1 << index;
          g.checkpoint = index;
          g.checkX = x;
          g.checkY = y;
          heal(g);
          g.saveNow = true; // src/save.ts writes the run this tick
          g.sfx |= Sound.Checkpoint;
          fx(g, FxKind.Chime, x, y + 0.6);
          og.ui.toast(world, say(world, "toast.saved"), { tone: "success" });
        }
        const on = ((g.litMask >>> index) & 1) === 1 || lit;
        sp.arrayLayer[i] = on ? Layer.LanternLit + (Math.floor(g.time * 5 + index) % 2) : Layer.LanternDark;
        sp.glow[i] = on ? 0.6 : 0;
        light.intensity[i] = on ? 1.15 + Math.sin(g.time * 7 + index) * 0.08 : 0;
      }
    }
  },
);

/**
 * The lift rides to and fro over the pit. og-game §13 says a kinematic body
 * moves by its og.Velocity; it does not (BODY_KINEMATIC: "moved by game code
 * through its Transform"), so the lift is moved here by its velocity, which it
 * keeps so the controller standing on it is carried (FINDINGS.md #F-kinematic).
 */
export const Lifts = defineSystem("Lifts", { reads: [Game], writes: [Lift, og.Transform, og.Velocity] }, (q, time) => {
  const dt = time.scaledDt;
  for (const a of q.archetypes()) {
    const lift = a.column(Lift);
    const tr = a.column(og.Transform);
    const v = a.column(og.Velocity);
    for (let i = 0; i < a.count; i++) {
      const x = tr.position[i * 3]!;
      if (x >= lift.x0[i]! + lift.travel[i]!) v.linear[i * 3] = -lift.speed[i]!;
      else if (x <= lift.x0[i]!) v.linear[i * 3] = lift.speed[i]!;
      else if (v.linear[i * 3] === 0) v.linear[i * 3] = lift.speed[i]!;
      v.linear[i * 3 + 1] = 0;
      tr.position[i * 3] = x + v.linear[i * 3]! * dt;
    }
  }
});

export const Stones = defineSystem("Stones", { reads: [Game], writes: [Stone, og.Transform, og.Velocity], after: ["Control"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  for (const a of q.archetypes()) {
    const st = a.column(Stone);
    const tr = a.column(og.Transform);
    const v = a.column(og.Velocity);
    for (let i = 0; i < a.count; i++) {
      if (g.start) {
        tr.position[i * 3] = st.x0[i]!;
        tr.position[i * 3 + 1] = st.y0[i]!;
        v.linear[i * 3] = v.linear[i * 3 + 1] = 0;
        continue;
      }
      const id = a.entity(i) as unknown as number;
      if (g.strike && (g.struck0 === id || g.struck1 === id || g.struck2 === id || g.struck3 === id)) {
        v.linear[i * 3] = g.facing * 6;
        v.linear[i * 3 + 1] = 3.5;
        v.angular[i * 3 + 2] = -g.facing * 4;
        st.knocks[i] = st.knocks[i]! + 1;
      }
    }
  }
});

export const Flickers = defineSystem("Flickers", { reads: [Game, Flicker], writes: [og.Light2D] }, (q, time, world) => {
  const g = must(world.resource(Game));
  const s = time.elapsed;
  void g;
  for (const a of q.archetypes()) {
    const f = a.column(Flicker);
    const light = a.column(og.Light2D);
    for (let i = 0; i < a.count; i++) {
      const p = f.phase[i]! + s * f.rate[i]!;
      light.intensity[i] = f.base[i]! * (0.86 + 0.1 * Math.sin(p) + 0.04 * Math.sin(p * 2.7 + 1));
    }
  }
});

export const Bells = defineSystem(
  "Bells",
  { reads: [og.Transform, og.FlipbookEvent], writes: [Game, Bell, og.Flipbook], after: ["Lanterns"] },
  (q, _time, world) => {
    const g = must(world.resource(Game));
    for (const a of q.archetypes()) {
      const bell = a.column(Bell);
      const tr = a.column(og.Transform);
      const fb = a.column(og.Flipbook);
      for (let i = 0; i < a.count; i++) {
        const x = tr.position[i * 3]!;
        const y = tr.position[i * 3 + 1]!;
        g.nearBell = g.phase === Phase.Play && Math.abs(g.heroX - x) < 1.8 && Math.abs(g.heroY - y) < 1.5;
        if (g.start) {
          bell.rung[i] = 0;
          fb.clip[i] = BELL_CLIP.still;
        }
        if (g.ringing === 1 && !bell.rung[i]) {
          bell.rung[i] = 1;
          fb.clip[i] = BELL_CLIP.ring;
          fb.time[i] = 0;
          g.sfx |= Sound.Win;
        }
      }
    }
    for (const e of world.events(og.FlipbookEvent)) {
      if (e.name !== "toll") continue;
      g.sfx |= Sound.Bell;
      og.camera.shake(world, 0.12);
    }
  },
);

export const Moles = defineSystem("Moles", { reads: [og.Transform], writes: [Game, Mole, og.Flipbook] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  for (const a of q.archetypes()) {
    const tr = a.column(og.Transform);
    const fb = a.column(og.Flipbook);
    const mole = a.column(Mole);
    for (let i = 0; i < a.count; i++) {
      const x = tr.position[i * 3]!;
      const y = tr.position[i * 3 + 1]!;
      g.nearMole = g.phase === Phase.Play && Math.abs(g.heroX - x) < 1.6 && Math.abs(g.heroY - y) < 1.5;
      mole.talking[i] = g.talking ? 1 : 0;
      const clip = g.talking ? MOLE_CLIP.talk : MOLE_CLIP.idle;
      if (fb.clip[i] !== clip) {
        fb.clip[i] = clip;
        fb.time[i] = 0;
      }
    }
  }
});

export const Hazards = defineSystem("Hazards", { reads: [Hazard, Tuning], writes: [Game], after: ["Control"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  if (g.phase !== Phase.Play || g.held || g.respawn > 0) return;
  for (const a of q.archetypes()) {
    const hz = a.column(Hazard);
    for (let i = 0; i < a.count; i++) {
      if (g.heroX < hz.minX[i]! || g.heroX > hz.maxX[i]! || g.heroY > hz.y[i]! - 0.4) continue;
      // The brambles: a heart, and back up on the near edge.
      const left = Math.abs(g.heroX - hz.minX[i]!) < Math.abs(g.heroX - hz.maxX[i]!);
      g.invulnerable = 0;
      hurtHero(world, g, must(world.resource(Tuning)), g.heroX);
      if (g.respawn === 0) {
        g.warp = true;
        g.warpX = left ? hz.minX[i]! - 0.8 : hz.maxX[i]! + 0.8;
        g.warpY = hz.y[i]! + 2;
        og.ui.toast(world, say(world, "toast.fell"), { tone: "danger" });
      }
    }
  }
});

export const Areas = defineSystem("Areas", { reads: [Area], writes: [Game, og.Lighting2D], after: ["Control"] }, (q, time, world) => {
  const g = must(world.resource(Game));
  let found = -1;
  let ambient = 1;
  let key = "";
  for (const a of q.archetypes()) {
    const ar = a.column(Area);
    for (let i = 0; i < a.count; i++) {
      const index = ar.index[i]!;
      if (found >= 0 && index > found) continue;
      const cx = g.heroX;
      const cy = g.heroY + 0.6;
      if (cx < ar.minX[i]! || cx > ar.maxX[i]! || cy < ar.minY[i]! || cy > ar.maxY[i]!) continue;
      found = index;
      ambient = ar.ambient[i]!;
      // The key's column is its UTF-8 bytes, zero-padded (og-core §4); the keys are ASCII.
      key = "";
      for (let k = i * 32; k < i * 32 + 32 && ar.key[k] !== 0; k++) key += String.fromCharCode(ar.key[k]!);
    }
  }
  if (found !== g.area) {
    if (g.phase === Phase.Play && found >= 0 && !g.start && time.tick > 1) og.ui.toast(world, say(world, key));
    g.area = found;
    g.arena = key === "area.arena";
  }
  // Ease the ambient towards the area's: a second or so walking into the dark.
  const target = g.phase === Phase.Play ? ambient : 1;
  g.ambient += (target - g.ambient) * (g.start ? 1 : 0.04);
  const lighting = must(world.resource(og.Lighting2D));
  // Warm on the surface, cooler and bluer underground.
  const warm = Math.min(1, Math.max(0, (g.ambient - 0.2) / 0.8));
  lighting.ambient = { r: g.ambient * (0.75 + 0.25 * warm), g: g.ambient * (0.75 + 0.2 * warm), b: g.ambient * (0.95 - 0.05 * warm) };
});

export const Backdrop = defineSystem("Backdrop", { reads: [og.Camera, Sky], writes: [og.Transform] }, (q, _time, world) => {
  const cam = must(world.resource(og.Camera));
  for (const a of q.archetypes()) {
    const tr = a.column(og.Transform);
    for (let i = 0; i < a.count; i++) {
      tr.position[i * 3] = cam.position.x;
      tr.position[i * 3 + 1] = cam.position.y;
    }
  }
});

export const Effects = defineSystem("Effects", { writes: [Game, Fx, og.Transform, og.ParticleSystem] }, (q, time, world) => {
  const g = must(world.resource(Game));
  if (g.fx === 0) return;
  for (const a of q.archetypes()) {
    const f = a.column(Fx);
    const tr = a.column(og.Transform);
    const ps = a.column(og.ParticleSystem);
    for (let i = 0; i < a.count; i++) {
      const kind = f.kind[i]!;
      if ((g.fx & kind) === 0) continue;
      const [x, y] = kind === FxKind.Dust ? [g.dustX, g.dustY] : kind === FxKind.Hit ? [g.hitX, g.hitY] : kind === FxKind.Chime ? [g.chimeX, g.chimeY] : [g.burstX, g.burstY];
      tr.position[i * 3] = x;
      tr.position[i * 3 + 1] = y;
      ps.startTick[i] = time.tick;
      f.bursts[i] = f.bursts[i]! + 1;
    }
  }
  g.fx = 0;
});

/**
 * The map in the dark: og.Lighting2D lights sprites but never a tilemap's
 * layers (FINDINGS.md #F-tilelight), so the map is tinted to the ambient
 * light here, and the Pools below add the lights' glow back over it.
 */
export const MapShade = defineSystem("MapShade", { reads: [Game], writes: [og.Tilemap], after: ["Areas"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  const warm = Math.min(1, Math.max(0, (g.ambient - 0.2) / 0.8));
  for (const a of q.archetypes()) {
    const tm = a.column(og.Tilemap);
    for (let i = 0; i < a.count; i++) {
      tm.color[i * 3] = g.ambient * (0.75 + 0.25 * warm);
      tm.color[i * 3 + 1] = g.ambient * (0.75 + 0.2 * warm);
      tm.color[i * 3 + 2] = g.ambient * (0.95 - 0.05 * warm);
    }
  }
});

export const Pools = defineSystem("Pools", { reads: [Game, Tuning], writes: [Pool, og.Sprite, og.Transform], after: ["Lanterns"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  // Pools show in the dark only: none at full daylight.
  const dark = Math.max(0, Math.min(1, (1 - g.ambient) / 0.6));
  for (const a of q.archetypes()) {
    const pool = a.column(Pool);
    const sp = a.column(og.Sprite);
    const tr = a.column(og.Transform);
    for (let i = 0; i < a.count; i++) {
      const kind = pool.kind[i]!;
      let on = 1;
      if (kind === 1) {
        tr.position[i * 3] = g.heroX;
        tr.position[i * 3 + 1] = g.heroY + 0.7;
        sp.width[i] = sp.height[i] = g.light * 1.7;
      } else if (kind === 2) {
        on = ((g.litMask >>> pool.index[i]!) & 1) === 1 ? 1 : 0;
      }
      sp.opacity[i] = pool.strength[i]! * dark * on;
      sp.visible[i] = sp.opacity[i]! > 0.01 ? 1 : 0;
    }
  }
});
