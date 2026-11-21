import { For, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmTimingProps {
  /** The judgement tally so far in the game's order, keyed rows of { id, judgement, label, count }: judgement the kind (perfect, great, good, bad, miss; its colour, on data-judgement), label its word as the game names it, count how many. */
  counts: ReadonlyArray<{ id: string; judgement: string; label: string; count: number }>;
  /** The latest hits on the timing scale, keyed rows of { id, judgement, offset, age }: offset from −1 (earliest the game judges) through 0 (on the beat) to 1 (latest); age from 0 (this hit) to 1 (about to drop), which fades it. The game keeps the list short (eight or so). */
  hits: ReadonlyArray<{ id: string; judgement: string; offset: number; age: number }>;
}

/**
 * How the run is being judged, at a corner of the field: the tally so far,
 * each judgement in its colour, and under it the timing scale from Early
 * to Late with the beat at its centre and the latest hits as ticks, the
 * newest brightest. The game judges and counts; read only.
 */
export function RhythmTiming(props: RhythmTimingProps) {
  return (
    <View
      class={`flex-col gap-1 w-49 max-w-full py-2 px-3 bg-plate-hi nine-plate-cut text-fg ${packClass("Plate", "frame")}`}
      role="group"
      label={t("rhythm.timing.name")}
    >
      <Show when={(props.counts ?? []).length}>
        <View role="list" label={t("rhythm.results.counts")} class={`flex-col gap-0 compact:flex-row compact:flex-wrap compact:gap-x-3`}>
          <For each={props.counts ?? []} key={(c) => c.id}>
            {(c) => (
              <View
                role="listitem"
                label={c().label}
                class={`flex-row items-baseline gap-4 compact:gap-1 text-fg data-judgement-perfect:text-gold data-judgement-great:text-hud-accent data-judgement-good:text-boon data-judgement-bad:text-warn data-judgement-miss:text-fg-muted`}
                data-judgement-perfect={c().judgement === "perfect"}
                data-judgement-great={c().judgement === "great"}
                data-judgement-good={c().judgement === "good"}
                data-judgement-bad={c().judgement === "bad"}
                data-judgement-miss={c().judgement === "miss"}
              >
                <Text class={`min-w-0 flex-1 text-base compact:text-caption font-bold whitespace-nowrap text-ellipsis overflow-hidden`}>{c().label}</Text>
                <Text class={`shrink text-base compact:text-caption font-bold tabular-nums whitespace-nowrap`}>{formatNumber(c().count)}</Text>
              </View>
            )}
          </For>
        </View>
      </Show>
      <View class={`flex-col gap-0.5`}>
        <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted compact:hidden`}>
          {t("rhythm.timing.label")}
        </Text>
        <View class={`flex flex-row flex-wrap justify-between items-center gap-y-0.5 gap-x-1 compact:flex-nowrap`}>
          <View class={`relative h-1.5 flex-1 basis-full compact:basis-0 compact:min-w-0 bg-well ring ring-rim`}>
            <View class={`absolute left-1/2 -translate-x-1/2 top-0 -mt-1 w-0.5 h-3.5 bg-sheen`} />
            <For each={props.hits ?? []} key={(h) => h.id}>
              {(h) => (
                <View
                  class={`absolute top-0 -translate-x-1/2 w-0.75 h-3 -mt-0.75 bg-gold data-judgement-great:bg-hud-accent data-judgement-good:bg-boon data-judgement-bad:bg-warn data-judgement-miss:bg-fg-muted`}
                  style={{
                    left: `${(h().offset <= -1 ? 0 : h().offset >= 1 ? 1 : (h().offset + 1) / 2) * 100}%`,
                    opacity: 1 - (h().age <= 0 ? 0 : h().age >= 1 ? 1 : h().age) * 0.8,
                  }}
                  data-judgement-perfect={h().judgement === "perfect"}
                  data-judgement-great={h().judgement === "great"}
                  data-judgement-good={h().judgement === "good"}
                  data-judgement-bad={h().judgement === "bad"}
                  data-judgement-miss={h().judgement === "miss"}
                />
              )}
            </For>
          </View>
          <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted`}>
            {t("rhythm.judge.early")}
          </Text>
          <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted`}>
            {t("rhythm.judge.late")}
          </Text>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RhythmTimingProps>> = {
  default: {"counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":380},{"id":"great","judgement":"great","label":"Great","count":30},{"id":"good","judgement":"good","label":"Good","count":2},{"id":"miss","judgement":"miss","label":"Miss","count":0}],"hits":[{"id":"h4","judgement":"perfect","offset":0.12,"age":0},{"id":"h3","judgement":"perfect","offset":0.04,"age":0.3},{"id":"h2","judgement":"great","offset":-0.24,"age":0.6},{"id":"h1","judgement":"perfect","offset":0.08,"age":0.85}]},
  early: {"counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":12},{"id":"great","judgement":"great","label":"Great","count":9},{"id":"good","judgement":"good","label":"Good","count":5},{"id":"bad","judgement":"bad","label":"Bad","count":3},{"id":"miss","judgement":"miss","label":"Miss","count":4}],"hits":[{"id":"e3","judgement":"good","offset":-0.7,"age":0},{"id":"e2","judgement":"bad","offset":-0.92,"age":0.4},{"id":"e1","judgement":"great","offset":-0.4,"age":0.7}]},
  empty: {"counts":[],"hits":[]},
  overflow: {"counts":[{"id":"critical","judgement":"perfect","label":"Transcendent Critical Perfect","count":98765},{"id":"perfect","judgement":"perfect","label":"Perfect","count":12345},{"id":"great","judgement":"great","label":"Great","count":0},{"id":"good","judgement":"good","label":"Good","count":0},{"id":"bad","judgement":"bad","label":"Bad","count":0},{"id":"miss","judgement":"miss","label":"Miss","count":0}],"hits":[{"id":"o1","judgement":"miss","offset":1.6,"age":0}]},
};
