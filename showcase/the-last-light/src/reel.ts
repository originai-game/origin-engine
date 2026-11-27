// The page's recording reel: effects on the path ahead of the wanderer
// (`fx`) and UI over the walk (`mode`, `style`). Nothing here runs in play:
// the reel draws nothing and changes nothing until one of its commands
// selects it, as the page's recordings do.
import { defineResource, defineSystem, f64, math, og } from "@origingame/engine";
import type { Texture, UiMaterial, World } from "@origingame/engine";
import { mount, useStore } from "@origingame/engine/ui";
import { Reel, type ReelCard } from "../assets/ui/reel.tsx";
import { Player } from "./player.ts";
import { must } from "./must.ts";

const STYLES = ["fantasy", "scifi", "cartoon", "anime", "ink", "tactical"] as const;
const MATS = ["foil", "holo", "rarity", "burn"] as const;
const ARTS = ["card-ember-strike", "card-iron-guard", "portrait-mage", "portrait-ranger", "portrait-cultist", "banner-coast"] as const;

export const ReelAssets = defineResource("ReelAssets", {
  ...Object.fromEntries(STYLES.map((s) => [`s_${s}`, f64()])),
  ...Object.fromEntries(MATS.map((s) => [`m_${s}`, f64()])),
  ...Object.fromEntries(ARTS.map((_, i) => [`a${i}`, f64()])),
  /** 1 while the card hand is shown, so the burn runs only then. */
  cards: f64(),
} as Record<string, ReturnType<typeof f64>>);

const assets = (world: World) => must(world.resource(ReelAssets)) as Record<string, number>;

export function loadReel(world: World): void {
  const r = assets(world);
  for (const s of STYLES) r[`s_${s}`] = must(og.load.style(world, `styles/${s}`)) as unknown as number;
  for (const m of MATS) r[`m_${m}`] = must(og.load.uiMaterial(world, `base/materials/materials/${m}`)) as unknown as number;
  ARTS.forEach((a, i) => { r[`a${i}`] = must(og.load.texture(world, `base/textures/art/${a}`)) as unknown as number; });
}

export function mountReel(world: World): void {
  const r = assets(world);
  const mat = (name: string) => r[`m_${name}`] as unknown as UiMaterial;
  const art = (i: number) => r[`a${i}`] as unknown as Texture;
  const cards: ReelCard[] = [
    { id: "foil", name: "Ember Strike", cost: "1", type: "attack", typeLabel: "Attack · Rare", rarity: "rare", text: "Deal 9 damage. Apply 2 Burn.", art: art(0), material: { shader: mat("foil"), params: { amount: 0.9, sweep: [0.35, 0] } }, lift: 0 },
    { id: "holo", name: "Iron Guard", cost: "1", type: "skill", typeLabel: "Skill · Epic", rarity: "epic", text: "Gain 12 Block. Retain.", art: art(1), material: { shader: mat("holo"), params: { amount: 0.4, speed: 0.15 } }, lift: 1 },
    { id: "rarity", name: "Archmage's Vow", cost: "3", type: "power", typeLabel: "Power · Legendary", rarity: "legendary", text: "At the start of your turn, draw 2 cards.", art: art(2), material: { shader: mat("rarity"), params: { color: [1, 0.62, 0.12], size: 16, pulse: 0.5, strength: 1.4 } }, lift: 1 },
    { id: "burn", name: "Hunter's Volley", cost: "2", type: "attack", typeLabel: "Attack", rarity: "uncommon", text: "Deal 4 damage three times.", art: art(3), material: { shader: mat("burn"), params: { edge: 7, scale: 36 } }, lift: 1 },
    { id: "mythic", name: "Hex of Ash", cost: "0", type: "curse", typeLabel: "Curse · Mythic", rarity: "mythic", text: "Unplayable. Lose 1 HP each turn.", art: art(4), material: { shader: mat("rarity"), params: { color: [0.62, 0.22, 1], size: 14, pulse: 0.7, strength: 1.3 } }, lift: 0 },
  ];
  const scope = og.ui.scope(world, "reel");
  const mode = scope.writable("mode", "none"), burn = scope.writable("burn", 0);
  mount(world, () => {
    const m = useStore(mode), b = useStore(burn);
    return Reel({ get mode() { return m(); }, cards, get burn() { return b(); }, portrait: art(2), keyArt: 0 });
  }, {}, { layer: "menu" });
}

/** The burning card burns away over 3 s and grows back over 3 s, whole for 1 s: a 7 s loop. */
export const Burn = defineSystem("Burn", { reads: [ReelAssets] }, (_q, time, world) => {
  if (assets(world).cards !== 1) return;
  const t = (time.tick / 60) % 7;
  const p = t < 1 ? 0 : t < 4 ? (t - 1) / 3 : 1 - (t - 4) / 3;
  must(og.ui.scope(world, "reel").writable("burn", 0).set(p * p * (3 - 2 * p)));
});

export const reelCommands = {
  /** fx:<pack>:<effect>[:ahead:side:scale:up]: an effect on the path, `ahead` metres before the wanderer and `side` to its right. */
  fx: (world: World, args: readonly string[]) => {
    const me = world.find("wanderer");
    if (me === null) return;
    const p = must(world.get(me, og.Transform)).position, h = must(world.resource(Player)).heading;
    const ahead = Number(args[2] ?? 4), side = Number(args[3] ?? 0), s = Number(args[4] ?? 1), up = Number(args[5] ?? 0);
    const f = { x: math.sin(h), z: math.cos(h) };
    const at = { x: p.x + f.x * ahead - f.z * side, y: p.y + up, z: p.z + f.z * ahead + f.x * side };
    must(og.vfx.spawn(world, { asset: must(og.load.vfx(world, `base/vfx/${args[0]}/${args[1]}`)), position: at, scale: s }));
  },
  mode: (world: World, args: readonly string[]) => {
    assets(world).cards = args[0] === "cards" ? 1 : 0;
    must(og.ui.scope(world, "reel").writable("mode", "none").set(args[0] ?? "none"));
  },
  style: (world: World, args: readonly string[]) => { must(og.ui.configure(world, { style: assets(world)[`s_${args[0] ?? "fantasy"}`] as never })); },
};
