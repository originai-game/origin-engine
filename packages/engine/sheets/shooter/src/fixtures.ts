// The sheet's fixture data (docs/ui.md §3.1: fixture values are data, not
// visual tuning). LOADOUT and FEED are the stress page: long names, locked
// and equipped slots, more slots than fit a phone.

// COMBAT is a mid-fight frame, laid out as the approved HUD mockup: the
// squad, objectives, radar, round, killfeed and pickups, the reticle with a
// headshot, the vitals with their effects, the ability bar and the
// ammunition. A field naming sample art (portrait, icon, map) holds its
// file under assets/art, which main.ts loads and replaces with the texture.
export const COMBAT = {
  spread: 0.3, hit: "headshot", hitAt: 1, aimed: true, shape: "cross",
  hits: [
    { id: "h1", angle: -1.9, strength: 0.9, kind: "normal" },
  ],
  heading: 0.733, units: " m",
  pois: [
    { id: "relay", bearing: 0.6, distance: 85, kind: "objective", icon: "", label: "Relay tower" },
    { id: "foe", bearing: 1.47, distance: 240, kind: "enemy", icon: "", label: "Enemy" },
  ],
  squad: [
    { id: "moraine", name: "Moraine", portrait: "moraine", status: "Reviving", state: "reviving", shield: 50, shieldMax: 100, health: 38, healthMax: 100, ghost: 64 },
    { id: "quill", name: "Quill", portrait: "quill", status: "", state: "ok", shield: 100, shieldMax: 100, health: 100, healthMax: 100, ghost: 0 },
  ],
  goals: [
    { id: "relay", text: "Secure the relay tower", tag: "2 / 3", current: true },
    { id: "catwalk", text: "Hold the catwalk", tag: "0:45", current: false },
  ],
  radar: { map: "map", centerX: 0.5, centerY: 0.5, zoom: 0.42, heading: 42 },
  blips: [
    { id: "a1", kind: "ally", x: 0.433, y: 0.542, label: "Moraine" },
    { id: "a2", kind: "ally", x: 0.534, y: 0.584, label: "Quill" },
    { id: "f1", kind: "foe", x: 0.55, y: 0.408, label: "Enemy" },
    { id: "f2", kind: "foe", x: 0.6, y: 0.442, label: "Enemy" },
    { id: "o1", kind: "objective", x: 0.475, y: 0.408, label: "Relay tower" },
  ],
  round: { round: 2, clock: "1:42", ours: 7, theirs: 5, urgent: false },
  kills: [
    { id: "c3", killer: "You", victim: "Vanta", weapon: "VK-7 Carbine", kind: "rifle", headshot: true, self: true, team: "ally" },
    { id: "c2", killer: "Quill", victim: "Okonkwo-Reyes", weapon: "Hollow SMG", kind: "smg", headshot: false, self: false, team: "ally" },
    { id: "c1", killer: "Vanta", victim: "Moraine", weapon: "Talon AR", kind: "rifle", headshot: false, self: false, team: "enemy" },
  ],
  pickups: [
    { id: "frag", name: "Frag grenade", icon: "frag", glyph: "bomb", quantity: 1, rarity: "legendary" },
    { id: "cell", name: "Shield cell", icon: "cell", glyph: "shield", quantity: 2, rarity: "rare" },
    { id: "light", name: "Light rounds", icon: "rounds", glyph: "bolt", quantity: 30, rarity: "common" },
  ],
  dealt: [
    { id: "d2", amount: 86, kind: "headshot" },
    { id: "d1", amount: 31, kind: "normal" },
  ],
  prompt: { key: "E", text: "Open supply bin", disabled: false, reason: "" },
  vitals: { name: "Ysolde", level: 24, portrait: "ysolde", health: 64, healthMax: 100, ghost: 82, shield: 75, shieldMax: 100, segment: 25, low: 0.3 },
  effects: [
    { id: "haste", name: "Haste", icon: "haste", glyph: "bolt", remaining: 0.7, label: "0:12", harmful: false },
    { id: "fury", name: "Fury", icon: "fury", glyph: "sword", remaining: 0.35, label: "×3 4.2", harmful: false },
    { id: "burn", name: "Burning", icon: "burn", glyph: "fire", remaining: 0.55, label: "6", harmful: true },
  ],
  abilities: [
    { id: "dash", name: "Phase dash", icon: "dash", glyph: "bolt", key: "Q", cooldown: 0.38, time: "6", count: 0, ready: false, disabled: false, ultimate: false, charge: 0 },
    { id: "frag", name: "Frag grenade", icon: "frag", glyph: "bomb", key: "G", cooldown: 0, time: "", count: 2, ready: true, disabled: false, ultimate: false, charge: 0 },
    { id: "ult", name: "Overdrive", icon: "overdrive", glyph: "star", key: "Z", cooldown: 0, time: "", count: 0, ready: false, disabled: false, ultimate: true, charge: 0.74 },
    { id: "stim", name: "Stim", icon: "stim", glyph: "potion", key: "4", cooldown: 0, time: "", count: 4, ready: false, disabled: false, ultimate: false, charge: 0 },
    { id: "wall", name: "Barrier", icon: "barrier", glyph: "shield", key: "X", cooldown: 0.92, time: "38", count: 0, ready: false, disabled: true, ultimate: false, charge: 0 },
  ],
  ammo: { weapon: "VK-7 Carbine", mode: "auto", magazine: 24, capacity: 30, reserve: 120, reloading: -1, low: 0.25, secondary: "Hollow SMG", secondaryAmmo: 32, secondaryKey: "2" },
};

/** The sample art under assets/art (sheet only, never installed), by the name a fixture field gives. */
export const ART = ["stage", "map", "ysolde", "moraine", "quill", "frag", "cell", "rounds", "dash", "overdrive", "stim", "barrier", "haste", "fury", "burn"] as const;

// The player is Kestrel: k6 is the player's kill and k1 the player's death,
// so those two rows are self rows, framed as the local player's (VALORANT's
// emphasis); a phone shows the three newest, so only k6 there.
export const FEED = [
  { id: "k6", killer: "Kestrel", victim: "Vanta", weapon: "Halcyon", headshot: true, self: true, team: "ally" },
  { id: "k5", killer: "Halden", victim: "Moraine", weapon: "Brace-12", headshot: false, self: false, team: "enemy" },
  { id: "k4", killer: "Sable", victim: "Quill", weapon: "Talon AR", headshot: false, self: false, team: "ally" },
  { id: "k3", killer: "xX_Midnight_Sniper_Xx", victim: "Tarn", weapon: "Lancet DMR", headshot: true, self: false, team: "enemy" },
  { id: "k2", killer: "Ashby", victim: "TheQuickBrownFoxJumps", weapon: "Vesper SMG", headshot: false, self: false, team: "ally" },
  { id: "k1", killer: "Corvid", victim: "Kestrel", weapon: "Pike .50", headshot: true, self: true, team: "enemy" },
];

export const LOADOUT = {
  // The heading and the category labels are the kit's own strings (empty
  // takes them), so they translate in every locale cell.
  title: "",
  selected: "halcyon",
  categories: [
    { id: "primary", label: "" },
    { id: "secondary", label: "" },
    { id: "melee", label: "" },
    { id: "utility", label: "" },
  ],
  items: [
    { id: "strider", category: "primary", name: "Strider AR", icon: "sword", detail: "25 / 75", equipped: true, locked: false, cost: "2,900" },
    { id: "halcyon", category: "primary", name: "Halcyon", icon: "sword", detail: "30 / 90", equipped: false, locked: false, cost: "2,900" },
    { id: "keel", category: "primary", name: "Keel Marksman Rifle, Extended Magazine", icon: "sword", detail: "12 / 36", equipped: false, locked: false, cost: "2,250" },
    { id: "pike", category: "primary", name: "Pike .50", icon: "sword", detail: "5 / 10", equipped: false, locked: true, cost: "4,700" },
    { id: "wren", category: "secondary", name: "Wren Pistol", icon: "bolt", detail: "15 / 45", equipped: true, locked: false, cost: "500" },
    { id: "revolver", category: "secondary", name: "Wren Revolver", icon: "bolt", detail: "6 / 24", equipped: false, locked: false, cost: "800" },
    { id: "knife", category: "melee", name: "Field Knife", icon: "sword", detail: "", equipped: true, locked: false, cost: "" },
    { id: "frag", category: "utility", name: "Frag Grenade", icon: "bomb", detail: "×2", equipped: true, locked: false, cost: "300" },
    { id: "smoke", category: "utility", name: "Smoke Canister", icon: "drop", detail: "×1", equipped: false, locked: false, cost: "200" },
    { id: "heal", category: "utility", name: "Field Medic Kit", icon: "potion", detail: "×0", equipped: false, locked: true, cost: "400" },
  ],
};
