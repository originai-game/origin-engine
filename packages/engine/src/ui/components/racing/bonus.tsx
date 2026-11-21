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
    <View class={`flex-col items-center gap-y-5.5 w-fit max-w-full text-fg compact:flex-row compact:justify-center compact:gap-x-3`} role="group" label={t("racing.bonus.name")}>
      <Show when={props.meter !== ""}>
        <View class={`flex-col items-center gap-y-1.5 max-w-full compact:flex-[0_1_auto] compact:min-w-0`} role="meter" label={props.meter} valueMin={0} valueMax={100} valueNow={share() * 100}>
          <Text class={`max-w-full min-w-0 text-xs font-semibold tracking-kicker uppercase whitespace-nowrap overflow-hidden text-hud-accent text-stroke-3 text-stroke-bg compact:tracking-normal`}>{props.meter}</Text>
          <View class={`relative w-45 max-w-full h-1.75 overflow-hidden bg-well shadow-plate ring ring-rim compact:w-25 compact:h-1.25`}>
            <View class="absolute left-0 top-0 h-full bg-hud-accent shadow-glow" style={{ width: `${share() * 100}%` }} />
          </View>
        </View>
      </Show>
      <Show when={props.points !== ""}>
        <Show when={props.at + 1} keyed>
          {() => (
            <View class={`flex-col items-center gap-y-0.5 max-w-full min-w-0 compact:items-start compact:flex-[0_1_auto] animate-stamp`} role="status">
              <Text class={`pl-1 pr-2 text-display-md leading-none italic font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg compact:pl-0 compact:pr-1.5 compact:text-base`}>{props.points}</Text>
              <Show when={props.detail !== ""}>
                <Text class={`max-w-full min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap overflow-hidden text-gold text-stroke-3 text-stroke-bg compact:tracking-normal`}>{props.detail}</Text>
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
  overflow: { "meter": "Slipstream from the car ahead at full draft", "value": 4, "points": "+1,234,567,890", "detail": "Drift · Near miss ×3 · Airtime · Burnout · Clean section · Perfect landing", "at": 3 },
};
