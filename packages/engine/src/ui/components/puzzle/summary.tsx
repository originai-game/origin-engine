import { For, Icon, Scroll, Show, Text, type UiEvent, View, formatNumber, t } from "@origingame/engine/ui";
import "./puzzle.en.ogt";

export interface PuzzleSummaryProps {
  /** How the level ended: win or lose. */
  result: string;
  /** The level's name as the game words it (Level 12); empty hides it. */
  level: string;
  /** The final score, as the game counts it. */
  score: number;
  /** The score of the star bar's end (its last threshold). Default 1. */
  scoreMax?: number;
  /** The star bar's thresholds, keyed rows of { id, at }: at a fraction of scoreMax (0 to 1), as the game's star rules place them. */
  thresholds?: ReadonlyArray<{ id: string; at: number }>;
  /** Stars the game awarded (0 to maxStars): spoken and written beside the bar. */
  stars: number;
  /** Stars the level can award. Default 3. */
  maxStars?: number;
  /** The game says the score is a new best: a badge with its word. */
  record?: boolean;
  /** The score breakdown, keyed rows of { id, label, value }: value already worded by the game (+2,400, ×3). */
  rows?: ReadonlyArray<{ id: string; label: string; value: string }>;
  /** The breakdown's total as the game words it; empty hides the total row. */
  total: string;
  /** Next is offered (a won level with a next one). */
  canNext?: boolean;
  /** The menu event, with the row's key where it comes from a row. */
  onMenu?: (e: UiEvent, key?: string) => void;
  /** The next event, with the row's key where it comes from a row. */
  onNext?: (e: UiEvent, key?: string) => void;
  /** The retry event, with the row's key where it comes from a row. */
  onRetry?: (e: UiEvent, key?: string) => void;
}

/**
 * The end of a level: won or failed, the level's name, the stars on the
 * shared StarBar with the stars the game awarded in words, a new-best badge,
 * the score breakdown the game words, and the next steps. Reports next
 * (only when canNext), retry and menu (also Escape or pad B). The game
 * computes the score, the stars and what comes next.
 */
export function PuzzleSummary(props: PuzzleSummaryProps) {
  return (
    <View class={`pt-6 pb-4 border-rim-soft rounded-tl-sheet rounded-tr-none rounded-br-sheet rounded-bl-none bg-linear-to-b from-[#0f1522ed] to-[#080c16f2] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] transition duration-317 w-full max-w-115.5 px-6 gap-3 border compact:pt-4 compact:pb-3 compact:px-3 compact:gap-1 data-selected:border-accent data-down:opacity-45`} label={props.result === 'win' ? t("puzzle.summary.win") : t("puzzle.summary.lose")} onAction={{ back: (e) => props.onMenu?.(e) }} data-result-lose={(props.result) === 'lose'}>
      <View class={`group/band items-center p-0 gap-0.5`} data-result-lose={(props.result) === 'lose'}>
        <Text class={`font-display text-[2rem] font-bold leading-tight tracking-sm uppercase text-transparent text-center bg-linear-to-b from-accent-hover from-18% via-[#ffecaa] via-52% to-[#ffb547] m-0 group-data-result-lose/band:bg-linear-to-b group-data-result-lose/band:from-accent-hover group-data-result-lose/band:from-10% group-data-result-lose/band:via-[#ffa0a6] group-data-result-lose/band:via-48% group-data-result-lose/band:to-[#de2634] group-data-result-lose/band:bg-transparent`}>
          {props.result === 'win' ? t("puzzle.summary.win") : t("puzzle.summary.lose")}
        </Text>
        <Show when={props.level}>
          <Text class={`font-semibold text-[0.857rem] leading-4.5 text-accent-hover/78 self-stretch text-center line-clamp-2 tracking-[0.14em] uppercase m-0`}>
            {props.level}
          </Text>
        </Show>
      </View>
      <View class={`items-center gap-1`} data-result-lose={(props.result) === 'lose'}>
        <Show when={(props.thresholds ?? []).length}>
          <View class="items-end justify-center gap-2">
            <For each={props.thresholds}>
              {(th, i) => (
              <Icon class="text-material-edge size-[5.571rem] data-centre:mb-3 data-centre:size-[7.143rem] compact:size-[3.714rem] data-lit:text-gold compact:data-centre:mb-2 compact:data-centre:size-[4.857rem] shrink" name="star" data-lit={i() < props.stars} data-centre={i() === 1 && (props.thresholds ?? []).length === 3} />
              )}
            </For>
          </View>
        </Show>
        <View class={`items-center flex-col gap-1 w-full`} role="meter" label={t("puzzle.summary.stars", { stars: props.stars, max: (props.maxStars ?? 3) })} valueMin={0} valueMax={(props.scoreMax ?? 1)} valueNow={props.score} valueText={formatNumber(props.score)}>
          <Text class={`font-display text-[1.714rem] font-bold tabular-nums text-fg whitespace-nowrap`}>
            {formatNumber(props.score)}
          </Text>
          <View class="relative w-56 h-6">
            <View class="absolute inset-x-3 top-1/2 -translate-y-1/2 h-1.5 bg-tile rounded-capsule overflow-hidden">
              <View class="h-full bg-gold" style={{ width: `${Math.min(1, props.score / ((props.scoreMax ?? 1) || 1)) * 100}%` }} />
            </View>
            <For each={props.thresholds}>
              {(s) => (
                <View class="absolute top-0 translate-x-[-50%]" style={{ left: `${(s().at ?? 0) * 100}%` }}>
                  <Icon class={`size-6 text-fg-subtle data-lit:text-gold`} name="star" data-lit={props.score >= (s().at ?? 0) * (props.scoreMax ?? 1)} />
                </View>
              )}
            </For>
          </View>
        </View>
        <Text class={`font-semibold text-[0.857rem] leading-4.5 text-accent-hover/78 whitespace-nowrap tracking-[0.14em] uppercase`}>
          {t("puzzle.summary.stars", { stars: props.stars, max: (props.maxStars ?? 3) })}
        </Text>
        <Show when={props.record}>
          <Text class={`font-bold text-[0.857rem] leading-4.5 tabular-nums rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-linear-to-b from-[#fff2c4] to-gold text-[#1e1604] transition duration-317 whitespace-nowrap tracking-[0.14em] uppercase py-0 px-2`}>
            {t("puzzle.summary.record")}
          </Text>
        </Show>
      </View>
      <Show when={(props.rows ?? []).length}>
        <View class="grow shrink bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] gap-2">
          <Scroll class={`py-1 px-3 gap-2 `} role="table" label={t("puzzle.summary.breakdown")}>
            <For each={props.rows}>
              {(r) => (
              <View class={`flex-row items-baseline text-fg transition duration-317 py-1 px-0 gap-3 rounded-control compact:gap-1 hover:bg-transparent hover:bg-none data-selected:text-fg data-selected:shadow-[inset_3px_0px_0px_#e8e8ed] disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} role="row">
                <Text class={`grow shrink basis-0 text-start text-accent-hover`} role="rowheader">
                  {r().label}
                </Text>
                <Text class={`tabular-nums text-accent-hover text-end whitespace-nowrap text-[1.143rem] font-bold`} role="cell">
                  {r().value}
                </Text>
              </View>
              )}
            </For>
            <Show when={props.total}>
              <View class={`flex-row items-baseline text-fg transition duration-317 mt-1 border-t border-accent-hover/16 py-1 px-0 gap-3 rounded-control compact:gap-1 hover:bg-transparent hover:bg-none data-selected:text-fg data-selected:shadow-[inset_3px_0px_0px_#e8e8ed] disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} role="row">
                <Text class="grow shrink basis-0 text-start text-accent-hover font-bold" role="rowheader">
                  {t("puzzle.summary.total")}
                </Text>
                <Text class={`tabular-nums text-gold text-end whitespace-nowrap text-[1.714rem] font-bold`} role="cell">
                  {props.total}
                </Text>
              </View>
            </Show>
          </Scroll>
        </View>
      </Show>
      <View class={`flex-row flex-wrap items-center justify-center gap-1 compact:gap-0.5`}>
        <View class={`grow shrink basis-0 min-h-11`} role="button" label={t("puzzle.summary.menu")} onPress={(e) => props.onMenu?.(e)}>
          <Text>{t("puzzle.summary.menu")}</Text>
        </View>
        <View class={`grow shrink basis-0 min-h-11`} role="button" label={t("puzzle.summary.retry")} onPress={(e) => props.onRetry?.(e)} data-primary={!props.canNext}>
          <Text>{t("puzzle.summary.retry")}</Text>
        </View>
        <Show when={props.canNext}>
          <View class={`grow shrink basis-0 min-h-11 text-accent`} role="button" label={t("puzzle.summary.next")} onPress={(e) => props.onNext?.(e)}>
            <Text>{t("puzzle.summary.next")}</Text>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<PuzzleSummaryProps>> = {
  default: {"result":"win","level":"Level 12","score":24300,"scoreMax":30000,"thresholds":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"stars":2,"maxStars":3,"record":false,"rows":[{"id":"clear","label":"Goals cleared","value":"18,300"},{"id":"moves","label":"Moves left × 6","value":"+6,000"}],"total":"24,300","canNext":true},
  full: {"result":"win","level":"Level 12","score":31500,"scoreMax":30000,"thresholds":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"stars":3,"maxStars":3,"record":true,"rows":[{"id":"clear","label":"Goals cleared","value":"21,000"},{"id":"moves","label":"Moves left × 7","value":"+7,000"},{"id":"combo","label":"Best combo","value":"+3,500"}],"total":"31,500","canNext":true},
  lose: {"result":"lose","level":"Level 12","score":6400,"scoreMax":30000,"thresholds":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"stars":0,"maxStars":3,"record":false,"rows":[{"id":"clear","label":"Goals cleared","value":"6,400"}],"total":"","canNext":false},
  empty: {"result":"win","level":"","score":0,"scoreMax":30000,"thresholds":[],"stars":0,"maxStars":3,"record":false,"rows":[],"total":"","canNext":false},
  overflow: {"result":"win","level":"Level 1016: The Very Long Caramel Causeway Beyond the Whispering Lemonade Falls","score":123456789,"scoreMax":150000000,"thresholds":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"stars":2,"maxStars":3,"record":true,"rows":[{"id":"a","label":"Goals cleared across every layer of frosting and caramel","value":"98,000,000"},{"id":"b","label":"Moves left × 12","value":"+12,000,000"},{"id":"c","label":"Best combo","value":"+8,456,789"},{"id":"d","label":"Special sweets","value":"+3,000,000"},{"id":"e","label":"Jelly bonus","value":"+1,500,000"},{"id":"f","label":"Daily streak","value":"+500,000"}],"total":"123,456,789","canNext":true},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"back":"Map"} as const;
