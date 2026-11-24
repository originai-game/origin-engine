import { Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface HeroButtonProps {
  /** Its words (End turn). */
  label: string;
  /** primary (empty), premium, secondary or danger. */
  tone?: string;
  /** Shown but inert (Waiting). */
  disabled?: boolean;
  /** A keycap beside the words (E); empty is none. */
  key?: string;
  /** A charge filling along its foot, 0 to 1 (a super, a focus), lit and breathing when full; negative is none. Default -1. */
  charge?: number;
  /** Press. */
  onPress?: (e: UiEvent) => void;
}

/**
 * A hero action's button: its words in capitals and a keycap beside them,
 * on a face that stands on its shadow (primary's primary, premium's gold,
 * secondary's glass, danger's danger); it lifts under a pointer and sinks
 * while held, and shows inert when disabled. With a charge, a bar fills
 * along its foot and the button lights when it is full. Its face, words
 * and charge are the Hero component's pack slots.
 */
export function HeroButton(props: HeroButtonProps) {
  const tone = () => props.tone ?? "";
  const charge = () => props.charge ?? -1;
  return (
    <View
      class={`group/hero relative flex-row items-center justify-center ${packClass("Hero", "frame")} ${tone() && tone() !== "primary" ? packClass("Hero", "frame", tone()) : ""} ${charge() >= 1 && !props.disabled ? packClass("Hero", "frame", "ready") : ""}`}
      role="button"
      label={props.label}
      disabled={props.disabled}
      onPress={(e) => props.onPress?.(e)}
      data-premium={tone() === "premium"}
      data-secondary={tone() === "secondary"}
      data-danger={tone() === "danger"}
    >
      <Text class={`whitespace-nowrap text-accent-fg group-data-premium/hero:text-gold-fg group-data-secondary/hero:text-fg group-data-danger/hero:text-danger-fg ${packClass("Hero", "label")}`}>{props.label}</Text>
      <Show when={props.key}>
        <Text class={`text-caption uppercase text-center whitespace-nowrap text-accent-fg group-data-premium/hero:text-gold-fg group-data-secondary/hero:text-fg group-data-danger/hero:text-danger-fg`}>{props.key}</Text>
      </Show>
      <Show when={charge() >= 0}>
        <View class={packClass("Hero", "charge")} role="meter" valueMin={0} valueMax={100} valueNow={Math.min(charge(), 1) * 100}>
          <View class={`transition-layout duration-base ${packClass("Hero", "charge", "fill")}`} style={{ width: `${Math.min(charge(), 1) * 100}%` }} />
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<HeroButtonProps>> = {
  default: {"label":"End turn","tone":"","disabled":false,"key":"E"},
  premium: {"label":"Ultimate","tone":"premium","disabled":false,"key":"R"},
  secondary: {"label":"Details","tone":"secondary","disabled":false,"key":""},
  danger: {"label":"Retreat","tone":"danger","disabled":false,"key":""},
  charging: {"label":"Super","tone":"","disabled":false,"key":"","charge":0.6},
  charged: {"label":"Super","tone":"","disabled":false,"key":"","charge":1},
  disabled: {"label":"Waiting","tone":"","disabled":true,"key":""},
};
