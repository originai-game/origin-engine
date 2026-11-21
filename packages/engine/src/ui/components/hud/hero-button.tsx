import { Show, Text, View, type UiEvent } from "@origingame/engine/ui";

export interface HeroButtonProps {
  /** Its words (End turn). */
  label: string;
  /** primary (empty), premium, secondary or danger. */
  tone?: string;
  /** Shown but inert (Waiting). */
  disabled?: boolean;
  /** A keycap beside the words (E); empty is none. */
  key?: string;
  /** Press. */
  onPress?: (e: UiEvent) => void;
}

/**
 * A hero action's button: its words in capitals and a keycap beside them,
 * on a face that stands on its shadow (primary's primary, premium's gold,
 * secondary's glass, danger's danger); it lifts under a pointer and sinks
 * while held, and shows inert when disabled. The disabled face's greying
 * filter has no TSX surface: it shows dimmed instead.
 */
export function HeroButton(props: HeroButtonProps) {
  const tone = props.tone ?? "";
  return (
    <View
      class={`group/hero flex-row items-center justify-center min-h-12 w-full max-w-[14.286rem] py-3 px-4 gap-x-3 rounded-[0.857rem] border-2 border-white/12 bg-linear-to-b from-white/12 to-transparent to-45% bg-accent shadow-[0px_3px_0px_#00000073,0px_4px_12px_0px_#00000059] transition duration-150 hover:-translate-y-[1px] active:translate-y-[2px] active:shadow-[0px_1px_0px_#00000073] data-premium:bg-none data-premium:bg-gold data-secondary:bg-none data-secondary:bg-[#77778445] data-danger:bg-none data-danger:bg-[#ff9599] disabled:translate-y-0 disabled:opacity-50`}
      role="button"
      label={props.label}
      disabled={props.disabled}
      onPress={(e) => props.onPress?.(e)}
      data-premium={tone === "premium"}
      data-secondary={tone === "secondary"}
      data-danger={tone === "danger"}
    >
      <Text class={`text-[18px] uppercase text-center whitespace-nowrap text-accent-fg group-data-premium/hero:text-fg-inverse group-data-secondary/hero:text-fg`}>{props.label}</Text>
      <Show when={props.key}>
        <Text class={`text-caption uppercase text-center whitespace-nowrap text-accent-fg group-data-premium/hero:text-fg-inverse group-data-secondary/hero:text-fg`}>{props.key}</Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<HeroButtonProps>> = {
  default: {"label":"End turn","tone":"","disabled":false,"key":"E"},
  premium: {"label":"Ultimate","tone":"premium","disabled":false,"key":"R"},
  secondary: {"label":"Details","tone":"secondary","disabled":false,"key":""},
  danger: {"label":"Retreat","tone":"danger","disabled":false,"key":""},
  disabled: {"label":"Waiting","tone":"","disabled":true,"key":""},
};
