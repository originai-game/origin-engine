import { Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import { HudTimer } from "../hud/hud-timer";
import "./arena.en.ogt";

export interface ArenaRoundHudProps {
  /** The round being played, as the game counts it. */
  round?: number;
  /** How many rounds the match has (Round 3 of 7); 0 hides the total. */
  rounds?: number;
  /** The first team's name; empty takes the kit's (Blue team). */
  nameA?: string;
  /** The second team's name; empty takes the kit's (Red team). */
  nameB?: string;
  /** The first team's score (rounds won) as the game counts it. */
  scoreA?: number;
  /** The second team's score as the game counts it. */
  scoreB?: number;
  /** The local player's team, a or b (its plate is marked You); empty is a spectator. */
  localTeam?: string;
  /** Seconds left in the round as the game counts them (a HudTimer); negative hides it. Default -1. */
  timeLeft?: number;
  /** The round's whole time, for the timer's ring (0: no ring). */
  timeTotal?: number;
  /** The round's end as the game decided it, for the local side: won, lost, draw, or none (no banner). Default none. */
  outcome?: string;
  /** The banner's id: a new value replays its entrance. */
  at?: number;
  /** A line under the banner as the game words it (Flawless, Match point); empty hides it. */
  detail?: string;
}

/**
 * The round HUD along the top of a match: each team's name and score on
 * either side of the round and its timer (HudTimer, as the game counts
 * it), the local team's plate marked You. When the game ends a round it
 * sets outcome: a banner across the screen says Round won, Round lost or
 * Round drawn (a status announced at once), its entrance replayed for a
 * new id and skipped under reduced motion. The game counts the score and
 * the time and decides the outcome; read only. Approximations: team a's
 * plate wears the pack's cut and keeps no rim or team edge (a cut takes
 * one fill, and the pack has no mirrored cut, so team b keeps its
 * rounded corner, gradient and rim), the verdict's gradient text fill
 * and stamp entrance are a solid ink with a glow (the entrance replay
 * with the banner's id is dropped), the team inks are the default blue
 * and red (the game's seed-team colours have no class surface), the
 * banner's size clamp to the screen and the figure's condensed stretch
 * are dropped, and a phone keeps the row, the names above the scores
 * (team b's name keeps its place after the score: no order surface).
 */
const PLATE_RIM = "inset-shadow-lit inset-ring-1 inset-ring-rim";
const TEAM_PLATE = `bg-glass ${PLATE_RIM}`;
const TEAM_A_PLATE = "inset-shadow-underline inset-shadow-team1 inset-ring-1 inset-ring-rim";
const TEAM_B_PLATE = "inset-shadow-underline inset-shadow-team2 inset-ring-1 inset-ring-rim";
const NAME = "min-w-0 text-base leading-tight font-bold uppercase tracking-sm text-fg";
const SCORE_A = "shrink-0 min-w-8 text-center text-display-lg leading-none font-bold tabular-nums whitespace-nowrap text-team1 shadow-glow-lg shadow-team1 compact:text-3xl";
const SCORE_B = "shrink-0 min-w-8 text-center text-display-lg leading-none font-bold tabular-nums whitespace-nowrap text-team2 shadow-glow-lg shadow-team2 compact:text-3xl";
const YOU = `shrink-0 py-0 px-1 text-xs leading-normal font-bold tracking-label uppercase text-hud-accent-fg nine-chip-cut bg-hud-accent whitespace-nowrap`;
const ROUND_PLATE = `flex-col items-center justify-center gap-y-0.5 py-0.5 px-4 bg-raised-pressed ${PLATE_RIM} ${packClass("Plate", "frame")}`;

export function ArenaRoundHud(props: ArenaRoundHudProps) {
  const nameA = () => (props.nameA ?? "") != "" ? props.nameA : t("arena.team.a");
  const nameB = () => (props.nameB ?? "") != "" ? props.nameB : t("arena.team.b");
  const ended = () => props.outcome == "won" || props.outcome == "lost" || props.outcome == "draw";
  return (
    <View class="flex-col items-center gap-y-6 max-w-full min-w-0 pointer-events-none text-fg compact:gap-y-4" role="group" label={t("arena.round.name")}>
      <View class="flex-row items-stretch justify-center gap-x-0.5 max-w-full min-w-0 compact:w-full compact:flex-wrap">
        <View
          class={`shrink basis-auto min-w-0 flex-row items-center justify-end gap-x-2 nine-panel-cut bg-glass py-1 px-4 compact:flex-1 compact:basis-0 compact:flex-wrap compact:content-center compact:gap-x-1 compact:py-0.5 compact:px-2 ${packClass("Plate", "frame")}`}
          role="group"
          label={nameA()}
          data-local={props.localTeam == "a"}
        >
          <Text class={`${NAME} compact:basis-full compact:text-caption compact:text-end`}>{nameA()}</Text>
          <Show when={props.localTeam == "a"}>
            <Text class={YOU}>{t("arena.you")}</Text>
          </Show>
          <Text class={SCORE_A} description={t("arena.score.teamScore")}>
            {formatNumber(props.scoreA ?? 0)}
          </Text>
        </View>
        <View class={`shrink-0 ${ROUND_PLATE} compact:basis-full compact:flex-row compact:flex-wrap compact:min-w-0 compact:gap-y-2 compact:py-0.5 compact:px-2`}>
          <Text class="text-xs leading-snug font-semibold tracking-label uppercase text-fg-muted whitespace-nowrap compact:whitespace-normal compact:text-center">
            {(props.rounds ?? 0) > 0 ? t("arena.round.roundOf", { round: props.round ?? 0, rounds: props.rounds ?? 0 }) : t("arena.round.round", { round: props.round ?? 0 })}
          </Text>
          <Show when={(props.timeLeft ?? -1) >= 0}>
            <HudTimer seconds={Math.max(props.timeLeft ?? 0, 0)} total={props.timeTotal ?? 0} countdown={true} warnAt={10} paused={false} label="" />
          </Show>
        </View>
        <View
          class={`shrink basis-auto min-w-0 flex-row items-center justify-start gap-x-2 rounded-tr-card py-1 px-4 ${TEAM_PLATE} ${TEAM_B_PLATE} compact:flex-1 compact:basis-0 compact:flex-wrap compact:content-center compact:gap-x-1 compact:py-0.5 compact:px-2`}
          role="group"
          label={nameB()}
          data-local={props.localTeam == "b"}
        >
          <Text class={SCORE_B} description={t("arena.score.teamScore")}>
            {formatNumber(props.scoreB ?? 0)}
          </Text>
          <Show when={props.localTeam == "b"}>
            <Text class={YOU}>{t("arena.you")}</Text>
          </Show>
          <Text class={`${NAME} compact:basis-full compact:text-caption compact:text-start`}>{nameB()}</Text>
        </View>
      </View>
      <View class="w-full flex-row justify-center" role="status" live="assertive">
        <Show when={ended()}>
          <View class={`flex-col items-center gap-y-1 max-w-full py-3 px-8 bg-glass nine-panel-cut compact:py-2 compact:px-4 ${packClass("Panel", "frame")}`}>
            <Text
              class={`max-w-full text-center text-8xl leading-none font-bold uppercase tracking-xs text-gold-hi shadow-glow-lg shadow-gold data-lost:text-danger data-lost:shadow-glow-lg data-lost:shadow-critical`}
              data-lost={props.outcome == "lost"}
            >
              {props.outcome == "won" ? t("arena.round.won") : props.outcome == "lost" ? t("arena.round.lost") : t("arena.round.draw")}
            </Text>
            <Show when={(props.detail ?? "") != ""}>
              <Text class="max-w-full text-center text-base font-semibold tracking-label uppercase text-fg text-shadow-2xs">{props.detail}</Text>
            </Show>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ArenaRoundHudProps>> = {
  default: {"round":4,"rounds":7,"nameA":"","nameB":"","scoreA":2,"scoreB":1,"localTeam":"a","timeLeft":74,"timeTotal":120,"outcome":"none","at":0,"detail":""},
  won: {"round":5,"rounds":7,"nameA":"","nameB":"","scoreA":3,"scoreB":1,"localTeam":"a","timeLeft":-1,"timeTotal":0,"outcome":"won","at":1,"detail":"Match point"},
  lost: {"round":5,"rounds":7,"nameA":"","nameB":"","scoreA":2,"scoreB":3,"localTeam":"a","timeLeft":-1,"timeTotal":0,"outcome":"lost","at":2,"detail":""},
  draw: {"round":6,"rounds":0,"nameA":"","nameB":"","scoreA":3,"scoreB":3,"localTeam":"","timeLeft":0,"timeTotal":120,"outcome":"draw","at":3,"detail":"Time ran out"},
  critical: {"round":7,"rounds":7,"nameA":"","nameB":"","scoreA":3,"scoreB":3,"localTeam":"b","timeLeft":4.2,"timeTotal":120,"outcome":"none","at":0,"detail":""},
  overflow: {"round":128,"rounds":250,"nameA":"The Unbroken Wardens of the Ninefold Gate","nameB":"Ember Court Invitational Reserve","scoreA":1204,"scoreB":998,"localTeam":"a","timeLeft":3599,"timeTotal":3600,"outcome":"won","at":4,"detail":"Flawless victory in overtime after the defenders held both points to the final second"},
};
