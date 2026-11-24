import { Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface TabsProps {
  /** What screen readers call the list ("Sections"). */
  label?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  children?: any;
}

/**
 * The tab list around `Tab`s (NSH base's `Tabs`): one stop of Tab, at the
 * selected tab; the arrows move among its tabs and select each one they
 * reach (WAI-ARIA's tab list, automatic activation). It holds no value and
 * no content: the game shows the selected tab's content beside it.
 */
export function Tabs(props: TabsProps) {
  return (
    <View role="tablist" label={props.label} focusGroup="wrap" focusRoving class={`${packClass("Tab", "list")} ${props.class ?? ""}`}>
      {props.children}
    </View>
  );
}

export interface TabProps {
  /** The current tab of its list: screen readers say so, `selected:` styles apply and the list's mark sits under it. The game owns the value. */
  selected?: boolean | undefined;
  disabled?: boolean | undefined;
  /** What a screen reader calls it when its children have no text that says it (an icon). */
  label?: string | undefined;
  description?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  /** It was pressed, or the arrows of its list reached it: the game selects it. */
  onPress?: ((e: UiEvent) => void) | undefined;
  children?: any;
}

/**
 * One tab of a tab list (NSH base's `Tab`), in `Tabs` with the others: it
 * owns its presses and its selected state for screen readers; the pack
 * dresses it (`Tab.item`) and the mark under the selected one (`Tab.mark`),
 * which slides from the tab it replaces (`Tab.indicator`). A count beside
 * its label is a `Text` of `packClass("Tab", "count")`, lit while selected.
 */
export function Tab(props: TabProps) {
  return (
    <View
      role="tab"
      class={`relative group/tab ${packClass("Tab", "item")} ${props.class ?? ""}`}
      selected={props.selected === true}
      disabled={props.disabled}
      label={props.label}
      description={props.description}
      onPress={(e) => props.onPress?.(e)}
      onFocus={(e) => {
        if (props.selected !== true) props.onPress?.(e);
      }}
    >
      <Show when={props.selected}>
        <View class={`absolute pointer-events-none ${packClass("Tab", "mark")} ${packClass("Tab", "indicator")}`} />
      </Show>
      {props.children}
    </View>
  );
}

const press = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<TabsProps>> = {
  default: {
    label: "Sections",
    children: () => [
      <Tab selected onPress={press}><Text>Heroes</Text><Text class={packClass("Tab", "count")}>3</Text></Tab>,
      <Tab onPress={press}><Text>Gear</Text><Text class={packClass("Tab", "count")}>12</Text></Tab>,
      <Tab onPress={press}><Text>Skills</Text></Tab>,
      <Tab disabled onPress={press}><Text>Pets</Text></Tab>,
    ],
  },
};
