import { Show, Text, View, packClass } from "@origingame/engine/ui";

export interface PanelProps {
  /** Its heading (Settings, Inventory); empty: none. */
  title?: string | undefined;
  /** Classes of the document that uses it (a width, a placement), after its own. */
  class?: string | undefined;
  /** Its content, laid out in a column. */
  children?: any;
}

/**
 * A titled panel: a window's frame (`Panel.frame`), the pack's room and gaps
 * inside it (`Panel.body`) and its heading (`Panel.title`, the words or the
 * band they sit on). Its content is a column. Every look is the pack's.
 */
export function Panel(props: PanelProps) {
  return (
    <View class={`flex-col ${packClass("Panel", "frame")} ${packClass("Panel", "body")} ${props.class ?? ""}`}>
      <Show when={props.title}>
        <Text class={packClass("Panel", "title")} role="heading" level={2}>{props.title}</Text>
      </Show>
      {props.children}
    </View>
  );
}

export const fixtures: Record<string, Partial<PanelProps>> = {
  default: {
    title: "Inventory",
    class: "w-96",
    children: () => <Text>Twelve slots, three of them filled.</Text>,
  },
};
