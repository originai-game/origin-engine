import { For, Scroll, Show, Text, View, formatNumber, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmResultsProps {
  /** The song's title as the heading; empty takes the kit's (Results). */
  song: string;
  /** The grade the game awarded (S, AA, C); empty hides it. */
  grade: string;
  /** The game's verdict: Cleared, or Failed (data-cleared). */
  cleared: boolean;
  /** The run's score as the game counts it, shown with grouping. */
  score: number;
  /** The player's best before this run (Best …); 0 hides it. */
  previous: number;
  /** The game says this run beat the best: New best shows by the score. */
  newBest: boolean;
  /** The longest combo of the run. */
  maxCombo: number;
  /** The accuracy as the game words it (97.42%); empty hides it. */
  accuracy: string;
  /** A run mark the game awarded: fullCombo (Full combo), allPerfect (All perfect), or empty for none; on data-mark. */
  mark: string;
  /** The judgement tally in the game's order, keyed rows of { id, judgement, label, count }: judgement the kind (perfect, great, good, bad, miss; on data-judgement), label its word as the game names it, count how many. */
  counts: ReadonlyArray<{ id: string; judgement: string; label: string; count: number }>;
  /** Retry is offered (false: disabled). */
  canRetry: boolean;
  /** Retry; silent when the game refuses it. */
  onRetry?: (e: UiEvent) => void;
  /** Next; also Escape and the back action (pad B). */
  onNext?: (e: UiEvent) => void;
}

/**
 * After the song: the grade, Cleared or Failed, the score with the old best
 * and New best, Full combo or All perfect, max combo and accuracy, and the
 * judgement tally. Retry (disabled when the game refuses it) and Next
 * (also Escape, pad B) report once. The game computes every figure, grade
 * and verdict.
 */
export function RhythmResults(props: RhythmResultsProps) {
  return (
    <View
      class={`flex-col gap-4 w-full h-full min-w-0 p-4 text-fg bg-glass nine-panel-cut ${packClass("Panel", "frame")}`}
      label={props.song !== "" ? props.song : t("rhythm.results.title")}
      data-mark-fullcombo={props.mark === "fullCombo"}
      data-mark-allperfect={props.mark === "allPerfect"}
      data-cleared={props.cleared}
      data-not-cleared={!props.cleared}
    >
      <View class={`flex-row shrink w-full min-h-0`}>
        <Text class={`text-display-xl compact:text-display-sm leading-flat font-bold min-w-0 whitespace-nowrap text-ellipsis overflow-hidden`}>
          {props.song !== "" ? props.song : t("rhythm.results.title")}
        </Text>
      </View>
      <Scroll class={`grow min-h-0`}>
        <View class={`flex-col gap-4 p-1 w-full`}>
          <View class={`flex flex-row flex-wrap justify-between items-end gap-4 w-full`}>
            <View class={`flex-row items-center gap-4 shrink`}>
              <Show when={props.grade !== ""}>
                <Text
                  class={`shrink min-w-14 pr-2 text-8xl compact:text-display-xl leading-flat font-bold italic text-center whitespace-nowrap text-gold data-not-cleared:text-foe`}
                  label={t("rhythm.results.gradeName", { grade: props.grade })}
                  data-not-cleared={!props.cleared}
                >
                  {props.grade}
                </Text>
              </Show>
              <View class={`flex-col items-start gap-0.5 min-w-0`}>
                <Text
                  class={`whitespace-nowrap py-0 px-2 text-base font-bold tracking-label uppercase text-gold-fg bg-hud-accent nine-chip-cut data-tag-failed:text-fg data-tag-failed:bg-critical ${packClass("Chip", "frame")}`}
                  data-tag-cleared={props.cleared}
                  data-tag-failed={!props.cleared}
                >
                  {props.cleared ? t("rhythm.results.cleared") : t("rhythm.results.failed")}
                </Text>
                <Show when={props.mark !== ""}>
                  <Text
                    class={`whitespace-nowrap py-0 px-2 text-base font-bold tracking-label uppercase text-gold-fg bg-gold nine-chip-cut`}
                    data-tag-fullcombo={props.mark === "fullCombo"}
                    data-tag-allperfect={props.mark === "allPerfect"}
                  >
                    {props.mark === "allPerfect" ? t("rhythm.results.allPerfect") : t("rhythm.results.fullCombo")}
                  </Text>
                </Show>
              </View>
            </View>
            <View role="group" label={t("rhythm.results.score")} class={`grow min-w-0 max-w-full flex-col items-start bg-plate-hi nine-plate-cut py-3 px-4 text-fg ${packClass("Plate", "frame")}`}>
              <Text class={`text-xs font-semibold tracking-label uppercase opacity-78`}>{t("rhythm.results.score")}</Text>
              <Text class={`text-8xl compact:text-4xl leading-none font-bold tabular-nums whitespace-nowrap pb-1`}>{formatNumber(props.score)}</Text>
              <View class={`flex flex-row flex-wrap items-center gap-1`}>
                <Show when={props.newBest}>
                  <Text
                    class={`whitespace-nowrap py-0 px-1 text-xs font-bold tracking-label uppercase text-gold-fg bg-gold nine-chip-cut ${packClass("Chip", "frame")}`}
                    data-tag-newbest={true}
                  >
                    {t("rhythm.results.newBest")}
                  </Text>
                </Show>
                <Show when={props.previous > 0}>
                  <Text class={`whitespace-nowrap text-xs font-semibold tracking-normal normal-case opacity-78`}>
                    {t("rhythm.results.previous", { score: props.previous })}
                  </Text>
                </Show>
              </View>
            </View>
          </View>
          <View class={`flex-row items-start gap-4 w-full compact:flex-col`}>
            <View class={`flex flex-row flex-wrap gap-2 min-w-0 grow basis-0`}>
              <View role="group" label={t("rhythm.results.maxCombo")} class={`flex-col justify-between gap-0.5 min-w-0 grow basis-37.5 bg-plate-hi nine-plate-cut py-2 px-3 text-fg ${packClass("Plate", "frame")}`}>
                <Text class={`text-xs font-semibold tracking-label uppercase opacity-78`}>{t("rhythm.results.maxCombo")}</Text>
                <Text class={`shrink whitespace-nowrap text-display-md compact:text-title font-bold tabular-nums`}>{formatNumber(props.maxCombo)}</Text>
              </View>
              <Show when={props.accuracy !== ""}>
                <View role="group" label={t("rhythm.results.accuracy")} class={`flex-col justify-between gap-0.5 min-w-0 grow basis-37.5 bg-plate-hi nine-plate-cut py-2 px-3 text-fg ${packClass("Plate", "frame")}`}>
                  <Text class={`text-xs font-semibold tracking-label uppercase opacity-78`}>{t("rhythm.results.accuracy")}</Text>
                  <Text class={`shrink whitespace-nowrap text-display-md compact:text-title font-bold tabular-nums`}>{props.accuracy}</Text>
                </View>
              </Show>
            </View>
            <Show when={(props.counts ?? []).length} fallback={
              <View role="status" class={`min-w-0 grow basis-0 p-3`}>
                <Text class={`text-center text-xs font-semibold tracking-label uppercase opacity-78`}>{t("rhythm.results.empty")}</Text>
              </View>
            }>
              <View role="list" label={t("rhythm.results.counts")} class={`flex-col gap-0.5 min-w-0 grow basis-0 bg-plate-hi nine-plate-cut py-2 px-3 text-fg ${packClass("Plate", "frame")}`}>
                <For each={props.counts ?? []} key={(c) => c.id}>
                  {(c) => (
                    <View
                      role="listitem"
                      label={c().label}
                      class={`flex-row items-baseline gap-3 py-0.5 w-full data-judgement-perfect:text-gold data-judgement-great:text-hud-accent data-judgement-good:text-boon data-judgement-bad:text-warn data-judgement-miss:opacity-80`}
                      data-judgement-perfect={c().judgement === "perfect"}
                      data-judgement-great={c().judgement === "great"}
                      data-judgement-good={c().judgement === "good"}
                      data-judgement-bad={c().judgement === "bad"}
                      data-judgement-miss={c().judgement === "miss"}
                    >
                      <Text class={`flex-1 min-w-0 text-base font-bold italic tracking-wide uppercase`}>{c().label}</Text>
                      <Text class={`shrink whitespace-nowrap text-title font-bold tabular-nums`}>{formatNumber(c().count)}</Text>
                    </View>
                  )}
                </For>
              </View>
            </Show>
          </View>
        </View>
      </Scroll>
      <View class={`flex-row justify-end gap-2 shrink w-full`}>
        <View
          role="button"
          label={t("rhythm.results.retry")}
          disabled={!props.canRetry}
          onPress={(e) => props.onRetry?.(e)}
          class={`min-h-11 min-w-30 compact:flex-1 compact:min-w-0 flex-row items-center justify-center gap-1 px-2 rounded-control border border-rim text-fg hover:bg-raised-hi disabled:opacity-45 ${packClass("Button", "plate")}`}
        >
          <Text class={`text-caption font-bold tracking-label uppercase`}>{t("rhythm.results.retry")}</Text>
        </View>
        <View
          role="button"
          label={t("rhythm.results.next")}
          onPress={(e) => props.onNext?.(e)}
          onKey={(e) => { if (e.key === "Escape") props.onNext?.(e); }}
          onAction={{ back: (e) => props.onNext?.(e) }}
          class={`min-h-11 min-w-30 compact:flex-1 compact:min-w-0 flex-row items-center justify-center gap-1 px-3 rounded-control bg-hud-accent text-gold-fg inset-shadow-lit ${packClass("Button", "plate", "primary")}`}
        >
          <Text class={`shrink-0 pointer-coarse:hidden text-title leading-none`} glyph="back" />
          <Text class={`text-caption font-bold tracking-label uppercase`}>{t("rhythm.results.next")}</Text>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RhythmResultsProps>> = {
  default: {"song":"Skyline Drive","grade":"S","cleared":true,"score":962410,"previous":948220,"newBest":true,"maxCombo":412,"accuracy":"97.42%","mark":"","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":388},{"id":"great","judgement":"great","label":"Great","count":21},{"id":"good","judgement":"good","label":"Good","count":3},{"id":"miss","judgement":"miss","label":"Miss","count":2}],"canRetry":true},
  failed: {"song":"Static Bloom","grade":"D","cleared":false,"score":198400,"previous":0,"newBest":false,"maxCombo":57,"accuracy":"76.30%","mark":"","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":120},{"id":"great","judgement":"great","label":"Great","count":64},{"id":"good","judgement":"good","label":"Good","count":31},{"id":"miss","judgement":"miss","label":"Miss","count":88}],"canRetry":true},
  fullCombo: {"song":"Ember Waltz","grade":"SS","cleared":true,"score":991870,"previous":995120,"newBest":false,"maxCombo":356,"accuracy":"99.12%","mark":"fullCombo","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":340},{"id":"great","judgement":"great","label":"Great","count":16},{"id":"good","judgement":"good","label":"Good","count":0},{"id":"miss","judgement":"miss","label":"Miss","count":0}],"canRetry":true},
  allPerfect: {"song":"Ember Waltz","grade":"SSS","cleared":true,"score":1000000,"previous":991870,"newBest":true,"maxCombo":356,"accuracy":"100.00%","mark":"allPerfect","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":356},{"id":"miss","judgement":"miss","label":"Miss","count":0}],"canRetry":true},
  disabled: {"song":"Vault of Echoes","grade":"A","cleared":true,"score":901330,"previous":0,"newBest":false,"maxCombo":201,"accuracy":"93.80%","mark":"","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":240},{"id":"miss","judgement":"miss","label":"Miss","count":9}],"canRetry":false},
  empty: {"song":"","grade":"","cleared":false,"score":0,"previous":0,"newBest":false,"maxCombo":0,"accuracy":"","mark":"","counts":[],"canRetry":true},
  overflow: {"song":"The Unbearably Long Extended Twelve-Minute Midnight Orchestral Remix of a Song","grade":"SSS+","cleared":true,"score":123456789,"previous":123450000,"newBest":true,"maxCombo":98765,"accuracy":"99.9999%","mark":"allPerfect","counts":[{"id":"critical","judgement":"perfect","label":"Transcendent Critical Perfect","count":98765},{"id":"perfect","judgement":"perfect","label":"Perfect","count":0},{"id":"great","judgement":"great","label":"Great","count":0},{"id":"good","judgement":"good","label":"Good","count":0},{"id":"bad","judgement":"bad","label":"Bad","count":0},{"id":"miss","judgement":"miss","label":"Miss","count":0},{"id":"early","judgement":"great","label":"Early","count":0},{"id":"late","judgement":"great","label":"Late","count":0}],"canRetry":true},
};
