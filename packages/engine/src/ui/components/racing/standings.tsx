import { For, Scroll, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./racing.en.ogt";

export interface RacingStandingsProps {
  /** The tower's name for a reader; empty takes the kit's (Standings). Not drawn. */
  title: string;
  /** The running order as the game ranks it, keyed rows of { id, position, name, gap, color, player, state }: position the place shown in its box; gap the interval as the game words it (+1.204, +1 lap; the leader's empty gap reads Leader); color the car's or team's colour as a CSS colour, on a stripe beside the name (empty: a neutral stripe); player marks the local player's row (lit in the HUD's accent, aria-current); state racing, pitting, retired or finished, each a word on a tag (Pit, DNF, Fin), a retired row dimmed. */
  rows?: ReadonlyArray<{ id: string; position: number; name: string; gap: string; color: string; player: boolean; state: string }>;
  /** How short its place is: empty shows every row; short keeps the leader and the rows beside the player; shorter the leader and the player. */
  room: string;
  /** A row is the player's. On a phone the tower keeps that row and the rows beside it, or the first three when false. */
  hasPlayer: boolean;
}

/**
 * The position tower at the left edge: one leaning glass row per car in
 * the order the game ranks them, the place in a box, the car's colour on a
 * stripe, the name and the gap (Leader for the leader). The player's row
 * is lit in the HUD's accent; pitting, retired and finished cars carry a
 * word on a tag. On a phone it collapses to the leader and the rows
 * either side of the player. Read only.
 */
export function RacingStandings(props: RacingStandingsProps) {
  const name = () => (props.title !== "" ? props.title : t("racing.standings.name"));
  const rows = () => props.rows ?? [];
  const keep = (i: number, r: { player: boolean }) => {
    const room = props.room;
    if (room === "shorter") return i === 0 || r.player;
    if (room === "short") {
      const beside = (i > 0 && rows()[i - 1]?.player) || (i + 1 < rows().length && rows()[i + 1]?.player);
      return i === 0 || r.player || beside;
    }
    return true;
  };
  return (
    <View
      class={`group/standings flex-col w-62.5 max-w-full max-h-full text-fg compact:w-full`}
      label={name()}
      data-near={props.room === "short" || props.room === "shorter"}
      data-only={props.room === "shorter"}
      data-fallback={!props.hasPlayer}
    >
      <Show when={rows().length} fallback={
        <Text class={`py-1 px-2 text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted bg-glass`} role="status">{t("racing.standings.empty")}</Text>
      }>
        <Scroll class="w-full min-h-0 flex-1">
          <View class="flex-col w-full gap-y-1 compact:gap-y-0.5" role="list" label={name()}>
            <For each={rows()} key={(r) => r.id}>
              {(r, i) => (
                <View
                  class={`group/srow flex-row items-center gap-x-2 w-full min-w-0 pr-3 bg-glass nine-plate-cut data-off:hidden data-retired:opacity-muted data-player:bg-hud-accent compact:gap-x-1.5 compact:pr-2 compact:hidden compact:data-on:flex compact:group-data-fallback/standings:data-top:flex`}
                  role="listitem"
                  label={r().name}
                  current={r().player}
                  data-on={keep(i(), r())}
                  data-off={!keep(i(), r())}
                  data-top={i() < 3}
                  data-player={r().player}
                  data-retired={r().state === "retired"}
                >
                  <Text class={`shrink-0 w-7.5 leading-6.5 text-center text-reading font-bold tabular-nums whitespace-nowrap text-fg bg-surface group-data-player/srow:bg-plate-hi compact:w-6 compact:leading-5 compact:text-caption`}>
                    {formatNumber(r().position)}
                  </Text>
                  <View
                    class={`shrink-0 w-1.25 h-5 bg-sheen compact:h-3.5`}
                    style={{ backgroundColor: r().color || undefined }}
                  />
                  <Text class={`flex-1 min-w-0 text-reading font-semibold whitespace-nowrap overflow-hidden text-fg group-data-player/srow:text-hud-accent-fg compact:text-caption`}>{r().name}</Text>
                  <Show when={r().state !== "racing"}>
                    <Text
                      class={`shrink-0 py-0 px-1.25 text-xs font-bold tracking-label uppercase whitespace-nowrap text-fg bg-well nine-chip-cut data-pitting:text-gold data-retired:text-foe-ink ${packClass("Chip", "frame")}`}
                      label={r().state === "pitting" ? t("racing.standings.pittingName") : r().state === "retired" ? t("racing.standings.retiredName") : t("racing.standings.finishedName")}
                      data-pitting={r().state === "pitting"}
                      data-retired={r().state === "retired"}
                    >
                      {r().state === "pitting" ? t("racing.standings.pitting") : r().state === "retired" ? t("racing.standings.retired") : t("racing.standings.finished")}
                    </Text>
                  </Show>
                  <Show when={r().gap !== "" || (r().position === 1 && r().state !== "retired")}>
                    <Text class={`shrink-0 text-reading font-bold tabular-nums whitespace-nowrap text-fg data-leader:text-fg-muted group-data-player/srow:text-hud-accent-fg compact:text-caption`} data-leader={r().gap === ""}>
                      {r().gap !== "" ? r().gap : t("racing.standings.leader")}
                    </Text>
                  </Show>
                </View>
              )}
            </For>
          </View>
        </Scroll>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingStandingsProps>> = {
  default: { "title": "", "rows": [{ "id": "okafor", "position": 1, "name": "M. Okafor", "gap": "", "color": "#ffd24a", "player": false, "state": "racing" }, { "id": "lindqvist", "position": 2, "name": "S. Lindqvist", "gap": "+1.204", "color": "#4f7bff", "player": false, "state": "racing" }, { "id": "you", "position": 3, "name": "You", "gap": "+1.861", "color": "#ff4d5e", "player": true, "state": "racing" }, { "id": "castellanos", "position": 4, "name": "R. Castellanos", "gap": "+0.337", "color": "#55e39a", "player": false, "state": "racing" }, { "id": "nakamura", "position": 5, "name": "T. Nakamura", "gap": "+2.950", "color": "#c98bff", "player": false, "state": "pitting" }], "hasPlayer": true },
  leading: { "title": "", "rows": [{ "id": "you", "position": 1, "name": "You", "gap": "", "color": "#ff4d5e", "player": true, "state": "racing" }, { "id": "okafor", "position": 2, "name": "M. Okafor", "gap": "+0.884", "color": "#ffd24a", "player": false, "state": "racing" }, { "id": "lindqvist", "position": 3, "name": "S. Lindqvist", "gap": "+2.310", "color": "#4f7bff", "player": false, "state": "racing" }], "hasPlayer": true },
  last: { "title": "", "rows": [{ "id": "okafor", "position": 1, "name": "M. Okafor", "gap": "", "color": "#ffd24a", "player": false, "state": "racing" }, { "id": "lindqvist", "position": 2, "name": "S. Lindqvist", "gap": "+1.204", "color": "#4f7bff", "player": false, "state": "racing" }, { "id": "castellanos", "position": 3, "name": "R. Castellanos", "gap": "+0.337", "color": "#55e39a", "player": false, "state": "racing" }, { "id": "nakamura", "position": 4, "name": "T. Nakamura", "gap": "+2.950", "color": "#c98bff", "player": false, "state": "racing" }, { "id": "brandt", "position": 5, "name": "L. Brandt", "gap": "+3.107", "color": "", "player": false, "state": "racing" }, { "id": "sato", "position": 6, "name": "K. Sato", "gap": "+4.020", "color": "#38e8ff", "player": false, "state": "pitting" }, { "id": "you", "position": 7, "name": "You", "gap": "+6.512", "color": "#ff4d5e", "player": true, "state": "racing" }, { "id": "marlow", "position": 8, "name": "J. Marlow", "gap": "", "color": "#ff9a48", "player": false, "state": "retired" }], "hasPlayer": true },
  empty: { "title": "", "rows": [], "hasPlayer": false },
  single: { "title": "Time trial", "rows": [{ "id": "you", "position": 1, "name": "You", "gap": "", "color": "", "player": true, "state": "racing" }], "hasPlayer": true },
  finished: { "title": "", "rows": [{ "id": "okafor", "position": 1, "name": "M. Okafor", "gap": "", "color": "#ffd24a", "player": false, "state": "finished" }, { "id": "you", "position": 2, "name": "You", "gap": "+0.207", "color": "#ff4d5e", "player": true, "state": "finished" }, { "id": "lindqvist", "position": 3, "name": "S. Lindqvist", "gap": "+2.940", "color": "#4f7bff", "player": false, "state": "racing" }], "hasPlayer": true },
  overflow: { "title": "", "rows": [{ "id": "r1", "position": 1, "name": "Maximilian Vandenberghe-Okonkwo", "gap": "", "color": "#ffd24a", "player": false, "state": "racing" }, { "id": "r2", "position": 2, "name": "Okafor", "gap": "+0.884", "color": "#4f7bff", "player": false, "state": "racing" }, { "id": "r3", "position": 3, "name": "Lindqvist", "gap": "+1.020", "color": "", "player": false, "state": "racing" }, { "id": "r4", "position": 4, "name": "Sato", "gap": "+2.310", "color": "", "player": false, "state": "racing" }, { "id": "r5", "position": 5, "name": "Brandt", "gap": "+3.007", "color": "", "player": false, "state": "pitting" }, { "id": "r6", "position": 6, "name": "Duarte", "gap": "+4.551", "color": "", "player": false, "state": "racing" }, { "id": "r7", "position": 7, "name": "Nakamura", "gap": "+5.902", "color": "", "player": false, "state": "racing" }, { "id": "r8", "position": 8, "name": "Haddad", "gap": "+7.118", "color": "", "player": false, "state": "racing" }, { "id": "r9", "position": 9, "name": "Kowalczyk", "gap": "+8.640", "color": "", "player": false, "state": "racing" }, { "id": "r10", "position": 10, "name": "Moreau", "gap": "+9.215", "color": "", "player": false, "state": "racing" }, { "id": "r11", "position": 11, "name": "Rhea Vance-Abernathy of the Coastal Racing Club", "gap": "+10.774", "color": "#ff4d5e", "player": true, "state": "racing" }, { "id": "r12", "position": 12, "name": "Ferreira", "gap": "+12.330", "color": "", "player": false, "state": "racing" }, { "id": "r13", "position": 13, "name": "Johansson", "gap": "+15.004", "color": "", "player": false, "state": "racing" }, { "id": "r14", "position": 14, "name": "Abernathy-Whitcombe", "gap": "+1 lap", "color": "", "player": false, "state": "racing" }, { "id": "r15", "position": 15, "name": "Ivanova", "gap": "+1 lap", "color": "", "player": false, "state": "racing" }, { "id": "r16", "position": 16, "name": "Quinn", "gap": "+2 laps", "color": "", "player": false, "state": "racing" }, { "id": "r17", "position": 17, "name": "Petrov", "gap": "", "color": "", "player": false, "state": "retired" }, { "id": "r18", "position": 18, "name": "Laurent", "gap": "", "color": "", "player": false, "state": "retired" }], "hasPlayer": true },
};
