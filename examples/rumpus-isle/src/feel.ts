// The juice and the view: Effects plays the one-shot emitters the systems
// asked for (moved where they happen and restarted by their start tick,
// og-game §10) and shakes the camera by trauma (og-feel §4); View eases the
// camera between the title's, the play's and the podium's framing, keeps the
// podium's confetti going, and writes each critter's name tag over it.

import { og, defineSystem } from "@origingame/engine";
import type { Font, Vector3 } from "@origingame/engine";
import { CAMERA, PARK_Y } from "./arena.ts";
import { COLOURS, Effect, Fx, Game, Phase, Prefs } from "./components.ts";
import { seats } from "./seats.ts";
import { linear, must, named } from "./util.ts";

export const Effects = defineSystem("Effects", { writes: [Fx, Effect, og.Transform, og.ParticleSystem, og.Camera] }, (q, time, world) => {
  const fx = must(world.resource(Fx));
  if (fx.shake > 0) {
    og.camera.shake(world, fx.shake);
    fx.shake = 0;
  }
  if (!fx.splash && !fx.dust && !fx.sparkle && !fx.crumbs) return;
  for (const a of q.archetypes()) {
    const kind = a.column(Effect).kind;
    const p = a.column(og.Transform).position;
    const ps = a.column(og.ParticleSystem);
    for (let i = 0; i < a.count; i++) {
      const k = kind[i]!;
      const want = k === 0 ? fx.splash : k === 1 ? fx.dust : k === 2 ? fx.sparkle : fx.crumbs;
      if (!want) continue;
      p[i * 3] = k === 0 ? fx.splashX : k === 1 ? fx.dustX : k === 2 ? fx.sparkleX : fx.crumbsX;
      p[i * 3 + 1] = k === 0 ? -0.55 : k === 3 ? -0.1 : 0.15;
      p[i * 3 + 2] = k === 0 ? fx.splashZ : k === 1 ? fx.dustZ : k === 2 ? fx.sparkleZ : fx.crumbsZ;
      ps.startTick[i] = time.tick;
      ps.visible[i] = 1;
    }
  }
  fx.splash = fx.dust = fx.sparkle = fx.crumbs = false;
});

function lerp3(a: Vector3, b: Vector3, t: number): Vector3 {
  return { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t, z: a.z + (b.z - a.z) * t };
}

export const View = defineSystem("View", { reads: [Prefs], writes: [og.Camera, Game, og.ParticleSystem] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const cam = must(world.resource(og.Camera));
  const prefs = must(world.resource(Prefs));
  const want = g.phase === Phase.Title ? CAMERA.title : g.phase === Phase.Lobby ? CAMERA.lobby : g.phase === Phase.Podium ? CAMERA.podium : CAMERA.play;
  // Eased towards the phase's framing (half-way in about a third of a second); the title drifts a little.
  const ease = prefs.reducedMotion ? 1 : 1 - Math.exp(-time.dt * 3.5);
  const drift = g.phase === Phase.Title && !prefs.reducedMotion ? Math.sin(time.elapsed * 0.25) * 1.2 : 0;
  const eye = { x: want.eye.x + drift, y: want.eye.y, z: want.eye.z };
  const now = cam.position;
  const at = lerp3(now, eye, ease);
  cam.position = at;
  cam.quaternion = og.transform.lookAt(at, want.at);
  // Confetti while the podium shows.
  const confetti = named(world, "fx-confetti");
  const ps = must(world.get(confetti, og.ParticleSystem));
  const on = g.phase === Phase.Podium;
  if (ps.visible !== on) must(world.set(confetti, og.ParticleSystem, { visible: on, startTick: time.tick }));
});

/**
 * Where a world point lands on the screen, in CSS pixels from the top left (null behind the camera):
 * the camera's pose and vertical fov, as three.js projects. The engine has no world-to-screen call,
 * and world text faces +z, rides the tone curve and the depth of field (FINDINGS.md), so the name
 * tags are screen text placed here.
 */
export function toScreen(cam: { position: Vector3; quaternion: { x: number; y: number; z: number; w: number }; fov: number }, p: Vector3, width: number, height: number): { x: number; y: number } | null {
  const d = { x: p.x - cam.position.x, y: p.y - cam.position.y, z: p.z - cam.position.z };
  // Rotate by the inverse of the camera's rotation.
  const q = cam.quaternion;
  const ix = -q.x, iy = -q.y, iz = -q.z, w = q.w;
  const tx = 2 * (iy * d.z - iz * d.y), ty = 2 * (iz * d.x - ix * d.z), tz = 2 * (ix * d.y - iy * d.x);
  const cx = d.x + w * tx + (iy * tz - iz * ty);
  const cy = d.y + w * ty + (iz * tx - ix * tz);
  const cz = d.z + w * tz + (ix * ty - iy * tx);
  if (cz > -0.1) return null;
  const f = 1 / Math.tan(((cam.fov || 50) * Math.PI) / 360);
  const ndcX = ((cx / -cz) * f) / (width / height);
  const ndcY = (cy / -cz) * f;
  return { x: ((ndcX + 1) / 2) * width, y: ((1 - ndcY) / 2) * height };
}

/** Name tags: each seated critter's name over it in its colour (screen text, under the UI). */
export const Tags = defineSystem("Tags", { reads: [Game, og.Camera] }, (_q, _time, world) => {
  const g = must(world.resource(Game));
  if (g.phase === Phase.Title || g.phase === Phase.Scores) return;
  const cam = must(world.resource(og.Camera));
  const vp = og.ui.frameViewport();
  const font = g.font as Font;
  seats(g).forEach((s, k) => {
    if (!s.joined || s.y < -0.4 || s.y < PARK_Y + 1) return;
    const at = toScreen(cam, { x: s.x, y: s.y + 1.25, z: s.z }, vp.width, vp.height);
    if (at === null) return;
    og.render.text(world, {
      font,
      text: `P${k + 1} ${COLOURS[s.colour]!.name}`,
      position: { x: at.x, y: at.y, z: 0 },
      size: 18,
      align: "center",
      color: linear(COLOURS[s.colour]!.rgb),
      outline: 0.7,
      outlineColor: { r: 0.03, g: 0.08, b: 0.16 },
      screen: true,
      layer: 10,
    });
  });
});
