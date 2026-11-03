// The UI centrepiece's frames: og-ui's own reference frames (crates/og-ui/
// references/metal, drawn by the engine on Metal and held by the gate). Two
// axes the visitor switches: the 17 genre screens × locale × device, and the
// 12 style packs × screen × device. `scripts/media.ts` encodes every cell
// into public/m/ui/ and writes src/generated/ui-grid.json.
//
// Status: interim. U8 rebuilds the genre showcases (genre × pack × locale in
// one set); when they land, they replace both axes here.
export const UI_STATUS: "interim" | "final" = "interim";
const REF = "crates/og-ui/references/metal";

type T = { en: string; zh: string };
const t = (en: string, zh: string): T => ({ en, zh });

/** [key, sheet group, the English screen's state, the screen's name in its ja/rtl frames, label] */
const GENRES: [string, string, string, string, T][] = [
  ["lobby", "kit-genres", "lobby-idle", "lobby", t("Party lobby", "组队大厅")],
  ["draft", "kit-genres", "draft-pick", "draft", t("Hero draft", "英雄选择")],
  ["select", "kit-genres", "select-idle", "select", t("Character select", "角色选择")],
  ["versus", "kit-genres", "versus", "versus", t("Fighting: versus", "格斗对战")],
  ["songs", "kit-genres", "songs-idle", "songs", t("Rhythm: song select", "音游选曲")],
  ["worldMap", "kit-genres", "map-idle", "worldMap", t("World map", "世界地图")],
  ["garage", "kit-genres2", "garage-idle", "garage", t("Racing garage", "赛车改装")],
  ["rts", "kit-genres2", "rts-idle", "rts", t("Real-time strategy", "即时战略")],
  ["cityMap", "kit-genres2", "citymap-traffic", "cityMap", t("City builder", "城市建造")],
  ["ledger", "kit-genres2", "ledger-idle", "ledger", t("Management ledger", "经营账本")],
  ["chat", "kit-genres2", "chat-hud", "chat", t("MMO chat", "MMO 聊天")],
  ["deck", "kit-genres3", "deck-idle", "deck", t("Deck builder", "卡牌组牌")],
  ["loadout", "kit-genres3", "loadout-idle", "loadout", t("Shooter loadout", "射击配装")],
  ["routes", "kit-genres3", "routes-idle", "routes", t("Story routes", "剧情路线")],
  ["cgGallery", "kit-genres3", "cg-idle", "cgGallery", t("Visual novel gallery", "视觉小说 CG 鉴赏")],
  ["photo", "kit-genres3", "photo-idle", "photo", t("Photo mode", "拍照模式")],
  ["settingsTree", "kit-genres3", "settings-idle", "settingsTree", t("Settings", "设置")],
];

export const locales = [
  { key: "en", label: t("English", "English") },
  { key: "ja", label: t("日本語", "日本語") },
  { key: "rtl", label: t("العربية", "العربية") },
] as const;
export const devices = [
  { key: "desktop", label: t("Desktop", "桌面") },
  { key: "phone", label: t("Phone", "手机") },
] as const;

export const PACKS: [string, T][] = [
  ["cyber", t("Cyber", "赛博")],
  ["gilded", t("Gilded", "鎏金")],
  ["kawaii", t("Kawaii", "可爱")],
  ["dread", t("Dread", "暗黑")],
  ["fantasy", t("Fantasy", "奇幻")],
  ["scifi", t("Sci-fi", "科幻")],
  ["pixel", t("Pixel", "像素")],
  ["storybook", t("Storybook", "绘本")],
  ["tactical", t("Tactical", "战术")],
  ["broadcast", t("Broadcast", "转播")],
  ["cartoon", t("Cartoon", "卡通")],
  ["craft", t("Craft", "手作")],
];
/** [key, label, desktop sheet cell (row, col) of 7×3 at 640×360, phone sheet cell of 10×3 at 390×844] */
const PACK_SCREENS: [string, T, [number, number], [number, number]][] = [
  ["title", t("Title", "标题"), [0, 1], [0, 1]],
  ["shop", t("Shop", "商店"), [1, 3], [1, 0]],
  ["inventory", t("Inventory", "背包"), [1, 4], [1, 1]],
  ["levels", t("Levels", "关卡"), [1, 5], [1, 2]],
  ["complete", t("Results", "结算"), [1, 6], [1, 3]],
  ["leaderboard", t("Leaderboard", "排行榜"), [2, 0], [1, 4]],
  ["settings", t("Settings", "设置"), [0, 3], [0, 3]],
];
export const packScreens = PACK_SCREENS.map(([key, label]) => ({ key, label }));
export const genres = GENRES.map(([key, , , , label]) => ({ key, label }));

export interface Cell { id: string; file: string; crop?: string; width: number }

/** Every frame the grid can show, by id `g/<genre>/<locale>/<device>` or `p/<pack>/<screen>/<device>`. */
export function cells(): Cell[] {
  const out: Cell[] = [];
  for (const [key, group, state, name] of GENRES)
    for (const { key: loc } of locales)
      for (const { key: dev } of devices) {
        const phone = dev === "phone" ? "phone-" : "";
        const screen = loc === "en" ? state : `${loc}-${name}`;
        out.push({ id: `g/${key}/${loc}/${dev}`, file: `${REF}/${group}-${phone}${screen}.png`, width: dev === "phone" ? 585 : 1280 });
      }
  for (const [pack] of PACKS)
    for (const [screen, , [dr, dc], [pr, pc]] of PACK_SCREENS) {
      out.push({ id: `p/${pack}/${screen}/desktop`, file: `${REF}/kit-style-${pack}.png`, crop: `640x360+${dc * 640}+${dr * 360}`, width: 640 });
      out.push({ id: `p/${pack}/${screen}/phone`, file: `${REF}/kit-style-${pack}-phone.png`, crop: `390x844+${pc * 390}+${pr * 844}`, width: 390 });
    }
  return out;
}
