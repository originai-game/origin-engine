import { Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./shooter.en.ogt";

export interface ShooterRoundProps {
  /** The round in play; 0 hides the word. */
  round?: number;
  /** The round's time left as the game words it (1:42); empty hides it. */
  clock?: string;
  /** The player's side's score. */
  ours?: number;
  /** The other side's score; negative hides the score. */
  theirs?: number;
  /** The game says time is short: the plate's lit edge and the clock turn to danger. */
  urgent?: boolean;
}

/**
 * The round on a plate lit along its top, its time left beside it, and the
 * score with the player's side in its blue against the other's red. The
 * game owns the clock and the score; read only. The plate wears the
 * pack's cut and keeps no rim or urgent glow (a cut takes one fill).
 */
export function ShooterRound(props: ShooterRoundProps) {
  const score = () => (props.theirs ?? -1) >= 0;
  return (
    <View
      class={`group/round flex-row items-center gap-x-3 max-w-full text-fg compact:flex-wrap compact:justify-start compact:gap-y-[0.143rem]`}
      role="group"
      label={score() ? t("shooter.round.score", { ours: props.ours ?? 0, theirs: props.theirs ?? 0 }) : t("shooter.round.name")}
      data-urgent={props.urgent}
    >
      <Show when={(props.round ?? 0) > 0 || (props.clock ?? "") != ""}>
        <View
          class={`shrink basis-auto min-w-0 flex-row items-baseline gap-1 py-[0.214rem] px-2 nine-plate-cut bg-plate-hi compact:flex-wrap ${packClass("Plate", "frame")}`}
        >
          <Show when={(props.round ?? 0) > 0}>
            <Text class="min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap truncate text-fg/78">
              {t("shooter.round.label", { round: props.round ?? 0 })}
            </Text>
          </Show>
          <Show when={(props.clock ?? "") != ""}>
            <Text
              class={`shrink-0 text-[1.143rem] leading-[1.1] font-bold tabular-nums whitespace-nowrap text-fg data-urgent:text-danger compact:text-[0.929rem]`}
              role="timer"
              data-urgent={props.urgent}
            >
              {props.clock}
            </Text>
          </Show>
        </View>
      </Show>
      <View class="min-w-0 flex-1 compact:hidden" />
      <Show when={score()}>
        <View class="shrink-0 flex-row items-baseline gap-1">
          <Text class="text-[1.714rem] leading-none font-bold tabular-nums whitespace-nowrap text-ally text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-[1.143rem]">
            {formatNumber(props.ours ?? 0)}
          </Text>
          <View class="self-center w-[0.714rem] h-[0.143rem] bg-white/66" />
          <Text class="text-[1.714rem] leading-none font-bold tabular-nums whitespace-nowrap text-danger text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-[1.143rem]">
            {formatNumber(props.theirs ?? 0)}
          </Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterRoundProps>> = {
  default: {"round":2,"clock":"1:42","ours":7,"theirs":5,"urgent":false},
  urgent: {"round":12,"clock":"0:09","ours":12,"theirs":12,"urgent":true},
  clock: {"round":0,"clock":"14:58","ours":0,"theirs":-1,"urgent":false},
  overflow: {"round":123,"clock":"1:02:44","ours":12345,"theirs":67890,"urgent":false},
};
