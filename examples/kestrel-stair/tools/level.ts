// The level: models/stair.glb built from src/layout.ts (blocks, ramps,
// stairs, the crypt, the ledges, walls, columns, braziers' bowls and the
// banners' posts), one primitive per material with textures drawn by
// tools/make-assets.ts; and the files that must agree with it: the
// navmesh (nav/stair.ogt) and the foes' behaviours (ai/*.ogt).

import { BANNERS, BLOCKS, BRAZIERS, COLUMNS, CRYPT, FOES, LEDGES, RAMPS, STAIRS, WALLS, WAYSTONES } from "../src/layout.ts";
import { Glb, MeshBuilder } from "./gltf.ts";
import type { V3 } from "./gltf.ts";
import { linear } from "./media.ts";

export type MaterialName = "paving" | "stone" | "grass" | "plaster" | "wood" | "dark";

/** The level's materials: a texture each (drawn in make-assets.ts), a tint and a roughness. */
const MATERIALS: Record<MaterialName, { tint: string; rough: number; tile: number }> = {
  paving: { tint: "#ffffff", rough: 0.9, tile: 3 },
  stone: { tint: "#ffffff", rough: 0.85, tile: 2.5 },
  grass: { tint: "#ffffff", rough: 0.95, tile: 4 },
  plaster: { tint: "#ffffff", rough: 0.9, tile: 3 },
  wood: { tint: "#ffffff", rough: 0.8, tile: 1.5 },
  dark: { tint: "#3a3530", rough: 1, tile: 2 },
};

export function level(textures: Record<Exclude<MaterialName, "dark">, Uint8Array>): Uint8Array {
  const g = new Glb();
  const mat = {} as Record<MaterialName, number>;
  for (const [name, m] of Object.entries(MATERIALS) as [MaterialName, (typeof MATERIALS)[MaterialName]][]) {
    const c = m.tint.replace("#", "");
    const factor = [0, 2, 4].map((i) => linear(parseInt(c.slice(i, i + 2), 16)));
    const tex = name === "dark" ? null : textures[name];
    mat[name] = g.material({
      name,
      pbrMetallicRoughness: { baseColorFactor: [...factor, 1], metallicFactor: 0, roughnessFactor: m.rough, ...(tex ? { baseColorTexture: { index: g.texture(tex) } } : {}) },
    });
  }
  const mb = new MeshBuilder();
  const tile = (n: MaterialName) => MATERIALS[n].tile;
  // A block's top in its own material, its sides in stone (the retaining walls).
  for (const b of BLOCKS) {
    mb.box(mat.stone, b.min, b.max, { skip: "-y+y", tile: tile("stone") });
    const [x0, , z0] = b.min;
    const [x1, y1, z1] = b.max;
    mb.quad(mat[b.mat], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0], { tile: tile(b.mat) });
  }
  for (const r of RAMPS) {
    const a: V3 = [r.x0, r.y0, r.z0];
    const b: V3 = [r.x1, r.y0, r.z0];
    const c: V3 = [r.x1, r.y1, r.z1];
    const d: V3 = [r.x0, r.y1, r.z1];
    const m = r.name === "scree" ? mat.dark : mat.paving;
    mb.quad(m, a, b, c, d, { tile: tile("paving") });
    // The sides: triangles down to the court.
    mb.tri(mat.stone, [r.x1, r.y0, r.z0], [r.x1, r.y0, r.z1], [r.x1, r.y1, r.z1]);
    mb.tri(mat.stone, [r.x0, r.y0, r.z1], [r.x0, r.y0, r.z0], [r.x0, r.y1, r.z1]);
  }
  for (const s of STAIRS) {
    const run = (s.z0 - s.z1) / s.steps;
    for (let i = 0; i < s.steps; i++) {
      const top = s.y0 + s.rise * (i + 1);
      mb.box(mat.stone, [s.x0, s.y0 - 0.01, s.z0 - run * (i + 1)], [s.x1, top, s.z0 - run * i], { skip: "-y", tile: tile("stone") });
    }
  }
  // The crypt: four walls (a door in the east one), a roof, the pillars and the altar.
  const [cx0, cy0, cz0] = CRYPT.min;
  const [cx1, cy1, cz1] = CRYPT.max;
  const w = CRYPT.wall;
  mb.box(mat.plaster, [cx0, cy0, cz0], [cx0 + w, cy1, cz1], { skip: "-y", tile: tile("plaster") });
  mb.box(mat.plaster, [cx0, cy0, cz0], [cx1, cy1, cz0 + w], { skip: "-y", tile: tile("plaster") });
  mb.box(mat.plaster, [cx0, cy0, cz1 - w], [cx1, cy1, cz1], { skip: "-y", tile: tile("plaster") });
  mb.box(mat.plaster, [cx1 - w, cy0, cz0], [cx1, cy1, CRYPT.door.z0], { skip: "-y", tile: tile("plaster") });
  mb.box(mat.plaster, [cx1 - w, cy0, CRYPT.door.z1], [cx1, cy1, cz1], { skip: "-y", tile: tile("plaster") });
  mb.box(mat.plaster, [cx1 - w, CRYPT.door.top, CRYPT.door.z0], [cx1, cy1, CRYPT.door.z1], { tile: tile("plaster") });
  mb.box(mat.stone, [cx0 - 0.3, cy1, cz0 - 0.3], [cx1 + 0.3, cy1 + 0.5, cz1 + 0.3], { tile: tile("stone") });
  for (const [x, z] of CRYPT.pillars) mb.box(mat.stone, [x - 0.4, cy0, z - 0.4], [x + 0.4, cy1, z + 0.4], { skip: "-y", tile: tile("stone") });
  mb.box(mat.stone, [CRYPT.altar.x - 0.6, cy0, CRYPT.altar.z - 1], [CRYPT.altar.x + 0.6, CRYPT.altar.top, CRYPT.altar.z + 1], { skip: "-y", tile: tile("stone") });
  for (const l of LEDGES) {
    const h = l.size / 2;
    mb.box(mat.stone, [l.x - h, 4 - 0.01, l.z - h], [l.x + h, l.top, l.z + h], { skip: "-y", tile: tile("stone") });
  }
  for (const b of WALLS) mb.box(mat.stone, b.min, b.max, { skip: "-y", tile: tile("stone") });
  for (const c of COLUMNS) {
    mb.box(mat.stone, [c.x - 0.45, c.y - 0.01, c.z - 0.45], [c.x + 0.45, c.y + 0.3, c.z + 0.45], { skip: "-y", tile: tile("stone") });
    mb.box(mat.stone, [c.x - 0.32, c.y + 0.3, c.z - 0.32], [c.x + 0.32, c.y + c.h, c.z + 0.32], { skip: "-y", tile: tile("stone") });
  }
  for (const b of BRAZIERS) {
    mb.box(mat.stone, [b.x - 0.18, b.y - 0.01, b.z - 0.18], [b.x + 0.18, b.y + 0.9, b.z + 0.18], { skip: "-y", tile: tile("stone") });
    mb.box(mat.dark, [b.x - 0.42, b.y + 0.9, b.z - 0.42], [b.x + 0.42, b.y + 1.15, b.z + 0.42], { tile: tile("stone") });
  }
  for (const b of BANNERS) mb.box(mat.wood, [b.x - 0.1, b.y - 0.01, b.z - 0.1], [b.x + 0.1, b.y + 4.4, b.z + 0.1], { skip: "-y", tile: tile("wood") });
  for (const b of BANNERS) mb.box(mat.wood, [b.x - 0.1, b.y + 4.2, b.z - 0.1], [b.x + 1.3, b.y + 4.35, b.z + 0.1], { tile: tile("wood") });
  for (const s of WAYSTONES) {
    mb.box(mat.stone, [s.x - 0.35, s.y - 0.01, s.z - 0.25], [s.x + 0.35, s.y + 1.5, s.z + 0.25], { skip: "-y", tile: tile("stone") });
  }
  const prims = [...mb.groups.entries()].map(([material, p]) => ({ pos: p.pos, nor: p.nor, uv: p.uv, idx: p.idx, material }));
  const mesh = g.mesh("stair", prims);
  g.node({ name: "stair", mesh }, true);
  return g.bytes();
}

/**
 * The navmesh: the level model's walkable tops for an agent 0.5 m round and
 * 2.6 m tall (the warden; the imps share it), stepping 0.3 m (the stairs'
 * 0.25 m rise) and climbing slopes to 36 degrees (the ramp and the steep
 * path, not the scree). The summit gate is not baked: the game sets it as
 * an obstacle (og.nav.setObstacle) and lifts it when the gate opens. The
 * `type navmesh` lines are the third-person archetype's (`agent`, `cellSize`,
 * `bounds`) and a guessed `model` entry the cook took: the knowledge package
 * names the parts but not the lines (FINDINGS #2).
 */
export function navmesh(): string {
  return `type navmesh
version 1
---
# The foes' navmesh, baked by og cook from models/stair.glb (tools/level.ts writes this file).
agent {
  radius 0.5
  height 2.6
  climb 0.3
  slope 36
}
cellSize 0.25
cellHeight 0.1
tileSize 32
model "models/stair.glb" {
  position 0 0 0
}
bounds {
  min -23 -1 -67
  max 23 14 45
}
`;
}

/** A foe's behaviour, its round from src/layout.ts. */
export function behaviour(name: string): string {
  const foe = FOES.find((f) => f.name === name)!;
  const route = foe.route.map(([x, y, z]) => `  point ${x} ${y} ${z}`).join("\n");
  if (foe.kind === "imp") {
    return `type behaviour
version 1
---
# A thorn imp (tools/level.ts writes this file from src/layout.ts). It runs
# its round; seeing the courier it chases at Tuning.impChase and, within
# reach, winds up (0.25 s) and bites (the game's \`bite\` command); hurt
# below half its thorns it flees and keeps away until it heals.
name "${name}"
route round {
${route}
}
tree {
  selector {
    sequence flee {
      check Foe.hp below Tuning.impFleeBelow   # read each think (FINDINGS #3, fixed)
      flee {
        distance 7
        speed Tuning.impChase
      }
    }
    sequence bite {
      check sees
      check Game.phase is 1
      check targetDistance below Tuning.impReach
      check Foe.cooldown below 0.01
      wait 0.25
      command bite {
        cooldown 1.1
      }
    }
    sequence chase {
      check sees
      check Game.phase is 1
      chase {
        speed Tuning.impChase
      }
    }
    patrol round {
      wait 0.8
      speed Tuning.impPatrol
    }
  }
}
`;
  }
  return `type behaviour
version 1
---
# A stone warden (tools/level.ts writes this file from src/layout.ts). It
# walks its round slowly; seeing the courier it alerts (the game's
# \`alert\`) and closes in; within reach it raises both fists (the slam's
# long wind-up is the game's, from the \`slam\` command) and stands
# recovering; lost, it searches where it last saw the courier, then goes
# back to its round.
name "${name}"
route round {
${route}
}
machine {
  initial patrol
  state patrol {
    patrol round {
      wait 1.6
      speed Tuning.wardenPatrol
    }
    to alert {
      check sees
      check Game.phase is 1
    }
  }
  state alert {
    enter {
      command alert
    }
    chase {
      speed Tuning.wardenChase
      reach 1.6
    }
    to slam {
      check targetDistance below Tuning.wardenReach   # read each think (FINDINGS #3, fixed)
    }
    to search {
      check sees is false
    }
    to patrol {
      check Game.phase not 1
    }
  }
  state slam {
    enter {
      command slam
    }
    stop
    to alert {
      after 1.8
    }
  }
  state search {
    move_to lastSeen {
      speed Tuning.wardenPatrol
    }
    to alert {
      check sees
      check Game.phase is 1
    }
    to patrol {
      after 4
    }
  }
}
`;
}
