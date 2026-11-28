// Regenerates the game's CC0 hero paths and CC BY 3.0 game-icons.net
// silhouettes, author declarations and credits. All paths use a 24-unit grid.
//   bun templates/tools/make-game-icons.ts [repo]
import { mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const REPO = process.argv[2] ?? resolve(import.meta.dir, "../..");
const OUT = join(REPO, "templates/styles/shared/icons");
const GAME_ICONS_REV = "82d948812bfe3f269ef8f731dcdb07b08160edc4";
// Decimal ties are even in the source paths, e.g. 4.125 -> 4.12. Use the
// exact binary significand so multiplying by 100 cannot introduce a tie.
function f(v: number): string {
  const data = new DataView(new ArrayBuffer(8));
  data.setFloat64(0, Math.abs(v));
  const bits = data.getBigUint64(0),
    exponent = Number((bits >> 52n) & 2047n) - 1023 - 52;
  let n = ((bits & ((1n << 52n) - 1n)) | (1n << 52n)) * 100n;
  const den = exponent < 0 ? 1n << BigInt(-exponent) : 1n;
  if (exponent > 0) n <<= BigInt(exponent);
  let q = n / den;
  const r = n % den;
  if (r * 2n > den || (r * 2n === den && q % 2n)) q++;
  const s = (Number(q) / 100).toFixed(2).replace(/\.?0+$/, "");
  return (v < 0 && q ? "-" : "") + (s || "0");
}
const poly = (pts: number[][]) => "M" + pts.map(([x, y]) => `${f(x!)} ${f(y!)}`).join(" L") + " Z";
function circle(cx: number, cy: number, r: number, cw = true): string {
  const s = cw ? 1 : 0;
  return `M${f(cx - r)} ${f(cy)} A${f(r)} ${f(r)} 0 1 ${s} ${f(cx + r)} ${f(cy)} A${f(r)} ${f(r)} 0 1 ${s} ${f(cx - r)} ${f(cy)} Z`;
}
function rect(x0: number, y0: number, x1: number, y1: number, r = 0): string {
  if (r <= 0)
    return poly([
      [x0, y0],
      [x1, y0],
      [x1, y1],
      [x0, y1],
    ]);
  return `M${f(x0 + r)} ${f(y0)} H${f(x1 - r)} A${f(r)} ${f(r)} 0 0 1 ${f(x1)} ${f(y0 + r)} V${f(y1 - r)} A${f(r)} ${f(r)} 0 0 1 ${f(x1 - r)} ${f(y1)} H${f(x0 + r)} A${f(r)} ${f(r)} 0 0 1 ${f(x0)} ${f(y1 - r)} V${f(y0 + r)} A${f(r)} ${f(r)} 0 0 1 ${f(x0 + r)} ${f(y0)} Z`;
}
function arc(cx: number, cy: number, r: number, a0: number, a1: number): string {
  const p = (a: number) => [cx + r * Math.cos((a * Math.PI) / 180), cy - r * Math.sin((a * Math.PI) / 180)];
  const [x0, y0] = p(a0),
    [x1, y1] = p(a1);
  return `M${f(x0!)} ${f(y0!)} A${f(r)} ${f(r)} 0 ${Math.abs(a1 - a0) > 180 ? 1 : 0} 0 ${f(x1!)} ${f(y1!)}`;
}
const crescent = (cx: number, cy: number, r: number, inner: number) =>
  `M${f(cx - r)} ${f(cy)} A${f(r)} ${f(r)} 0 0 0 ${f(cx + r)} ${f(cy)} A${f(r)} ${f(inner)} 0 0 1 ${f(cx - r)} ${f(cy)} Z`;
function star_pts(cx: number, cy: number, R: number, r: number, n = 5, rot = 90): number[][] {
  return Array.from({ length: 2 * n }, (_, k) => {
    const a = ((rot + (k * 180) / n) * Math.PI) / 180,
      rad = k % 2 === 0 ? R : r;
    return [cx + rad * Math.cos(a), cy - rad * Math.sin(a)];
  });
}
function fill(tone: string, d: string, rounded = 0): string {
  const paint = tone === "ink" ? "currentColor" : `var(--color-${tone})`;
  return rounded
    ? `<path d="${d}" fill="${paint}" stroke="${paint}" stroke-width="${f(rounded)}" stroke-linejoin="round"/>`
    : `<path d="${d}" fill="${paint}"/>`;
}
function line(tone: string, d: string, w: number): string {
  const paint = tone === "ink" ? "currentColor" : `var(--color-${tone})`;
  return `<path d="${d}" fill="none" stroke="${paint}" stroke-width="${f(w)}" stroke-linecap="round" stroke-linejoin="round"/>`;
}
const SHADE = "icon-shade",
  GLOSS = "icon-gloss",
  MARK = "icon-mark",
  OUTLINE = "icon-outline";
function disc_icon(tone: string, mark: string[] = []): string[] {
  return [
    fill(tone, circle(12, 11.5, 9)),
    fill(SHADE, crescent(12, 11.5, 9, 7.2)),
    ...mark,
    line(GLOSS, arc(12, 11.5, 6.6, 112, 162), 1.6),
    fill(GLOSS, circle(8.9, 6.5, 0.9)),
  ];
}
const sparkle = (cx: number, cy: number, R: number) => poly(star_pts(cx, cy, R, R * 0.32, 4));

const AUTHORS: Record<string, string> = {
  lorc: "Lorc",
  delapouite: "Delapouite",
  "carl-olsen": "Carl Olsen",
  sbed: "Sbed",
  skoll: "Skoll",
  "lord-berandas": "Lord Berandas",
  faithtoken: "Faithtoken",
};
const BORROWED: [string, [string, string][]][] = [
  [
    "Combat and stats",
    [
      ["attack", "lorc/crossed-swords"],
      ["sword", "lorc/broadsword"],
      ["axe", "lorc/battle-axe"],
      ["bow", "lorc/pocket-bow"],
      ["staff", "lorc/wizard-staff"],
      ["magic", "lorc/magic-swirl"],
      ["armor", "lorc/breastplate"],
      ["speed", "lorc/sprint"],
      ["crit", "lorc/on-target"],
      ["accuracy", "delapouite/crosshair"],
      ["strength", "delapouite/biceps"],
      ["intellect", "lorc/brain"],
      ["luck", "lorc/clover"],
      ["levelUp", "delapouite/upgrade"],
      ["bomb", "lorc/unlit-bomb"],
      ["skull", "lorc/skull-crossed-bones"],
      ["flame", "carl-olsen/flame"],
      ["barrier", "lorc/bolt-shield"],
    ],
  ],
  [
    "Navigation and menus",
    [
      ["home", "delapouite/house"],
      ["shop", "delapouite/shop"],
      ["bag", "lorc/knapsack"],
      ["hero", "delapouite/person"],
      ["team", "delapouite/meeple-group"],
      ["friends", "delapouite/three-friends"],
      ["quest", "lorc/scroll-unfurled"],
      ["map", "lorc/treasure-map"],
      ["mail", "lorc/envelope"],
      ["chat", "delapouite/chat-bubble"],
      ["leaderboard", "delapouite/podium"],
      ["calendar", "delapouite/calendar"],
      ["event", "delapouite/party-popper"],
      ["castle", "delapouite/castle"],
      ["banner", "lorc/flying-flag"],
      ["compass", "lorc/compass"],
      ["book", "lorc/book-cover"],
      ["search", "lorc/magnifying-glass"],
      ["cards", "faithtoken/card-pick"],
      ["dice", "delapouite/rolling-dices"],
      ["rocket", "lorc/rocket"],
      ["inventory", "delapouite/backpack"],
      ["skillTree", "lorc/branch-arrow"],
      ["rank", "skoll/rank-3"],
    ],
  ],
  [
    "Settings and system",
    [
      ["sound", "delapouite/speaker"],
      ["soundOff", "delapouite/speaker-off"],
      ["music", "delapouite/musical-notes"],
      ["bell", "lorc/ringing-bell"],
      ["vibrate", "delapouite/vibrating-smartphone"],
      ["language", "lorc/globe"],
      ["gamepad", "delapouite/gamepad"],
      ["save", "delapouite/save"],
      ["exit", "delapouite/exit-door"],
      ["power", "lord-berandas/power-button"],
      ["info", "delapouite/info"],
      ["account", "delapouite/id-card"],
      ["phone", "delapouite/smartphone"],
      ["unlock", "delapouite/padlock-open"],
      ["settings", "lorc/cog"],
      ["refresh", "delapouite/clockwise-rotation"],
    ],
  ],
  [
    "Social",
    [
      ["like", "delapouite/thumb-up"],
      ["dislike", "delapouite/thumb-down"],
      ["handshake", "delapouite/shaking-hands"],
      ["share", "delapouite/share"],
      ["megaphone", "delapouite/megaphone"],
      ["trade", "lorc/trade"],
    ],
  ],
  [
    "Shop",
    [
      ["cart", "delapouite/shopping-cart"],
      ["sale", "delapouite/price-tag"],
      ["wallet", "delapouite/wallet"],
      ["crate", "delapouite/wooden-crate"],
      ["piggyBank", "delapouite/piggy-bank"],
      ["coins", "delapouite/coins-pile"],
      ["cash", "delapouite/money-stack"],
      ["bank", "delapouite/bank"],
      ["video", "delapouite/video-camera"],
      ["vip", "lorc/crowned-heart"],
      ["pass", "delapouite/vertical-banner"],
      ["ad", "delapouite/tv"],
    ],
  ],
  [
    "Rewards",
    [
      ["chestOpen", "skoll/open-treasure-chest"],
      ["medal", "lorc/medal"],
      ["laurels", "lorc/laurels"],
      ["ribbon", "delapouite/ribbon-medal"],
      ["lootBag", "lorc/swap-bag"],
      ["missions", "lorc/checkbox-tree"],
      ["target", "lorc/archery-target"],
      ["sparkles", "delapouite/sparkles"],
      ["claimAll", "delapouite/basket"],
      ["stamina", "delapouite/lungs"],
    ],
  ],
  [
    "Status effects",
    [
      ["burn", "lorc/flame-claws"],
      ["freeze", "lorc/snowflake-2"],
      ["poison", "lorc/poison-bottle"],
      ["bleed", "lorc/bleeding-wound"],
      ["stun", "delapouite/knocked-out-stars"],
      ["sleep", "delapouite/night-sleep"],
      ["silence", "delapouite/silenced"],
      ["slow", "lorc/snail"],
      ["haste", "delapouite/running-shoe"],
      ["regen", "delapouite/healing"],
      ["curse", "lorc/cursed-star"],
      ["blind", "delapouite/blindfold"],
      ["shock", "sbed/electric"],
      ["ghost", "lorc/ghost"],
      ["root", "lorc/root-tip"],
      ["weak", "lorc/broken-heart"],
      ["charm", "lorc/charm"],
    ],
  ],
];
const HERO: Record<string, string[]> = {};
HERO["coin"] = [
  fill("icon-gold", circle(12, 11.5, 9)),
  fill(SHADE, crescent(12, 11.5, 9, 7.2)),
  fill(SHADE, circle(12, 11.5, 6.2)),
  fill("icon-gold", circle(12, 11.6, 5.2)),
  fill(
    SHADE,
    poly([
      [12, 8.2],
      [14.4, 11.6],
      [12, 15],
      [9.6, 11.6],
    ]),
  ),
  line(GLOSS, arc(12, 11.5, 7.6, 112, 165), 1.4),
  fill(GLOSS, circle(8.6, 6.3, 0.8)),
];
HERO["gem"] = [
  fill(
    "icon-purple",
    poly([
      [6.6, 3.5],
      [17.4, 3.5],
      [21.2, 8.8],
      [12, 20.6],
      [2.8, 8.8],
    ]),
    1,
  ),
  fill(
    GLOSS,
    poly([
      [6.6, 3.5],
      [11.6, 3.5],
      [8.9, 8.8],
      [2.8, 8.8],
    ]),
  ),
  fill(
    SHADE,
    poly([
      [15.1, 8.8],
      [21.2, 8.8],
      [12, 20.6],
    ]),
  ),
  fill(
    SHADE,
    poly([
      [12.4, 3.5],
      [17.4, 3.5],
      [21.2, 8.8],
      [15.1, 8.8],
    ]),
  ),
  fill(
    GLOSS,
    poly([
      [8.9, 8.8],
      [11.2, 8.8],
      [12, 17.6],
    ]),
  ),
];
const _star = star_pts(12, 12.2, 9.2, 4.2);
HERO["star"] = [
  fill("icon-gold", poly(_star), 2),
  fill(SHADE, poly([[12, 12.2], _star[2], _star[3], _star[4], _star[5], _star[6]])),
  line(GLOSS, `M${f(11.3)} ${f(6.2)} L${f(10.1)} ${f(9.0)}`, 1.4),
  fill(GLOSS, circle(6.9, 9.6, 0.8)),
];
const _heart =
  "M12 20.2 C5 15.6 2.8 12.1 2.8 8.7 C2.8 5.7 5.1 3.6 7.6 3.6 C9.4 3.6 11 4.6 12 6.2 C13 4.6 14.6 3.6 16.4 3.6 C18.9 3.6 21.2 5.7 21.2 8.7 C21.2 12.1 19 15.6 12 20.2 Z";
HERO["heart"] = [
  fill("icon-red", _heart),
  fill(SHADE, "M21.2 8.7 C21.2 12.1 19 15.6 12 20.2 C8.4 17.8 6 15.6 4.5 13.4 C6.6 15.4 9 16.8 12 17.8 C17.2 14.6 20.6 11.8 21.2 8.7 Z"),
  line(GLOSS, "M5.3 9.4 C5.1 7.5 6.2 6.1 7.8 6", 1.6),
];
const _shield = "M12 2.6 L20 5.4 L20 11.2 C20 15.8 16.8 19 12 21 C7.2 19 4 15.8 4 11.2 L4 5.4 Z";
HERO["shield"] = [
  fill("icon-blue", _shield, 0.6),
  fill(SHADE, "M12 2.6 L20 5.4 L20 11.2 C20 15.8 16.8 19 12 21 Z"),
  fill(SHADE, "M12 6 L16.8 7.7 L16.8 11.3 C16.8 14.3 14.9 16.4 12 17.7 C9.1 16.4 7.2 14.3 7.2 11.3 L7.2 7.7 Z"),
  fill("icon-blue", "M12 7.4 L15.5 8.6 L15.5 11.3 C15.5 13.6 14.1 15.2 12 16.2 C9.9 15.2 8.5 13.6 8.5 11.3 L8.5 8.6 Z"),
  line(GLOSS, "M5.9 7 L5.9 11.2 C5.9 13.4 6.9 15.3 8.5 16.6", 1.3),
];
const _bolt = [
  [13.8, 2.4],
  [4.6, 13.4],
  [11.1, 13.4],
  [9.4, 21.6],
  [19.4, 9.4],
  [13, 9.4],
  [16.4, 2.4],
];
HERO["energy"] = [
  fill("icon-gold", poly(_bolt), 1.2),
  fill(
    SHADE,
    poly([
      [12.5, 13.4],
      [9.4, 21.6],
      [19.4, 9.4],
      [17.2, 9.4],
    ]),
  ),
  line(GLOSS, "M13.3 4.3 L7.9 11", 1.3),
];
HERO["ammo"] = [
  fill("icon-gold", rect(8.6, 9, 15.4, 18.2)),
  fill("icon-gold", rect(7.6, 17.6, 16.4, 21, 0.8)),
  fill("icon-orange", "M8.6 9.4 C8.6 5.8 10 3.2 12 2.6 C14 3.2 15.4 5.8 15.4 9.4 Z"),
  fill(SHADE, rect(13.4, 9.4, 15.4, 17.6)),
  fill(SHADE, "M13.4 9.4 L15.4 9.4 C15.4 6.6 14.6 4.4 13.4 3.3 Z"),
  fill(SHADE, rect(7.6, 19.6, 16.4, 21, 0.6)),
  line(GLOSS, "M10.4 10.6 L10.4 16.4", 1.2),
  line(GLOSS, "M10.3 7.6 C10.4 6.4 10.8 5.4 11.4 4.7", 1.1),
];
HERO["lock"] = [
  line("icon-steel", "M7.6 10.8 V7.8 A4.4 4.4 0 0 1 16.4 7.8 V10.8", 2.6),
  fill("icon-gold", rect(4.4, 10, 19.6, 21, 2)),
  fill(SHADE, rect(4.4, 17.6, 19.6, 21, 2)),
  fill(OUTLINE, circle(12, 14.2, 1.7)),
  fill(
    OUTLINE,
    poly([
      [11.2, 14.6],
      [12.8, 14.6],
      [13.3, 18.2],
      [10.7, 18.2],
    ]),
  ),
  line(GLOSS, "M6.6 12.3 L6.6 15.6", 1.2),
];
HERO["key"] = [
  fill("icon-gold", circle(7.4, 12, 4.8) + " " + circle(7.4, 12, 1.9, false)),
  fill("icon-gold", rect(11, 10.6, 21.2, 13.4, 0.5)),
  fill("icon-gold", rect(15.2, 13, 17.2, 16.4, 0.4)),
  fill("icon-gold", rect(18.8, 13, 20.8, 17.6, 0.4)),
  fill(SHADE, rect(11.6, 12.4, 21.2, 13.4, 0.4)),
  line(GLOSS, arc(7.4, 12, 3.4, 105, 170), 1.2),
];
const _lid = "M3.4 10.2 V8.2 C3.4 5.4 5.8 3.6 8.6 3.6 H15.4 C18.2 3.6 20.6 5.4 20.6 8.2 V10.2 Z";
HERO["chest"] = [
  fill("icon-wood", rect(3.4, 9.6, 20.6, 20.8, 1.2)),
  fill("icon-wood", _lid),
  fill(SHADE, rect(3.4, 9.4, 20.6, 10.6)),
  fill(SHADE, rect(3.4, 18, 20.6, 20.8, 1.2)),
  fill("icon-gold", rect(6, 4.6, 8.2, 20.8)),
  fill("icon-gold", rect(15.8, 4.6, 18, 20.8)),
  fill("icon-gold", rect(9.9, 8.4, 14.1, 13.6, 0.7)),
  fill(OUTLINE, circle(12, 10.5, 0.9)),
  fill(OUTLINE, rect(11.55, 10.6, 12.45, 12.4)),
  line(GLOSS, "M9.6 5.4 H14.4", 1.1),
];
HERO["gift"] = [
  fill("icon-red", rect(4.2, 10.2, 19.8, 20.8, 0.8)),
  fill("icon-red", rect(3, 7.2, 21, 10.8, 0.8)),
  fill(SHADE, rect(4.2, 10.8, 19.8, 12)),
  fill(SHADE, rect(16.8, 12, 19.8, 20.8, 0.8)),
  fill("icon-gold", rect(10.6, 7.2, 13.4, 20.8)),
  fill("icon-gold", "M12 7.4 C9.4 3.2 5.6 3.8 6.4 6.2 C6.9 7.6 9.6 7.6 12 7.4 Z"),
  fill("icon-gold", "M12 7.4 C14.6 3.2 18.4 3.8 17.6 6.2 C17.1 7.6 14.4 7.6 12 7.4 Z"),
  line(GLOSS, "M5 8.6 H8.6", 1.1),
];
HERO["crown"] = [
  fill("icon-gold", "M3.6 8.6 L7.6 12.6 L12 5.6 L16.4 12.6 L20.4 8.6 L18.8 17.6 H5.2 Z", 1.2),
  fill("icon-gold", circle(3.6, 7.6, 1.5)),
  fill("icon-gold", circle(12, 4.4, 1.5)),
  fill("icon-gold", circle(20.4, 7.6, 1.5)),
  fill("icon-gold", rect(5, 17, 19, 20.6, 0.8)),
  fill(SHADE, rect(5, 17, 19, 18.2)),
  fill(
    "icon-red",
    poly([
      [12, 11.8],
      [13.8, 14],
      [12, 16.2],
      [10.2, 14],
    ]),
  ),
  line(GLOSS, "M7 18.8 H10.6", 1),
];
HERO["potion"] = [
  fill("icon-glass", rect(9.6, 4.4, 14.4, 9.6)),
  fill("icon-glass", circle(12, 14.4, 6.6)),
  fill("icon-red", `M${f(12 - Math.sqrt(5.4 ** 2 - 0.8 ** 2))} 13.6 A5.4 5.4 0 0 0 ${f(12 + Math.sqrt(5.4 ** 2 - 0.8 ** 2))} 13.6 Z`),
  fill(SHADE, `M${f(12 - Math.sqrt(5.4 ** 2 - 2.4 ** 2))} 16.8 A5.4 5.4 0 0 0 ${f(12 + Math.sqrt(5.4 ** 2 - 2.4 ** 2))} 16.8 Z`),
  fill("icon-wood", rect(9.2, 2.4, 14.8, 5.2, 0.6)),
  line(GLOSS, arc(12, 14.4, 4.7, 115, 170), 1.3),
];
HERO["ticket"] = [
  fill("icon-orange", "M2.6 6.4 H21.4 V10 A2 2 0 0 0 21.4 14 V17.6 H2.6 V14 A2 2 0 0 0 2.6 10 Z", 0.8),
  fill(SHADE, rect(2.6, 15.6, 21.4, 17.6)),
  line(SHADE, rect(6.2, 8.8, 17.8, 15.2), 1),
  line(GLOSS, "M4.4 8.2 H9", 1.1),
];
HERO["xp"] = disc_icon("icon-green", [fill(MARK, sparkle(12, 11.5, 4.6))]);
HERO["alert"] = disc_icon("icon-red", [line(MARK, "M12 6.6 V12.4", 2.6), fill(MARK, circle(12, 16.1, 1.5))]);
const _burst = star_pts(12, 12, 10, 7.6, 12, 90);
HERO["new"] = [
  fill("icon-red", poly(_burst), 0.8),
  fill(SHADE, poly([[12, 12], ..._burst.slice(16, 24), _burst[0]!])),
  fill(MARK, sparkle(12, 12, 5)),
];
HERO["rarity"] = [
  fill(
    "icon-blue",
    poly([
      [12, 2.6],
      [21, 12],
      [12, 21.4],
      [3, 12],
    ]),
    1,
  ),
  fill(
    GLOSS,
    poly([
      [12, 2.6],
      [12, 7.6],
      [7.6, 12],
      [3, 12],
    ]),
  ),
  fill(
    SHADE,
    poly([
      [21, 12],
      [12, 21.4],
      [12, 16.4],
      [16.4, 12],
    ]),
  ),
];
HERO["trophy"] = [
  line("icon-gold", "M6.4 5.8 H4 C3.6 9.4 5 11 7.2 11.2", 1.7),
  line("icon-gold", "M17.6 5.8 H20 C20.4 9.4 19 11 16.8 11.2", 1.7),
  fill("icon-gold", "M6 3.4 H18 V8.4 C18 12.2 15.4 14.6 12 14.6 C8.6 14.6 6 12.2 6 8.4 Z"),
  fill("icon-gold", rect(10.8, 14, 13.2, 17.6)),
  fill("icon-wood", rect(7, 17.4, 17, 20.8, 0.9)),
  fill(SHADE, "M14.6 3.4 H18 V8.4 C18 11.6 16.2 13.8 13.6 14.4 C15.2 12.6 15.6 10.4 14.6 3.4 Z"),
  line(GLOSS, "M8.4 5.4 V8.6", 1.2),
];
HERO["hourglass"] = [
  fill(
    "icon-glass",
    "M6.6 4.6 H17.4 C17.4 9 13.2 10.6 13.2 12 C13.2 13.4 17.4 15 17.4 19.4 H6.6 C6.6 15 10.8 13.4 10.8 12 C10.8 10.6 6.6 9 6.6 4.6 Z",
  ),
  fill("icon-gold", "M8.4 7 H15.6 C15 8.8 12.8 10 12 11 C11.2 10 9 8.8 8.4 7 Z"),
  fill("icon-gold", "M12 14.6 C13.2 15.6 15.8 17 16 19.4 H8 C8.2 17 10.8 15.6 12 14.6 Z"),
  fill("icon-wood", rect(4.8, 2.4, 19.2, 5, 0.8)),
  fill("icon-wood", rect(4.8, 19, 19.2, 21.6, 0.8)),
  line(GLOSS, "M8.4 5.8 C8.6 7.6 9.8 8.8 10.8 9.6", 1),
];

const svg = (body: string[]) => '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">\n  ' + body.join("\n  ") + "\n</svg>\n";

/** SVG arcs pack flags as single digits; they must not be read as coordinates. */
function scaled(d: string, k: number): string {
  const out: string[] = [];
  let i = 0,
    cmd = "",
    arg = 0;
  while (i < d.length) {
    const c = d[i]!;
    if (/[a-z]/i.test(c)) {
      cmd = c;
      arg = 0;
      out.push(c);
      i++;
    } else if (/[ ,\t\n\r]/.test(c)) i++;
    else {
      const flag = cmd.toLowerCase() === "a" && [3, 4].includes(arg % 7);
      const m = flag ? c : /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?/.exec(d.slice(i))?.[0];
      if (!m) throw new Error(`bad path data at ${d.slice(i, i + 20)}`);
      const v = Number(m),
        unscaled = cmd.toLowerCase() === "a" && [2, 3, 4].includes(arg % 7);
      out.push(f(unscaled ? v : v * k));
      i += m.length;
      arg++;
    }
  }
  let s = "";
  for (const t of out) s += /^[a-z]/i.test(t) ? t : (s === "" || /[a-z]$/i.test(s) ? "" : " ") + t;
  return s;
}

mkdirSync(join(OUT, "game"), { recursive: true });
const names = Object.keys(HERO),
  credits: [string, string][] = [];
for (const [name, body] of Object.entries(HERO)) writeFileSync(join(OUT, "game", `${name}.svg`), svg(body));
for (const [, items] of BORROWED)
  for (const [name, path] of items) {
    if (name in HERO) throw new Error(`duplicate hero ${name}`);
    const response = await fetch(`https://raw.githubusercontent.com/game-icons/icons/${GAME_ICONS_REV}/${path}.svg`, {
      signal: AbortSignal.timeout(60000),
    });
    if (!response.ok) throw new Error(`${path}: HTTP ${response.status}`);
    const text = await response.text(),
      body: string[] = [];
    for (const match of text.matchAll(/<path([^>]*)\/>/g)) {
      const d = /\bd="([^"]+)"/.exec(match[1]!)![1]!;
      if (d.trim() === "M0 0h512v512H0z") continue;
      body.push(`<path d="${scaled(d, 24 / 512)}" fill="currentColor"/>`);
    }
    if (!body.length) throw new Error(`${path}: no paths`);
    writeFileSync(join(OUT, "game", `${name}.svg`), svg(body));
    names.push(name);
    credits.push([name, path]);
  }
writeFileSync(
  join(OUT, "game.ts"),
  [
    "// Generated by templates/tools/make-game-icons.ts; edit the tool, not this file.",
    "// Hero icons: CC0. Silhouettes: CC BY 3.0, third_party/game-icons/ICONS.md.",
    "import {file} from '@origingame/engine/author';",
    "import {icons} from './game-names.ts';",
    "export default file.icons({",
    "  source: 'shared/icons/game',",
    "  size: 48, range: 8, join: 'round', sticker: [1.3, 0, 1.2],",
    "  icons,",
    "});",
    "",
  ].join("\n"),
);
writeFileSync(
  join(OUT, "game-names.ts"),
  [
    "// Generated by templates/tools/make-game-icons.ts; shared by the cook and gallery.",
    "export const icons = [",
    ...names.map((n) => `  [${JSON.stringify(n)}, ${JSON.stringify(n)}],`),
    "] as const;",
    "",
  ].join("\n"),
);
const tp = join(REPO, "third_party/game-icons");
mkdirSync(tp, { recursive: true });
writeFileSync(
  join(tp, "ICONS.md"),
  "# game-icons.net icons in the engine's game icon set\n\n" +
    `From https://github.com/game-icons/icons at \`${GAME_ICONS_REV}\`, CC BY 3.0\n` +
    "(`LICENSE`), their 512-unit paths scaled to the 24-unit grid and the\n" +
    "site's background square dropped by `templates/tools/make-game-icons.ts`.\n" +
    "The engine's credits (`packages/engine/src/credits.ts`) name each author.\n\n" +
    "| Engine name | game-icons.net icon | Author |\n|---|---|---|\n" +
    credits.map(([n, p]) => `| \`${n}\` | \`${p}\` | ${AUTHORS[p.split("/")[0]!]} |\n`).join(""),
);
const sticker = new Set([SHADE, GLOSS, MARK, OUTLINE]);
writeFileSync(
  join(OUT, "README.md"),
  [
    "# The engine's game icon set",
    "",
    "`game.ts` is the icon set `<Icon name>` draws from when the game has no icon of",
    "that name (the default pack's `iconSet`, docs/ui.md §3.3); names it lacks fall",
    "back to the engine's Lucide line icons (`back`, `close`, `chevronLeft`, `gear` …).",
    "Every icon sits on a 24-unit grid and is cooked as a distance field at 48 px",
    "an em, crisp from 16 to 64 px. `templates/tools/make-game-icons.ts` writes this",
    "directory; edit the tool, not the files.",
    "",
    "## Hero icons (tones)",
    "",
    "Our own designs (CC0), drawn as stickers: an `icon-outline` rim and an",
    "`icon-shadow` drop under the body, which is painted in theme colours, then",
    "`icon-shade` and `icon-gloss` shapes and `icon-mark` marks over it. A pack",
    "restyles all of them through those colours (CONVENTIONS.md §The default",
    "pack's names); a pack whose own `iconSet` names an icon replaces it.",
    "",
    "| Name | Body colours |",
    "|---|---|",
    ...Object.entries(HERO).map(([n, body]) => {
      const roles = [
        ...new Set([...body.join("").matchAll(/var\(--color-([a-z-]+)\)/g)].map((m) => m[1]!).filter((r) => !sticker.has(r))),
      ].sort();
      return `| \`${n}\` | ${roles.map((r) => `\`${r}\``).join(", ") || "the ink (`fill-*` colours it)"} |`;
    }),
    "",
    "## Ink icons",
    "",
    "Filled game-icons.net silhouettes (CC BY 3.0; authors in",
    "`third_party/game-icons/ICONS.md`, credited on the Credits screen), drawn in the",
    "icon's ink (`text-*`, or `fill-*`).",
    "",
    ...BORROWED.map(([kind, items]) => `- **${kind}**: ` + items.map(([n]) => `\`${n}\``).join(", ")),
    "",
  ].join("\n"),
);
console.log(`${Object.keys(HERO).length} hero icons, ${credits.length} borrowed, ${names.length} in game.ts`);
