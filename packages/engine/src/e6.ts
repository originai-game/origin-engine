// The E6 modules' namespaces (IDL `nav`, `ai`, `platform`, `i18n`) and the depth wave's `net`: thin calls into the host's
// binding symbols, typed by the generated declarations through index.ts.

import type { HostAbi } from "./generated/abi.ts";
import { boundHost } from "./runtime.ts";

type W = { readonly id: number };

function abi(): HostAbi<Uint32Array, unknown> {
  const h = boundHost();
  if (!h) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  return h.abi;
}

const parse = <T>(text: string): T => JSON.parse(text) as T;
const json = (value: unknown): string | undefined => (value === undefined ? undefined : JSON.stringify(value));

export const nav = Object.freeze({
  grid: (world: W, desc?: unknown) => parse(abi().og_nav_grid(world.id, json(desc))),
  gridFromTilemap: (world: W, desc?: unknown) => parse(abi().og_nav_grid_from_tilemap(world.id, json(desc))),
  setCost: (world: W, grid: number, x: number, y: number, cost: number) => parse(abi().og_nav_set_cost(world.id, grid, x, y, cost)),
  cellAt: (world: W, grid: number, point: unknown) => parse(abi().og_nav_cell_at(world.id, grid, JSON.stringify(point))),
  findPath: (world: W, desc?: unknown) => parse(abi().og_nav_find_path(world.id, json(desc))),
  flowField: (world: W, desc?: unknown) => parse(abi().og_nav_flow_field(world.id, json(desc))),
  setFlowTargets: (world: W, field: number, targets: unknown) => parse(abi().og_nav_set_flow_targets(world.id, field, JSON.stringify(targets))),
  flowAt: (world: W, field: number, point: unknown) => parse(abi().og_nav_flow_at(world.id, field, JSON.stringify(point))),
  load: (world: W, path: string, profile?: string) => parse(abi().og_nav_load(world.id, path, profile)),
  build: (world: W, desc?: unknown) => parse(abi().og_nav_build(world.id, json(desc))),
  setObstacle: (world: W, mesh: number, id: number, obstacle?: unknown) => parse(abi().og_nav_set_obstacle(world.id, mesh, id, json(obstacle))),
  carve: (world: W, mesh: number, id: number, shape?: unknown) => parse(abi().og_nav_carve(world.id, mesh, id, json(shape))),
  setArea: (world: W, mesh: number, id: number, desc?: unknown) => parse(abi().og_nav_set_area(world.id, mesh, id, json(desc))),
  setLink: (world: W, mesh: number, id: number, desc?: unknown) => parse(abi().og_nav_set_link(world.id, mesh, id, json(desc))),
  setLinkOpen: (world: W, mesh: number, id: number, open: boolean) => parse(abi().og_nav_set_link_open(world.id, mesh, id, open)),
  finishLink: (world: W, entity: number) => abi().og_nav_finish_link(world.id, entity),
  nearest: (world: W, mesh: number, point: unknown, extent?: unknown) => parse(abi().og_nav_nearest(world.id, mesh, JSON.stringify(point), json(extent))),
  pathOf: (world: W, entity: number) => parse(abi().og_nav_path_of(world.id, entity)),
});

export const ai = Object.freeze({
  load: (world: W, path: string) => parse(abi().og_ai_load(world.id, path)),
  parse: (world: W, text: string) => parse(abi().og_ai_parse(world.id, text)),
  get: (world: W, entity: number, key: string) => parse(abi().og_ai_get(world.id, entity, key)),
  set: (world: W, entity: number, key: string, value: unknown) => parse(abi().og_ai_set(world.id, entity, key, JSON.stringify(value))),
  restart: (world: W, entity: number) => parse(abi().og_ai_restart(world.id, entity)),
  finish: (world: W, entity: number, success: boolean, action?: string) => abi().og_ai_finish(world.id, entity, success, action),
  noise: (world: W, desc?: unknown) => parse(abi().og_ai_noise(world.id, json(desc))),
  describe: (world: W, entity: number) => parse(abi().og_ai_describe(world.id, entity)),
});

/** og.story (G2-8): dialogues and quests as data; the story is world state. */
export const story = Object.freeze({
  load: (world: W, path: string) => parse(abi().og_story_load(world.id, path)),
  parse: (world: W, text: string) => parse(abi().og_story_parse(world.id, text)),
  start: (world: W, desc?: unknown) => parse(abi().og_story_start(world.id, json(desc))),
  view: (world: W, conversation: number) => parse(abi().og_story_view(world.id, conversation)),
  history: (world: W, conversation: number) => parse(abi().og_story_history(world.id, conversation)),
  conversations: (world: W) => parse(abi().og_story_conversations(world.id)),
  advance: (world: W, conversation: number) => parse(abi().og_story_advance(world.id, conversation)),
  choose: (world: W, conversation: number, index: number) => parse(abi().og_story_choose(world.id, conversation, index)),
  stop: (world: W, conversation: number) => abi().og_story_stop(world.id, conversation),
  bark: (world: W, desc?: unknown) => parse(abi().og_story_bark(world.id, json(desc))),
  ambient: (world: W, desc?: unknown) => parse(abi().og_story_ambient(world.id, json(desc))),
  barks: (world: W) => parse(abi().og_story_barks(world.id)),
  get: (world: W, name: string) => parse(abi().og_story_get(world.id, name)),
  set: (world: W, name: string, value: unknown) => parse(abi().og_story_set(world.id, name, JSON.stringify(value))),
  reset: (world: W) => abi().og_story_reset(world.id),
  startQuest: (world: W, id: string) => parse(abi().og_story_start_quest(world.id, id)),
  progress: (world: W, quest: string, objective: string, amount?: number) =>
    abi().og_story_progress(world.id, quest, objective, amount),
  setStage: (world: W, quest: string, stage: string) => parse(abi().og_story_set_stage(world.id, quest, stage)),
  completeQuest: (world: W, quest: string) => parse(abi().og_story_complete_quest(world.id, quest)),
  failQuest: (world: W, quest: string) => parse(abi().og_story_fail_quest(world.id, quest)),
  quest: (world: W, id: string) => parse(abi().og_story_quest(world.id, id)),
  quests: (world: W) => parse(abi().og_story_quests(world.id)),
  track: (world: W, id: string, on: boolean) => abi().og_story_track(world.id, id, on),
  markSeen: (world: W, id: string) => abi().og_story_mark_seen(world.id, id),
  mark: (world: W, id: string, desc?: unknown) => abi().og_story_mark(world.id, id, json(desc)),
});

export const platform = Object.freeze({
  storeGet: (world: W, key: string) => parse(abi().og_platform_store_get(world.id, key)),
  storeSet: (world: W, desc?: unknown) => parse(abi().og_platform_store_set(world.id, json(desc))),
  storeList: (world: W) => abi().og_platform_store_list(world.id),
  leaderboardSubmit: (world: W, desc?: unknown) => parse(abi().og_platform_leaderboard_submit(world.id, json(desc))),
  leaderboardTop: (world: W, desc?: unknown) => abi().og_platform_leaderboard_top(world.id, json(desc)),
  leaderboardAround: (world: W, desc?: unknown) => abi().og_platform_leaderboard_around(world.id, json(desc)),
  telemetry: (world: W, name: string, attrs?: unknown) => parse(abi().og_platform_telemetry(world.id, name, json(attrs))),
  point: (world: W, desc?: unknown) => parse(abi().og_platform_point(world.id, json(desc))),
  aiMark: (world: W, desc?: unknown) => parse(abi().og_platform_ai_mark(world.id, json(desc))),
  loadSettings: (world: W) => abi().og_platform_load_settings(world.id),
  saveSettings: (world: W, value: unknown) => parse(abi().og_platform_save_settings(world.id, JSON.stringify(value))),
  saveSlot: (world: W, desc?: unknown) => parse(abi().og_platform_save_slot(world.id, json(desc))),
  loadSlot: (world: W, desc?: unknown) => parse(abi().og_platform_load_slot(world.id, json(desc))),
  products: (world: W, desc?: unknown) => parse(abi().og_platform_products(world.id, json(desc))),
  purchase: (world: W, desc?: unknown) => parse(abi().og_platform_purchase(world.id, json(desc))),
  entitlements: (world: W) => abi().og_platform_entitlements(world.id),
  consume: (world: W, purchase: string) => parse(abi().og_platform_consume(world.id, purchase)),
  grant: (world: W, purchase: string) => parse(abi().og_platform_grant(world.id, purchase)),
  granted: (world: W, purchase: string) => abi().og_platform_granted(world.id, purchase),
  adReady: (world: W, desc?: unknown) => parse(abi().og_platform_ad_ready(world.id, json(desc))),
  showAd: (world: W, desc?: unknown) => parse(abi().og_platform_show_ad(world.id, json(desc))),
  achievementUnlock: (world: W, id: string) => parse(abi().og_platform_achievement_unlock(world.id, id)),
  achievementProgress: (world: W, desc?: unknown) => parse(abi().og_platform_achievement_progress(world.id, json(desc))),
  achievements: (world: W) => abi().og_platform_achievements(world.id),
  player: (world: W) => abi().og_platform_player(world.id),
  friends: (world: W, desc?: unknown) => abi().og_platform_friends(world.id, json(desc)),
  inviteSend: (world: W, desc?: unknown) => parse(abi().og_platform_invite_send(world.id, json(desc))),
  invites: (world: W) => abi().og_platform_invites(world.id),
  share: (world: W, desc?: unknown) => parse(abi().og_platform_share(world.id, json(desc))),
  supports: (world: W, method: string) => abi().og_platform_supports(world.id, method),
  notice: (world: W, notice: number) => parse(abi().og_platform_notice(world.id, notice)),
  result: (world: W, request: number) => parse(abi().og_platform_result(world.id, request)),
  info: (world: W) => parse(abi().og_platform_info(world.id)),
  now: (world: W) => abi().og_platform_now(world.id),
  roomCreate: (world: W, desc?: unknown) => parse(abi().og_platform_room_create(world.id, json(desc))),
  roomJoin: (world: W, room: string) => parse(abi().og_platform_room_join(world.id, room)),
  quickPlay: (world: W, desc?: unknown) => parse(abi().og_platform_quick_play(world.id, json(desc))),
  roomLeave: (world: W) => parse(abi().og_platform_room_leave(world.id)),
  // Bytes cross as { "$og.bytes": base64 } (og-runtime net.rs BYTES_KEY).
  roomSend: (world: W, data: unknown, to?: number) =>
    parse(abi().og_platform_room_send(world.id, JSON.stringify(data instanceof Uint8Array ? { "$og.bytes": toBase64(data) } : (data ?? null)), to)),
  roomPeers: (world: W) => parse(abi().og_platform_room_peers(world.id)),
  roomInfo: (world: W) => parse(abi().og_platform_room_info(world.id)),
  roomMessage: (world: W, message: number) => {
    const m = parse<{ bytes: string | null } | null>(abi().og_platform_room_message(world.id, message));
    return m && { ...m, bytes: typeof m.bytes === "string" ? fromBase64(m.bytes) : null };
  },
});

export const net = Object.freeze({
  submit: (world: W, input?: unknown) => parse(abi().og_net_submit(world.id, json(input))),
  shape: (world: W, desc?: unknown) => parse(abi().og_net_shape(world.id, json(desc))),
  submitInput: (world: W, component: { name: string }, value?: unknown) =>
    parse(abi().og_net_submit_input(world.id, component.name, JSON.stringify(value ?? {}))),
  command: (world: W, component: { name: string }, value?: unknown) =>
    parse(abi().og_net_command(world.id, component.name, JSON.stringify(value ?? {}))),
  commands: (world: W, component: { name: string }) => parse(abi().og_net_commands(world.id, component.name)),
  status: (world: W) => parse(abi().og_net_status(world.id)),
  hash: (world: W) => parse<string>(abi().og_net_hash(world.id)),
  resimulating: (world: W) => abi().og_net_resimulating(world.id) as boolean,
  finalHash: (world: W, tick: number) => parse<string | null>(abi().og_net_final_hash(world.id, tick)),
});

function toBase64(bytes: Uint8Array): string {
  let bin = "";
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]!);
  return btoa(bin);
}

function fromBase64(text: string): Uint8Array {
  const bin = atob(text);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

export const i18n = Object.freeze({
  load: (world: W, path: string) => parse(abi().og_i18n_load(world.id, path)),
  setLocale: (world: W, locale: string) => parse(abi().og_i18n_set_locale(world.id, locale)),
  locale: (world: W) => parse(abi().og_i18n_locale(world.id)),
  t: (world: W, key: string, args?: unknown) => abi().og_i18n_t(world.id, key, json(args)),
  has: (world: W, key: string) => abi().og_i18n_has(world.id, key),
  formatNumber: (world: W, value: number, format?: unknown) => abi().og_i18n_format_number(world.id, value, json(format)),
  formatCompact: (world: W, value: number, decimals?: number) => abi().og_i18n_format_compact(world.id, value, decimals),
  formatCurrency: (world: W, amount: number, currency: string) => abi().og_i18n_format_currency(world.id, amount, currency),
  formatDate: (world: W, seconds: number, format?: unknown) => abi().og_i18n_format_date(world.id, seconds, json(format)),
  formatTime: (world: W, seconds: number, format?: unknown) => abi().og_i18n_format_time(world.id, seconds, json(format)),
  formatDuration: (world: W, seconds: number, style?: string) => abi().og_i18n_format_duration(world.id, seconds, style),
});
