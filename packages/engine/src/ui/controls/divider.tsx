import { Show, Text, View, packClass } from "@origingame/engine/ui";

export interface DividerProps {
  /** `horizontal` (the default) or `vertical`. */
  orientation?: "horizontal" | "vertical" | undefined;
  /** Words centred on a horizontal rule (OR, Rewards); empty: a plain rule. */
  label?: string | undefined;
  /** Classes after its own (a margin). */
  class?: string | undefined;
}

/**
 * A hairline rule between groups, horizontal or vertical, optionally with
 * words centred on it. The pack draws the line (`Divider.line`) and the
 * words (`Divider.label`). Read only; screen readers hear a separator.
 */
export function Divider(props: DividerProps) {
  const vertical = () => props.orientation === "vertical";
  return (
    <Show
      when={!vertical()}
      fallback={<View class={`self-stretch w-px shrink-0 ${packClass("Divider", "line")} ${props.class ?? ""}`} role="separator" />}
    >
      <View class={`flex-row items-center gap-3 w-full ${props.class ?? ""}`} role="separator" label={props.label || undefined}>
        <View class={`grow shrink basis-0 h-px ${packClass("Divider", "line")}`} />
        <Show when={props.label}>
          <Text class={`shrink-0 ${packClass("Divider", "label")}`}>{props.label}</Text>
          <View class={`grow shrink basis-0 h-px ${packClass("Divider", "line")}`} />
        </Show>
      </View>
    </Show>
  );
}

export const ground = "panel";

export const fixtures: Record<string, Partial<DividerProps>> = {
  plain: {},
  labelled: { label: "or" },
};
