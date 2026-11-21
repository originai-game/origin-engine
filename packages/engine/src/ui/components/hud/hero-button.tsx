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
      class={`group/hero flex-row items-center justify-center min-h-12 w-full max-w-50 py-3 px-4 gap-x-3 rounded-panel border-2 border-rim bg-accent shadow-key transition duration-150 hover:-translate-y-px active:translate-y-0.5 active:shadow-key data-premium:bg-none data-premium:bg-gold data-secondary:bg-none data-secondary:bg-fg-subtle data-danger:bg-none data-danger:bg-danger disabled:translate-y-0 disabled:opacity-50`}
      role="button"
      label={props.label}
      disabled={props.disabled}
      onPress={(e) => props.onPress?.(e)}
      data-premium={tone === "premium"}
      data-secondary={tone === "secondary"}
      data-danger={tone === "danger"}
    >
      <Text class={`text-lg uppercase text-center whitespace-nowrap text-accent-fg group-data-premium/hero:text-fg-inverse group-data-secondary/hero:text-fg-inverse`}>{props.label}</Text>
      <Show when={props.key}>
        <Text class={`text-caption uppercase text-center whitespace-nowrap text-accent-fg group-data-premium/hero:text-fg-inverse group-data-secondary/hero:text-fg-inverse`}>{props.key}</Text>
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
