import { Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./racing.en.ogt";

export interface RacingBannerProps {
  /** What the banner calls: countdown (the count), go, finalLap, finish, call (the game's own words, in text), or none (nothing shown, as for any other word). Default "none". */
  kind?: string;
  /** The countdown's figure (3, 2, 1); the game steps it. */
  figure: number;
  /** The call's words as the game words them for kind call (Final lap next, Sector best); empty shows nothing. */
  text: string;
  /** The call's id: a new value replays the entrance (each count, the go, the flag). */
  at: number;
  /** On the finish banner, the player's place as an ordinal (1st); 0 hides it. */
  place: number;
  /** A line under the call as the game words it (New lap record, Lap 3 of 3); empty hides it. */
  detail: string;
}

/** English ordinal (1st, 2nd, 23rd): formatOrdinal has no TSX surface. */
function ordinal(n: number): string {
  const s = ["th", "st", "nd", "rd"];
  const v = n % 100;
  return `${n}${s[(v - 20) % 10] || s[v] || s[0]}`;
}

/**
 * The race's calls at the top centre: a plate of smoked glass with a lit
 * top edge in the HUD's accent, the call in widely set capitals (the
 * count, GO, the final lap, the finish with the player's place, or the
 * game's own words) and a line under it. The game steps the call and its
 * id (at); a new id plays the entrance again. Announced as a status.
 */
export function RacingBanner(props: RacingBannerProps) {
  const kind = () => props.kind ?? "none";
  // The plate's content, shared by the two entrances: the stamp for a
  // countdown, the rise for every other call.
  const plateInner = () => (
    <>
      <View class="flex-row items-baseline justify-center gap-x-2 max-w-full min-w-0">
        <Text class={`min-w-0 text-[1.714rem] leading-[1.1] font-bold tabular-nums tracking-[0.22em] uppercase whitespace-nowrap overflow-hidden text-fg text-shadow-xs text-shadow-bg/55 compact:text-[1.143rem] group-data-kind-countdown/plate:text-[2.429rem] group-data-kind-countdown/plate:tracking-normal`}>
          {kind() === "countdown" ? formatNumber(props.figure) : kind() === "go" ? t("racing.banner.go") : kind() === "finalLap" ? t("racing.banner.finalLap") : kind() === "finish" ? t("racing.banner.finish") : props.text}
        </Text>
        <Show when={kind() === "finish" && props.place > 0}>
          <Text class={`shrink-0 text-[1.714rem] font-bold tabular-nums whitespace-nowrap text-[#38e8ff] compact:text-[1.143rem]`}>{ordinal(props.place)}</Text>
        </Show>
      </View>
      <Show when={props.detail !== ""}>
        <Text class={`max-w-full min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-[#ffffffc7] text-shadow-xs text-shadow-bg/55 compact:tracking-normal`}>{props.detail}</Text>
      </Show>
    </>
  );
  return (
    <View class={`pointer-events-none w-full max-w-full text-fg`} role="status" label={t("racing.banner.name")} data-kind-countdown={kind() === "countdown"}>
      <Show when={kind() === "countdown" || kind() === "go" || kind() === "finalLap" || kind() === "finish" || (kind() === "call" && props.text !== "")}>
        <Show when={props.at + 1} keyed>
          {() => (
            <Show
              when={kind() === "countdown"}
              fallback={
                <View
                  class={`group/plate flex-col items-center max-w-full min-w-0 gap-y-[0.143rem] py-[0.429rem] px-[1.286rem] bg-linear-to-b from-[#1e2637e5] to-plate-lo rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none shadow-[inset_0px_0px_0px_1px_#38e8ff73,inset_0px_2.288px_0px_#38e8ff] transition duration-240 ease-out starting:opacity-0 starting:translate-y-2 compact:py-[0.286rem] compact:px-[1rem]`}
                  data-kind-countdown={false}
                >
                  {plateInner()}
                </View>
              }
            >
              <View
                class={`group/plate flex-col items-center max-w-full min-w-0 gap-y-[0.143rem] py-[0.429rem] px-[1.286rem] bg-linear-to-b from-[#1e2637e5] to-plate-lo rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none shadow-[inset_0px_0px_0px_1px_#38e8ff73,inset_0px_2.288px_0px_#38e8ff] animate-stamp compact:py-[0.286rem] compact:px-[1rem]`}
                data-kind-countdown={true}
              >
                {plateInner()}
              </View>
            </Show>
          )}
        </Show>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingBannerProps>> = {
  default: { "kind": "call", "figure": 0, "text": "Final lap next", "at": 1, "place": 0, "detail": "" },
  countdown: { "kind": "countdown", "figure": 3, "text": "", "at": 2, "place": 0, "detail": "" },
  go: { "kind": "go", "figure": 0, "text": "", "at": 4, "place": 0, "detail": "" },
  finalLap: { "kind": "finalLap", "figure": 0, "text": "", "at": 5, "place": 0, "detail": "Lap 3 of 3" },
  finish: { "kind": "finish", "figure": 0, "text": "", "at": 6, "place": 2, "detail": "New lap record" },
  none: { "kind": "none", "figure": 0, "text": "", "at": 0, "place": 0, "detail": "" },
  overflow: { "kind": "finish", "figure": 0, "text": "", "at": 7, "place": 23, "detail": "Disqualified for cutting the chicane at the Esses on the final lap" },
};
