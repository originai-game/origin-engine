import { For, Icon, Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface CollapsibleProps {
  /** Its content is shown. The game owns it: a press on the header reports the next value with `onOpenChange`. */
  open: boolean;
  /** The header's text. */
  title: string;
  disabled?: boolean | undefined;
  /** Classes of the document that uses it. */
  class?: string | undefined;
  onOpenChange?: ((e: UiEvent, open: boolean) => void) | undefined;
  children?: any;
}

/**
 * A section whose content is shown or left out (NSH base's `Collapsible`):
 * its header is a button that says whether it is expanded, and its
 * content enters and leaves by the pack's `Collapsible.panel`. The pack
 * draws the header (`Collapsible.trigger`) and the chevron that turns
 * (`Collapsible.chevron`).
 */
export function Collapsible(props: CollapsibleProps) {
  return (
    <View class={`group/collapsible flex-col ${props.class ?? ""}`} expanded={props.open}>
      <View
        role="button"
        class={packClass("Collapsible", "trigger")}
        expanded={props.open}
        disabled={props.disabled}
        onPress={(e) => props.onOpenChange?.(e, !props.open)}
      >
        <Text class="min-w-0 grow">{props.title}</Text>
        <Icon name="chevronDown" class={packClass("Collapsible", "chevron")} />
      </View>
      <Show when={props.open}>
        <View role="region" label={props.title} class={packClass("Collapsible", "panel")}>
          {props.children}
        </View>
      </Show>
    </View>
  );
}

export interface AccordionSection {
  value: string;
  title: string;
  disabled?: boolean | undefined;
  /** What it shows while open. */
  content: () => any;
}

export interface AccordionProps {
  /** The open sections' values. The game owns them: a press reports the next list with `onChange`. */
  value: readonly string[];
  sections: readonly AccordionSection[];
  /** Several may be open at once; otherwise opening one closes the others. */
  multiple?: boolean | undefined;
  /** Classes of the document that uses it. */
  class?: string | undefined;
  onChange?: ((e: UiEvent, value: string[]) => void) | undefined;
}

/**
 * Stacked sections that expand and collapse (NSH base's `Accordion`), one
 * at a time unless `multiple`, each a `Collapsible` in the pack's
 * `Accordion.item`; the arrows move between their headers.
 */
export function Accordion(props: AccordionProps) {
  const toggle = (e: UiEvent, v: string, open: boolean) => {
    const rest = props.multiple ? props.value.filter((x) => x !== v) : [];
    props.onChange?.(e, open ? [...rest, v] : rest);
  };
  return (
    <View class={`flex-col ${props.class ?? ""}`} focusGroup="stop">
      <For each={props.sections} key={(s) => s.value}>
        {(s) => (
          <Collapsible
            class={packClass("Accordion", "item")}
            title={s().title}
            disabled={s().disabled}
            open={props.value.includes(s().value)}
            onOpenChange={(e, open) => toggle(e, s().value, open)}
          >
            {s().content()}
          </Collapsible>
        )}
      </For>
    </View>
  );
}

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

const change = () => {};

export const fixtures: Record<string, Partial<CollapsibleProps>> = {
  open: { open: true, title: "Rewards", onOpenChange: change, children: () => <Text>300 gold, a rare chest and 2 hero shards.</Text> },
  closed: { open: false, title: "Rules", onOpenChange: change, children: () => <Text>Hidden.</Text> },
};
