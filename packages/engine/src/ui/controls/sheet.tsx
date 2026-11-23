import { Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";
import { Button } from "./button";

export interface SheetProps {
  /** Shown or not. The game owns it: Escape, `back`, a press on the scrim or a swipe down reports `onClose`. */
  open: boolean;
  /** What screen readers call it. */
  label?: string | undefined;
  /** The handle at its top (default true). */
  grabber?: boolean | undefined;
  /** Classes of the document that uses it, on its panel. */
  class?: string | undefined;
  onClose?: ((e: UiEvent) => void) | undefined;
  /** What sits at its foot, under its content (the main action). */
  footer?: any;
  children?: any;
}

/**
 * A panel that rises from the bottom of the screen over a scrim (NSH
 * base's `Sheet`, the phone's form of a menu or a picker): modal for
 * screen readers and focus. The pack draws its panel (`Sheet.frame`, which
 * slides in and out) and its handle (`Sheet.grabber`). It fills its
 * parent, so a game puts it on the `modal` layer.
 */
export function Sheet(props: SheetProps) {
  const close = (e: UiEvent) => props.onClose?.(e);
  return (
    <Show when={props.open}>
      <View class="absolute inset-0 flex-col">
        <View class={packClass("Modal", "scrim")} onPress={close} />
        <View
          role="dialog"
          modal={true}
          label={props.label}
          class={`${packClass("Sheet", "frame")} ${props.class ?? ""}`}
          onAction={{ back: close }}
          onSwipe={(e) => (e.key === "down" ? close(e) : undefined)}
        >
          <Show when={props.grabber !== false}>
            <View class={packClass("Sheet", "grabber")} />
          </Show>
          {props.children}
          {props.footer}
        </View>
      </View>
    </Show>
  );
}

const press = () => {};

export const fixtures: Record<string, Partial<SheetProps>> = {
  default: {
    open: true,
    label: "Sort by",
    onClose: press,
    children: () => [
      <Text class="text-lg font-bold">Sort by</Text>,
      <Button variant="ghost" class="justify-start" onPress={press}><Text>Rarity</Text></Button>,
      <Button variant="ghost" class="justify-start" onPress={press}><Text>Power</Text></Button>,
      <Button variant="ghost" class="justify-start" onPress={press}><Text>Newest</Text></Button>,
    ],
    footer: () => <Button variant="primary" onPress={press}><Text>Done</Text></Button>,
  },
};
