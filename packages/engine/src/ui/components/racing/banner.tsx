import { Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
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
        <Text class={`min-w-0 text-title leading-tight font-bold tabular-nums tracking-kicker uppercase whitespace-nowrap overflow-hidden text-fg text-stroke-3 text-stroke-bg compact:text-base group-data-kind-countdown/plate:text-display-md group-data-kind-countdown/plate:tracking-normal`}>
          {kind() === "countdown" ? formatNumber(props.figure) : kind() === "go" ? t("racing.banner.go") : kind() === "finalLap" ? t("racing.banner.finalLap") : kind() === "finish" ? t("racing.banner.finish") : props.text}
        </Text>
        <Show when={kind() === "finish" && props.place > 0}>
          <Text class={`shrink-0 text-title font-bold tabular-nums whitespace-nowrap text-hud-accent-ink compact:text-base`}>{ordinal(props.place)}</Text>
        </Show>
      </View>
      <Show when={props.detail !== ""}>
        <Text class={`max-w-full min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap overflow-hidden text-fg-muted text-stroke-3 text-stroke-bg compact:tracking-normal`}>{props.detail}</Text>
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
                  class={`group/plate flex-col items-center max-w-full min-w-0 gap-y-0.5 py-1.5 px-4.5 bg-plate-hi nine-plate-cut transition duration-base ease-out starting:opacity-0 starting:translate-y-2 compact:py-1 compact:px-3.5 ${packClass("Plate", "frame")}`}
                  data-kind-countdown={false}
                >
                  {plateInner()}
                </View>
              }
            >
              <View
                class={`group/plate flex-col items-center max-w-full min-w-0 gap-y-0.5 py-1.5 px-4.5 bg-plate-hi nine-plate-cut animate-stamp compact:py-1 compact:px-3.5 ${packClass("Plate", "frame")}`}
                data-kind-countdown={true}
                sound="countdown"
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
  overflow: { "kind": "finish", "figure": 0, "text": "", "at": 7, "place": 23, "detail": "Disqualified for cutting the chicane at the Esses on the final lap" },
};
