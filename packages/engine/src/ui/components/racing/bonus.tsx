import { Show, Text, View, t } from "@origingame/engine/ui";
import "./racing.en.ogt";

export interface RacingBonusProps {
  /** A meter's name as the game words it (Slipstream, Drafting), over a bar in the HUD's accent; empty hides the meter. */
  meter: string;
  /** The meter's share, 0 to 1. */
  value: number;
  /** The points being earned now as the game words them (+2,450), large and slanted; empty hides them. */
  points: string;
  /** What earns them as the game words it (Drift · Near miss ×3), in gold under the points; empty hides it. */
  detail: string;
  /** The chain's id: a new value replays the points' entrance. */
  at: number;
}

/**
 * Feedback above the road at the top centre: a meter (the slipstream) as
 * a word over a bar in the HUD's accent, and the points a drift or a
 * chain earns, large and slanted, with what earns them in gold under it.
 * The game decides the meter and every figure; read only.
 */
export function RacingBonus(props: RacingBonusProps) {
  const share = () => Math.min(Math.max(props.value, 0), 1);
  return (
    <View class={`flex-col items-center gap-y-[1.571rem] w-fit max-w-full text-fg compact:flex-row compact:justify-center compact:gap-x-[0.857rem]`} role="group" label={t("racing.bonus.name")}>
      <Show when={props.meter !== ""}>
        <View class={`flex-col items-center gap-y-[0.429rem] max-w-full compact:flex-[0_1_auto] compact:min-w-0`} role="meter" label={props.meter} valueMin={0} valueMax={100} valueNow={share() * 100}>
          <Text class={`max-w-full min-w-0 text-[0.857rem] font-semibold tracking-[0.22em] uppercase whitespace-nowrap overflow-hidden text-[#38e8ff] text-shadow-xs text-shadow-bg/55 compact:tracking-normal`}>{props.meter}</Text>
          <View class={`relative w-[12.857rem] max-w-full h-[0.5rem] overflow-hidden bg-[#04060cb8] shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_#ffffff24,0px_2px_5.714px_#00000073] compact:w-[7.143rem] compact:h-[0.357rem]`}>
            <View class="absolute left-0 top-0 h-full bg-[#38e8ff] shadow-[2.288px_0px_10px_#38e8ffbf]" style={{ width: `${share() * 100}%` }} />
          </View>
        </View>
      </Show>
      <Show when={props.points !== ""}>
        <Show when={props.at + 1} keyed>
          {() => (
            <View class={`flex-col items-center gap-y-[0.143rem] max-w-full min-w-0 compact:items-start compact:flex-[0_1_auto] animate-stamp`} role="status">
              <Text class={`pl-[0.286rem] pr-[0.571rem] text-[2.429rem] leading-none italic font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs text-shadow-bg/55 compact:pl-0 compact:pr-[0.429rem] compact:text-[1.143rem]`}>{props.points}</Text>
              <Show when={props.detail !== ""}>
                <Text class={`max-w-full min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-gold text-shadow-xs text-shadow-bg/55 compact:tracking-normal`}>{props.detail}</Text>
              </Show>
            </View>
          )}
        </Show>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingBonusProps>> = {
  default: { "meter": "Slipstream", "value": 0.72, "points": "+2,450", "detail": "Drift · Near miss ×3", "at": 1 },
  meter: { "meter": "Slipstream", "value": 0.3, "points": "", "detail": "", "at": 0 },
  points: { "meter": "", "value": 0, "points": "+800", "detail": "Clean overtake", "at": 2 },
  empty: { "meter": "", "value": 0, "points": "", "detail": "", "at": 0 },
  overflow: { "meter": "Slipstream from the car ahead at full draft", "value": 4, "points": "+1,234,567,890", "detail": "Drift · Near miss ×3 · Airtime · Burnout · Clean section · Perfect landing", "at": 3 },
};
