import { For, Scroll, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./arena.en.ogt";

export interface ScoreboardRow {
  /** The player's key. */
  id: string;
  /** a or b. */
  team?: string;
  /** What a reader calls them. */
  name?: string;
  /** Kills as the game counts them. */
  kills?: number;
  /** Deaths as the game counts them. */
  deaths?: number;
  /** Assists as the game counts them. */
  assists?: number;
  /** The score as the game counts it. */
  score?: number;
  /** The local player's row (You, the row is current). */
  local?: boolean;
  /** The row is dimmed and named Disconnected. */
  disconnected?: boolean;
}

export interface ArenaScoreboardProps {
  /** The board's heading; empty takes the kit's (Scoreboard). */
  title?: string;
  /** What each row shows: kda (kills, deaths, assists and score) or score (score only). */
  columns?: string;
  /** The first team's name; empty takes the kit's (Blue team). */
  nameA?: string;
  /** The second team's name; empty takes the kit's (Red team). */
  nameB?: string;
  /** The first team's score as the game words it (rounds, points); empty hides it. */
  scoreA?: string;
  /** The second team's score as the game words it; empty hides it. */
  scoreB?: string;
  /** Every player in the game's order. */
  rows?: ReadonlyArray<ScoreboardRow>;
  /** The two teams stand side by side; otherwise one above the other. */
  split?: boolean;
}

/**
 * The team scoreboard over the match, bare: the title small and centred,
 * each team a table headed by its name in its colour and its score set
 * large, its rows on dark glass washed in the team's colour from an edge
 * of it, the local player's row lit white, a disconnected player's row
 * dimmed with its name kept and the tag beside it. Side by side where the
 * board is wide, stacked where it is not. Read only; it scrolls inside
 * the height it is given. The game counts every figure; the board only
 * shows them. Approximations: the team inks are the default blue and red
 * (the game's seed-team colours have no class surface), the rows' wash is
 * a linear gradient and the local row's layered wash a flat tint, the
 * figures' condensed stretch is dropped, a disconnected row dims by
 * opacity, the hover wash is dropped (the same fill), the scroll fade
 * at the list's foot is dropped, and a long word wraps without breaking.
 */
const TAG = `shrink-0 py-0 px-1 text-xs leading-normal font-bold tracking-label uppercase text-fg nine-chip-cut bg-raised-pressed whitespace-nowrap ${packClass("Chip", "frame")}`;
const CAPTION = "text-end whitespace-nowrap text-xs font-semibold tracking-label uppercase text-fg-muted";
const STAT = "text-end whitespace-nowrap text-base font-bold tabular-nums compact:flex-1 compact:min-w-0";

function TeamTable(props: { team: "a" | "b"; name: string; score: string; columns: string; rows: ReadonlyArray<ScoreboardRow> }) {
  const blue = props.team == "a";
  const ink = blue ? "text-team1" : "text-team2";
  const rule = blue ? "inset-shadow-underline inset-shadow-team1" : "inset-shadow-underline inset-shadow-team2";
  const glow = blue ? "shadow-glow-lg shadow-team1" : "shadow-glow-lg shadow-team2";
    const edge = blue ? "border-l-3 border-team1" : "border-l-3 border-team2";
  const kda = () => props.columns == "kda";
  return (
    <View class="min-w-0 flex-col" role="table" label={props.name}>
      <View class={`flex-row items-end w-full ${rule}`} role="row">
        <View class="flex-1 min-w-0 flex-row items-baseline gap-x-3 py-1 px-2" role="columnheader">
          <Text class="min-w-0 text-title font-bold tracking-sm uppercase text-fg text-start">{props.name}</Text>
          <Show when={props.score != ""}>
            <Text class={`flex-none min-w-8 text-center whitespace-nowrap text-display-md font-bold tabular-nums ${ink} ${glow}`} description={t("arena.score.teamScore")}>{props.score}</Text>
          </Show>
        </View>
        <Show when={kda()}>
          <Text class={`py-1 px-2 min-w-8 ${CAPTION}`} role="columnheader" label={t("arena.score.killsName")}>{t("arena.score.kills")}</Text>
          <Text class={`py-1 px-2 min-w-8 ${CAPTION}`} role="columnheader" label={t("arena.score.deathsName")}>{t("arena.score.deaths")}</Text>
          <Text class={`py-1 px-2 min-w-8 ${CAPTION}`} role="columnheader" label={t("arena.score.assistsName")}>{t("arena.score.assists")}</Text>
        </Show>
        <Text class={`py-1 px-2 ${CAPTION}`} role="columnheader">{t("arena.score.score")}</Text>
      </View>
      <View class="flex-col gap-y-0.5">
        <For each={props.rows ?? []} key={(r) => r.id}>
          {(r) => (
            <View
              class={`flex-row items-center min-h-8 compact:flex-wrap ${edge} ${r().disconnected ? packClass("Row", "frame", "muted") : r().local ? packClass("Row", "frame", "selected") : packClass("Row", "frame")}`}
              role="row"
              current={r().local === true}
              data-local={r().local === true}
              data-disconnected={r().disconnected === true}
            >
              <View class="flex-1 min-w-0 flex-row flex-wrap items-center gap-x-1 gap-y-0.5 py-1 px-2 compact:basis-full compact:grow compact:shrink" role="rowheader">
                <Text class="min-w-0 text-start text-reading font-semibold text-inherit">{r().name}</Text>
                <Show when={r().local === true}>
                  <Text class={TAG}>{t("arena.you")}</Text>
                </Show>
                <Show when={r().disconnected === true}>
                  <Text class={TAG}>{t("arena.score.disconnected")}</Text>
                </Show>
              </View>
              <Show when={kda()}>
                <Text class={`py-1 px-2 min-w-8 ${STAT}`} role="cell">{formatNumber(r().kills ?? 0)}</Text>
                <Text class={`py-1 px-2 min-w-8 ${STAT}`} role="cell">{formatNumber(r().deaths ?? 0)}</Text>
                <Text class={`py-1 px-2 min-w-8 ${STAT}`} role="cell">{formatNumber(r().assists ?? 0)}</Text>
              </Show>
              <Text class="py-1 px-2 text-end whitespace-nowrap text-title font-bold tabular-nums compact:flex-[2_1_0] compact:min-w-0" role="cell">{formatNumber(r().score ?? 0)}</Text>
            </View>
          )}
        </For>
      </View>
    </View>
  );
}

/**
 * A two-team scoreboard: each team's name and score and its players' rows,
 * kills, deaths, assists and score or score alone, side by side or one
 * above the other. Read only; the game orders the rows.
 */
export function ArenaScoreboard(props: ArenaScoreboardProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("arena.score.title");
  const rows = () => props.rows ?? [];
  const teamA = () => rows().filter((r) => r.team == "a");
  const teamB = () => rows().filter((r) => r.team == "b");
  const nameA = () => props.nameA || t("arena.team.a");
  const nameB = () => props.nameB || t("arena.team.b");
  return (
    <View class={`w-full min-w-0 max-h-full flex-col gap-y-3 p-4 text-fg bg-glass nine-panel-cut ${packClass("Panel", "frame")}`} label={heading()}>
      <View class="flex-none flex-row w-full min-h-0">
        <Text class="flex-1 min-w-0 text-center text-base font-semibold tracking-label uppercase text-fg-muted">{heading()}</Text>
      </View>
      <Show when={rows().length} fallback={
        <Text class="p-4 text-center text-xs font-semibold tracking-label uppercase text-fg-muted" role="status">{t("arena.score.empty")}</Text>
      }>
        <Scroll class="min-h-0 flex-1">
          <View class="grid grid-cols-1 items-start gap-4 w-full data-split:grid-cols-2" data-split={props.split === true}>
            <TeamTable team="a" name={nameA()} score={props.scoreA ?? ""} columns={props.columns ?? "kda"} rows={teamA()} />
            <TeamTable team="b" name={nameB()} score={props.scoreB ?? ""} columns={props.columns ?? "kda"} rows={teamB()} />
          </View>
        </Scroll>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ArenaScoreboardProps>> = {
  default: {"title":"","columns":"kda","nameA":"","nameB":"","scoreA":"7","scoreB":"5","split":false,"rows":[{"id":"kai","team":"a","name":"Kai","kills":12,"deaths":4,"assists":6,"score":2840,"local":false,"disconnected":false},{"id":"you","team":"a","name":"Rhea","kills":9,"deaths":5,"assists":11,"score":2410,"local":true,"disconnected":false},{"id":"ana","team":"a","name":"Ana","kills":3,"deaths":7,"assists":14,"score":1720,"local":false,"disconnected":false},{"id":"rook","team":"b","name":"Rook","kills":10,"deaths":8,"assists":3,"score":2290,"local":false,"disconnected":false},{"id":"wren","team":"b","name":"Wren","kills":6,"deaths":9,"assists":7,"score":1650,"local":false,"disconnected":true},{"id":"quill","team":"b","name":"Quill","kills":4,"deaths":10,"assists":9,"score":1310,"local":false,"disconnected":false}]},
  score: {"title":"","columns":"score","nameA":"Iron Wolves","nameB":"Ember Court","scoreA":"2","scoreB":"3","split":false,"rows":[{"id":"kai","team":"a","name":"Kai","kills":0,"deaths":0,"assists":0,"score":34,"local":false,"disconnected":false},{"id":"you","team":"a","name":"Rhea","kills":0,"deaths":0,"assists":0,"score":28,"local":true,"disconnected":false},{"id":"rook","team":"b","name":"Rook","kills":0,"deaths":0,"assists":0,"score":41,"local":false,"disconnected":false},{"id":"wren","team":"b","name":"Wren","kills":0,"deaths":0,"assists":0,"score":19,"local":false,"disconnected":false}]},
  disconnected: {"title":"","columns":"kda","nameA":"","nameB":"","scoreA":"0","scoreB":"1","split":false,"rows":[{"id":"you","team":"a","name":"Rhea","kills":2,"deaths":1,"assists":0,"score":420,"local":true,"disconnected":false},{"id":"kai","team":"a","name":"Kai","kills":0,"deaths":0,"assists":0,"score":0,"local":false,"disconnected":true},{"id":"rook","team":"b","name":"Rook","kills":1,"deaths":2,"assists":0,"score":310,"local":false,"disconnected":true},{"id":"wren","team":"b","name":"Wren","kills":0,"deaths":0,"assists":1,"score":90,"local":false,"disconnected":true}]},
  overflow: {"title":"Championship Series · Grand Finals · Map 5 of 5 · Overtime","columns":"kda","nameA":"The Unbroken Wardens of the Ninefold Gate","nameB":"Ember Court Invitational Reserve","scoreA":"12","scoreB":"12","split":true,"rows":[{"id":"a1","team":"a","name":"Maximilian Vandenberghe-Okonkwo","kills":128,"deaths":3,"assists":214,"score":184320,"local":false,"disconnected":false},{"id":"a2","team":"a","name":"Rhea","kills":31,"deaths":22,"assists":40,"score":9840,"local":true,"disconnected":false},{"id":"a3","team":"a","name":"Ana","kills":12,"deaths":30,"assists":51,"score":7210,"local":false,"disconnected":false},{"id":"a4","team":"a","name":"Kai","kills":20,"deaths":25,"assists":18,"score":6600,"local":false,"disconnected":true},{"id":"a5","team":"a","name":"Jo","kills":9,"deaths":28,"assists":33,"score":5120,"local":false,"disconnected":false},{"id":"b1","team":"b","name":"Abernathy-Whitcombe","kills":44,"deaths":19,"assists":20,"score":11250,"local":false,"disconnected":false},{"id":"b2","team":"b","name":"Wren","kills":25,"deaths":27,"assists":30,"score":8030,"local":false,"disconnected":false},{"id":"b3","team":"b","name":"Rook","kills":18,"deaths":24,"assists":22,"score":6900,"local":false,"disconnected":false},{"id":"b4","team":"b","name":"Moreau","kills":15,"deaths":29,"assists":26,"score":6010,"local":false,"disconnected":false},{"id":"b5","team":"b","name":"Laurent","kills":7,"deaths":33,"assists":41,"score":4880,"local":false,"disconnected":true}]},
};
