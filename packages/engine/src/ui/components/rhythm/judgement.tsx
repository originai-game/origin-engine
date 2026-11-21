import { Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmJudgementProps {
  /** The last hit's judgement as the game judged it: perfect, great, good, bad, miss, or none (nothing shown); on data-judgement for the pack, and always a word. Default "none". */
  judgement?: string;
  /** The judgement's word as the game names it (Critical, Marvelous); empty takes the kit's word for judgement. */
  label: string;
  /** early or late under the word (Early, Late); empty hides it. */
  timing: string;
  /** The hit's offset as the game words it (+4 ms, −12 ms), beside the timing; empty hides it. */
  offset: string;
  /** The hit's id: a new value draws the word afresh (each note judged), so the word and the combo pop again. */
  hit: number;
  /** The running combo as the game counts it; 0 hides it. */
  combo: number;
  /** The combo just broke: Combo broken shows in its place. */
  broken: boolean;
  /** The run is still on course for a mark: fullCombo (Full combo), allPerfect (All perfect), or empty for none; on data-mark. */
  mark: string;
}

/**
 * The play field's centre: the running combo as the hero figure with its
 * word (Combo broken when it breaks, Full combo or All perfect while the
 * run holds one), and under it the last hit's judgement, each in its own
 * lit colour, with Early or Late and the offset. A new hit id pops the word
 * and the count again. The game owns timing windows, judging and counting;
 * this only shows what it was told. Read only; not a live region.
 */
export function RhythmJudgement(props: RhythmJudgementProps) {
  const judgement = props.judgement ?? "none";
  return (
    <View
      class={`flex-col items-center w-full`}
      role="group"
      label={t("rhythm.judge.name")}
      data-judgement-perfect={judgement === "perfect"}
      data-judgement-great={judgement === "great"}
      data-judgement-good={judgement === "good"}
      data-judgement-bad={judgement === "bad"}
      data-judgement-miss={judgement === "miss"}
      data-mark-fullcombo={props.mark === "fullCombo"}
      data-mark-allperfect={props.mark === "allPerfect"}
    >
      <Show when={props.combo > 0 || props.broken}>
        <View class={`flex-col items-center max-w-full`} data-broken={props.broken}>
          <Show when={!props.broken}>
            <Show when={Math.max(props.hit, 1)} keyed>
              {() => (
              <Text class={`text-[10.714rem] compact:text-7xl leading-[0.85] font-bold tabular-nums italic whitespace-nowrap text-fg text-stroke-3 text-stroke-bg animate-beat`}>
                {formatNumber(props.combo)}
              </Text>
              )}
            </Show>
          </Show>
          <View class={`flex-col items-center gap-0.5 compact:flex-row compact:items-baseline compact:gap-2`}>
            <Text
              class={`text-base font-semibold uppercase whitespace-nowrap text-hud-accent tracking-[0.4em] compact:tracking-label data-broken:text-fg-muted data-broken:tracking-label`}
              data-broken={props.broken}
            >
              {props.broken ? t("rhythm.judge.broken") : t("rhythm.judge.combo")}
            </Text>
            <Show when={!props.broken && props.mark !== ""}>
              <Text
                class={`mt-1 compact:mt-0 py-0 px-2 text-xs font-semibold tracking-label uppercase whitespace-nowrap text-gold bg-well nine-chip-cut ${packClass("Chip", "frame")}`}
                data-mark-fullcombo={props.mark === "fullCombo"}
                data-mark-allperfect={props.mark === "allPerfect"}
              >
                {props.mark === "allPerfect" ? t("rhythm.judge.allPerfect") : t("rhythm.judge.fullCombo")}
              </Text>
            </Show>
          </View>
        </View>
      </Show>
      <Show when={judgement !== "none" ? Math.max(props.hit, 1) : 0} keyed>
        {() => (
        <View
          class={`flex-col items-center max-w-full mt-6 compact:mt-3 animate-stamp`}
          data-judgement-perfect={judgement === "perfect"}
          data-judgement-great={judgement === "great"}
          data-judgement-good={judgement === "good"}
          data-judgement-bad={judgement === "bad"}
          data-judgement-miss={judgement === "miss"}
        >
          <Text
            class={`max-w-full min-w-0 text-8xl compact:text-title leading-[1.05] font-bold italic tracking-wide uppercase text-center whitespace-nowrap text-ellipsis overflow-hidden text-gold data-judgement-great:text-hud-accent data-judgement-good:text-boon data-judgement-bad:text-warn data-judgement-miss:text-fg-muted`}
            data-judgement-perfect={judgement === "perfect"}
            data-judgement-great={judgement === "great"}
            data-judgement-good={judgement === "good"}
            data-judgement-bad={judgement === "bad"}
            data-judgement-miss={judgement === "miss"}
          >
            {props.label !== "" ? props.label : (judgement === "perfect" ? t("rhythm.judge.perfect") : (judgement === "great" ? t("rhythm.judge.great") : (judgement === "good" ? t("rhythm.judge.good") : (judgement === "bad" ? t("rhythm.judge.bad") : t("rhythm.judge.miss")))))}
          </Text>
          <Show when={props.timing !== "" || props.offset !== ""}>
            <View class={`flex-row items-baseline justify-center gap-1`}>
              <Show when={props.timing !== ""}>
                <Text
                  class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg data-timing-early:text-hud-accent data-timing-late:text-late`}
                  data-timing-early={props.timing === "early"}
                  data-timing-late={props.timing === "late"}
                >
                  {props.timing === "early" ? t("rhythm.judge.early") : t("rhythm.judge.late")}
                </Text>
              </Show>
              <Show when={props.offset !== ""}>
                <Text
                  class={`text-base font-bold tabular-nums whitespace-nowrap text-gold data-timing-early:text-hud-accent data-timing-late:text-late`}
                  data-timing-early={props.timing === "early"}
                  data-timing-late={props.timing === "late"}
                >
                  {props.offset}
                </Text>
              </Show>
            </View>
          </Show>
        </View>
        )}
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RhythmJudgementProps>> = {
  default: {"judgement":"perfect","label":"","timing":"late","offset":"+4 ms","hit":1,"combo":412,"broken":false,"mark":"fullCombo"},
  great: {"judgement":"great","label":"","timing":"early","offset":"−21 ms","hit":2,"combo":129,"broken":false,"mark":""},
  good: {"judgement":"good","label":"","timing":"late","offset":"+48 ms","hit":3,"combo":130,"broken":false,"mark":""},
  miss: {"judgement":"miss","label":"","timing":"","offset":"","hit":4,"combo":0,"broken":true,"mark":""},
  overflow: {"judgement":"perfect","label":"Transcendent Critical Perfect","timing":"late","offset":"+1 ms","hit":5,"combo":12345,"broken":false,"mark":"allPerfect"},
};
