import { Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";
import { Button } from "./button";

export interface DialogProps {
  /** Shown or not. The game owns it: Escape, `back` or a press on the scrim reports `onClose`, and the game hides it. */
  open: boolean;
  /** Its heading, which screen readers name it by. */
  title?: string | undefined;
  /** What it says under the heading. */
  description?: string | undefined;
  /** A press on the scrim closes it (default true); a dialog that must be answered sets false. */
  dismissible?: boolean | undefined;
  /** Classes of the document that uses it, on its panel. */
  class?: string | undefined;
  onClose?: ((e: UiEvent) => void) | undefined;
  /** The buttons, in a row at its foot. */
  actions?: any;
  /** What it holds between its text and its buttons. */
  children?: any;
}

/**
 * A dialog over a scrim (NSH base's `Dialog`; an alert dialog is one with
 * buttons and no other content): modal for screen readers and focus, it
 * pops in by `Modal.frame` over `Modal.scrim`. The pack draws its panel
 * (`Dialog.frame`), heading (`Dialog.title`), text (`Dialog.body`) and
 * button row (`Dialog.actions`). It fills its parent, so a game puts it
 * on the `modal` layer or over the screen it covers.
 */
export function Dialog(props: DialogProps) {
  const close = (e: UiEvent) => props.onClose?.(e);
  return (
    <Show when={props.open}>
      <View class="absolute inset-0 flex p-4">
        <View class={packClass("Modal", "scrim")} onPress={(e) => (props.dismissible === false ? undefined : close(e))} />
        <View
          role={props.actions && !props.children ? "alertdialog" : "dialog"}
          modal={true}
          label={props.title}
          description={props.description}
          class={`${packClass("Dialog", "frame")} ${packClass("Modal", "frame")} ${props.class ?? ""}`}
          onAction={{ back: close }}
        >
          <Show when={props.title}>
            <Text class={packClass("Dialog", "title")} role="heading" level={2}>{props.title}</Text>
          </Show>
          <Show when={props.description}>
            <Text class={packClass("Dialog", "body")}>{props.description}</Text>
          </Show>
          {props.children}
          <Show when={props.actions}>
            <View class={packClass("Dialog", "actions")}>{props.actions}</View>
          </Show>
        </View>
      </View>
    </Show>
  );
}

const press = () => {};

export const fixtures: Record<string, Partial<DialogProps>> = {
  default: {
    open: true,
    title: "Leave the match?",
    description: "Your team plays on without you, and the match counts as a loss.",
    onClose: press,
    actions: () => [
      <Button variant="ghost-muted" onPress={press}><Text>Stay</Text></Button>,
      <Button variant="danger" autofocus onPress={press}><Text>Leave</Text></Button>,
    ],
  },
};
