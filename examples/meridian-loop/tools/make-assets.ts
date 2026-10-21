// Every asset of Meridian Loop, made by code (`bun tools/make-assets.ts`):
// the circuit, the cars and the wheels (tools/circuit.ts, tools/car.ts), the
// ground grid, the mini-map, the smoke and skid textures, the particle
// emitters, the look, the sounds and the music (tools/audio.ts), and the UI
// skin (the engine's neon pack, copied from templates/ui-kit and retinted).
// The same bytes on every run. `--no-music` skips the 11 MB track.
//
// Not committed (.gitignore): the music, and the skin's copied fonts and
// atlases; everything else here is small and committed as made.

import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { carModel, ghostModel, wheelModel } from "./car.ts";
import { groundModel, minimap, sceneryModel, trackModel } from "./circuit.ts";
import * as A from "./audio.ts";
import { draw, hash2, wav } from "./media.ts";
import { MAP } from "../src/track.ts";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const assets = join(root, "assets");
const noMusic = process.argv.includes("--no-music");

function write(rel: string, bytes: Uint8Array | string): void {
  const path = join(assets, rel);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, bytes);
}

// Models.
write("models/track.glb", trackModel());
write("models/scenery.glb", sceneryModel());
write("models/ground.glb", groundModel());
write("models/car.glb", carModel());
write("models/ghost.glb", ghostModel());
write("models/wheel.glb", wheelModel(false));
write("models/ghost-wheel.glb", wheelModel(true));

// Textures.
write("textures/minimap.png", minimap(MAP));
// A soft puff for the tyre smoke and the landing dust.
write(
  "textures/puff.png",
  draw(64, (u, v, x, y) => {
    const d = Math.hypot(u - 0.5, v - 0.5) * 2;
    const n = 0.75 + 0.25 * hash2(x >> 2, y >> 2, 3);
    const a = Math.max(0, 1 - d) ** 1.6 * n;
    return [255, 255, 255, a * 255];
  }),
);
// A spark: a bright core, a soft halo.
write(
  "textures/spark.png",
  draw(32, (u, v) => {
    const d = Math.hypot(u - 0.5, v - 0.5) * 2;
    const a = Math.max(0, 1 - d) ** 2.5;
    return [255, 255, 255, a * 255];
  }),
);
// The skid marks' decal atlas: one rubber streak, darker at its middle, ragged at its ends.
write(
  "textures/skid.png",
  draw(64, (u, v, x, y) => {
    const across = Math.abs(u - 0.5) * 2;
    const along = Math.min(v, 1 - v) * 2;
    const grain = 0.7 + 0.3 * hash2(x, y >> 1, 9);
    const a = Math.max(0, 1 - across ** 3) * Math.min(1, along * 3) * grain;
    return [14, 12, 16, a * 235];
  }),
);
write("textures/puff.png.ogt", "type import\nversion 1\n---\ntexture {\n  kind color\n}\n");

// Particles.
write(
  "fx/smoke.ogt",
  `type particles
version 1
---
# Tyre smoke from a sliding rear wheel: emitted while the slide lasts (src/car.ts writes startTick and stopTick).
maxParticles 160
shape sphere 0.25
spread 40
rate 34
duration 1
looping true
lifetime 0.9 1.5
speed 0.6 1.6
size 0.7 1.1
sizeOverLife 0 0.5 1 2.6
colorOverLife {
  key 0 0.82 0.74 0.95 0.5
  key 0.5 0.55 0.45 0.75 0.28
  key 1 0.4 0.3 0.6 0
}
gravity 0 0.9 0
drag 1.4
noise 0.6 0.4 0.3
texture "textures/puff.png"
blend alpha
space world
softDistance 0.6
`,
);
write(
  "fx/sparks.ogt",
  `type particles
version 1
---
# Sparks where the car scrapes a wall: a one-shot burst (src/car.ts restarts it on each bump).
maxParticles 64
shape point
spread 70
rate 0
burst 0 36
duration 0.2
looping false
lifetime 0.25 0.6
speed 5 12
size 0.05 0.11
sizeOverLife 0 1 1 0.2
colorOverLife {
  key 0 1 0.95 0.7 1
  key 0.4 1 0.55 0.2 1
  key 1 1 0.2 0.5 0
}
color 3 2.2 1.6 1
gravity 0 -9 0
drag 1.2
texture "textures/spark.png"
blend additive
space world
align velocity 0.05
collision 0.35 0.3
`,
);
write(
  "fx/dust.ogt",
  `type particles
version 1
---
# A landing after the jump: a ring of grit thrown out from the wheels.
maxParticles 90
shape cone 85 1.2
spread 10
rate 0
burst 0 70
duration 0.3
looping false
lifetime 0.5 1.1
speed 3 7
size 0.4 0.9
sizeOverLife 0 0.6 1 2
colorOverLife {
  key 0 0.75 0.6 1 0.55
  key 1 0.45 0.3 0.7 0
}
gravity 0 -2 0
drag 2.5
texture "textures/puff.png"
blend alpha
space world
`,
);

// The look: the neon preset, tuned for a night circuit (src/world.ts adds the moon and the lamps).
write(
  "looks/meridian.ogt",
  `type look
version 1
---
name "Meridian night"
extends "neon-vector"
skin "skins/meridian/skin.ogt"
RenderSettings {
  bloom true
  bloomThreshold 0.75
  bloomKnee 0.4
  bloomIntensity 0.9
  toneMapping agx
  exposure 1.15
  environmentIntensity 0.35
}
MaterialStyle {
  # The preset's cyan hull outline, a constant 1.5 px: with an outlineDistance it grows without limit near the camera and floods the frame (FINDINGS #7).
  outlineWidth 1.5
  shadowTint "#2a0d4a"
  shadowTintAmount 0.5
}
Fog {
  enabled true
  color "#2a0f3d"
  density 0.0026
  sunColor "#ff5fb2"
  sunPower 6
  sky 0.25
}
Sky {
  enabled true
  zenith "#05030f"
  horizon "#6a1d63"
  ground "#07040e"
  sunDirection 0.1 0.06 -1
  sunColor "#ff6f3c"
  sunSize 9
  sunIntensity 6
  glow 1.2
  glowPower 6
  stars 0.7
  brightness 0.8
}
PostEffects {
  vignette true
}
`,
);

// Sounds.
const mono = (rel: string, f: () => Float32Array) => write(rel, wav(f(), A.RATE, 1));
mono("audio/engine.wav", A.engine);
mono("audio/skid.wav", A.skid);
mono("audio/bump.wav", A.bump);
mono("audio/land.wav", A.land);
mono("audio/beep.wav", A.beep);
mono("audio/go.wav", A.go);
mono("audio/sector.wav", A.sector);
mono("audio/lap.wav", A.lap);
mono("audio/best.wav", A.best);
mono("audio/slower.wav", A.slower);
mono("audio/ui-hover.wav", A.hover);
mono("audio/ui-click.wav", A.click);
mono("audio/ui-open.wav", A.open);
mono("audio/ui-close.wav", A.close);
mono("audio/ui-error.wav", A.error);
if (!noMusic) write("music/meridian.wav", wav(A.music(), A.RATE, 2));
write("music/meridian.wav.ogt", "type import\nversion 1\n---\n# 57.6 s of stereo: streamed in 4-second chunks (og-audio §5).\nsound {\n  stream true\n  chunk 4\n}\n");

// The skin: the engine's neon pack (templates/ui-kit/assets/skins/neon), its art and fonts copied, its file retinted.
const pack = join(root, "..", "..", "templates", "ui-kit", "assets", "skins", "neon");
const mine = join(assets, "skins", "meridian");
mkdirSync(mine, { recursive: true });
for (const f of readdirSync(pack)) if (f !== "skin.ogt") copyFileSync(join(pack, f), join(mine, f));
// Retinted: a hotter magenta accent and cyan focus; no CJK fallback (the game speaks English and German, both in the pack's Latin-1 faces).
const skin = new TextDecoder()
  .decode(readFileSync(join(pack, "skin.ogt")))
  .replace("# The neon style pack: generated by templates/ui-kit/tools/make-skins.py (CC0 art, OFL fonts; LICENSES.md).", "# Meridian Loop's skin: the engine's neon pack (templates/ui-kit), copied and retinted by tools/make-assets.ts.")
  .replace('name "neon"', 'name "meridian"')
  .replace('accent "#c81ebc"', 'accent "#d4127f"')
  .replace('accentHover "#d42ac8"', 'accentHover "#e0258f"')
  .replace('accentActive "#b018a6"', 'accentActive "#b80e6d"')
  .replace('accentInk "#ff8af8"', 'accentInk "#ff8ac8"')
  .replace(/\n  fallback "[^"]*"/, "");
write("skins/meridian/skin.ogt", skin);
console.log("meridian-loop: assets written");
