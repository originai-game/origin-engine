// The sheet's fixture data (docs/ui.md §3.1: fixture values are data, not
// visual tuning). COMBAT is a mid-fight frame; LOADOUT and FEED are the stress
// page: long names, locked and equipped slots, more slots than fit a phone.

export const COMBAT = {
  health: 64, healthMax: 100, shield: 35, shieldMax: 100,
  spread: 0.3, hit: "headshot", hitAt: 1, aimed: true, shape: "cross",
  weapon: "Halcyon Carbine", mode: "auto", magazine: 6, capacity: 28, reserve: 112, reloading: -1,
  hits: [
    { id: "h1", angle: 0.9, strength: 0.9, kind: "normal" },
    { id: "h2", angle: -2.3, strength: 0.5, kind: "shield" },
  ],
};

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
  title: "Loadout",
  selected: "halcyon",
  categories: [
    { id: "primary", label: "Primary" },
    { id: "secondary", label: "Secondary" },
    { id: "melee", label: "Melee" },
    { id: "utility", label: "Utility" },
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
