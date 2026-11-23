import { For, Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface ToastItem {
  /** Its key: a toast keeps its node while it stays in the list. */
  id: string;
  /** What it says. */
  title: string;
  /** A second, quieter line. */
  description?: string | undefined;
  /** Its tone: plain, `success` or `danger` (the pack's `Toast.frame.<tone>`). */
  tone?: "success" | "danger" | undefined;
}

export interface ToastStackProps {
  /** The toasts shown, newest last. The game owns the list and its timing: it drops a toast after its time, or when `onDismiss` reports a swipe. */
  toasts: readonly ToastItem[];
  /** Classes of the document that uses it, on the stack. */
  class?: string | undefined;
  onDismiss?: ((e: UiEvent, id: string) => void) | undefined;
}

/**
 * Short messages that come and go at the top of the screen (NSH base's
 * `ToastStack` and `Toast`): each is announced to screen readers as it
 * arrives (a `status`, a danger one an `alert`), enters and leaves by the
 * pack's `Toast.frame`, and a swipe sideways or up dismisses it. The
 * stack (`Toast.stack`) clears the top safe area and passes presses
 * through between toasts. A game puts it on the `toast` layer.
 */
export function ToastStack(props: ToastStackProps) {
  return (
    <View class={`absolute inset-x-0 top-0 ${packClass("Toast", "stack")} ${props.class ?? ""}`}>
      <For each={props.toasts} key={(t) => t.id}>
        {(t) => (
          <View
            role={t().tone === "danger" ? "alert" : "status"}
            class={packClass("Toast", "frame", t().tone)}
            onSwipe={(e) => (e.key === "down" ? undefined : props.onDismiss?.(e, t().id))}
          >
            <View class="flex-col min-w-0 grow">
              <Text class="font-semibold truncate">{t().title}</Text>
              <Show when={t().description}>
                <Text class="text-sm text-fg-muted">{t().description}</Text>
              </Show>
            </View>
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<ToastStackProps>> = {
  default: {
    toasts: [
      { id: "a", title: "Saved", description: "Your progress is in the cloud.", tone: "success" },
      { id: "b", title: "Friend request from Mira" },
      { id: "c", title: "Connection lost", description: "Retrying…", tone: "danger" },
    ],
  },
};
