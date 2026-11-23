import { Scroll, Show, Text, View, packClass, type UiChild, type UiEvent } from "@origingame/engine/ui";

export interface DialogProps {
  /** Whether the dialog is shown. */
  open: boolean;
  /** Dialog heading and accessible name. */
  title: string;
  /** Optional subtitle/description under the title. */
  description?: string | undefined;
  /** Render with `role="alertdialog"` for urgent or destructive prompts. */
  alert?: boolean | undefined;
  /** Called when the player presses Escape / `back` or clicks the scrim. */
  onClose?: ((e: UiEvent) => void) | undefined;
  /** Action buttons placed in the footer row. */
  footer?: UiChild;
  /** Dialog body content. */
  children?: UiChild;
  class?: string | undefined;
}

/**
 * A modal dialog with scrim, focus trap (`modal={true}`), scrollable body,
 * and optional footer slot. Dismissal calls `onClose`.
 */
export function Dialog(props: DialogProps) {
  return (
    <Show when={props.open}>
      <View
        class={`absolute inset-0 flex-col items-center justify-center p-6 compact:p-3 ${packClass("Modal", "scrim")}`}
      >
        <View
          class="absolute inset-0 bg-scrim"
          onPress={(e) => props.onClose?.(e)}
        />
        <View
          role={props.alert ? "alertdialog" : "dialog"}
          modal={true}
          label={props.title}
          description={props.description}
          onAction={{ back: (e) => props.onClose?.(e) }}
          class={`relative flex-col shrink min-h-0 w-115 max-w-full max-h-full pt-6 pb-4 px-6 gap-3 rounded-control bg-layer-1 text-fg inset-ring-1 inset-ring-rim shadow-panel compact:p-4 ${packClass("Panel", "frame")} ${packClass("Modal", "frame")} ${props.class ?? ""}`}
        >
          <View class="flex-col gap-1">
            <Text class="font-display text-2xl font-bold leading-tight text-fg m-0">
              {props.title}
            </Text>
            <Show when={props.description !== undefined && props.description !== ""}>
              <Text class="text-sm text-fg-muted m-0">{props.description}</Text>
            </Show>
          </View>
          <Show when={props.children !== undefined}>
            <Scroll class="min-h-0 shrink flex-col gap-2">
              {props.children}
            </Scroll>
          </Show>
          <Show when={props.footer !== undefined}>
            <View class="flex-row flex-wrap-reverse items-center justify-end gap-2 pt-2 border-t border-rim-soft">
              {props.footer}
            </View>
          </Show>
        </View>
      </View>
    </Show>
  );
}

const noop = () => {};

export const fixtures: Record<string, Partial<DialogProps>> = {
  default: {
    open: true,
    title: "Save progress",
    description: "Choose a slot or create a new save.",
    onClose: noop,
    children: () => <Text class="text-base text-fg">Chapter 3 — 02:14:08</Text>,
  },
};
