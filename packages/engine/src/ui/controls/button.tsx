import { Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

/** rune-kit's button variants; `quiet` is the old name of `ghost-muted`. */
export type ButtonVariant = "neutral" | "primary" | "danger" | "warning" | "outline" | "contrast" | "ghost" | "ghost-muted" | "ghost-faint" | "quiet";

export interface ButtonProps {
  /**
   * The look, rune-kit's matrix: `neutral` (the default), `primary` (the
   * main action), `danger`, `warning`, `contrast` (the inverse of the
   * page), `outline` (a rim, no fill) and the ghosts `ghost`,
   * `ghost-muted` and `ghost-faint` (no plate until hovered). `quiet` is
   * `ghost-muted`. The pack dresses each through `Button.plate.<variant>`.
   */
  variant?: ButtonVariant | undefined;
  /** Its padding and type: `sm`, `md` (the default) or `lg`. Each is a touch target 44 px tall or more. */
  size?: "sm" | "md" | "lg" | undefined;
  /** `size="sm"`, its old name. */
  small?: boolean | undefined;
  /** A toggle that is on: screen readers say so, and `aria-pressed:` styles apply. */
  pressed?: boolean | undefined;
  /** The current one of a set (a tab, a page): screen readers say so, and `selected:` styles apply. */
  selected?: boolean | undefined;
  /** Its action is running: screen readers say so, it shows the pack's spinner (`Button.spinner`) and `aria-busy:` styles apply. Presses still count; set `disabled` as well to refuse them. */
  busy?: boolean | undefined;
  disabled?: boolean | undefined;
  /** What a screen reader calls it when its content is not words (an icon). */
  label?: string | undefined;
  /** What pressing it does, read after its name. */
  description?: string | undefined;
  /** What it is for screen readers, when not a button (a tab, a menu item). */
  role?: string | undefined;
  /** Classes of the document that uses it; they come after its own, so they win. */
  class?: string | undefined;
  tabIndex?: number | undefined;
  /** It takes the focus once it shows (a menu's first choice, a dialog's safe one). */
  autofocus?: boolean | undefined;
  /** Seconds a press or confirm must be held before it presses (hold-to-confirm). */
  hold?: number | undefined;
  onPress?: ((e: UiEvent) => void) | undefined;
  /** The hold's progress, 0 to 1, in `e.value` ("0" when let go early). */
  onHold?: ((e: UiEvent) => void) | undefined;
  onLongPress?: ((e: UiEvent) => void) | undefined;
  onAction?: Record<string, (e: UiEvent) => void> | undefined;
  children?: any;
  [data: `data-${string}`]: unknown;
}

/** Each size's padding, gap and type; every one at least a touch target tall. */
const SIZE = {
  sm: "min-h-11 px-3 gap-1.5 text-xs",
  md: "min-h-11 px-4 gap-2 text-sm",
  lg: "min-h-14 px-6 gap-2 text-base",
};

/**
 * A press target (NSH base's `Button`): it owns its presses, its pressed,
 * selected, busy and disabled states and its role for screen readers, and
 * leaves its look to the pack: `Button.plate` and `Button.plate.<variant>`
 * dress it, every state included. Its content is a row, centred.
 */
export function Button(props: ButtonProps) {
  const variant = () => (props.variant === "quiet" ? "ghost-muted" : props.variant === "neutral" ? undefined : props.variant);
  const size = () => props.size ?? (props.small ? "sm" : "md");
  return (
    <View
      {...props}
      role={props.role ?? "button"}
      data-pressed={props.pressed}
      class={`flex-row items-center justify-center font-semibold ${packClass("Button", "plate", variant())} ${SIZE[size()]} ${props.class ?? ""}`}
    >
      <Show when={props.busy}>
        <View class={packClass("Button", "spinner")} />
      </Show>
      {props.children}
    </View>
  );
}

const press = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<ButtonProps>> = {
  default: { onPress: press, children: () => <Text>Continue</Text> },
  primary: { variant: "primary", onPress: press, children: () => <Text>Start</Text> },
  danger: { variant: "danger", onPress: press, children: () => <Text>Delete save</Text> },
  warning: { variant: "warning", onPress: press, children: () => <Text>Spend 50</Text> },
  contrast: { variant: "contrast", onPress: press, children: () => <Text>Sign in</Text> },
  outline: { variant: "outline", onPress: press, children: () => <Text>Cancel</Text> },
  ghost: { variant: "ghost", onPress: press, children: () => <Text>Details</Text> },
  "ghost-muted": { variant: "ghost-muted", size: "sm", onPress: press, children: () => <Text>Later</Text> },
  "ghost-faint": { variant: "ghost-faint", size: "sm", onPress: press, children: () => <Text>Skip</Text> },
  large: { variant: "primary", size: "lg", onPress: press, children: () => <Text>Play</Text> },
  pressed: { pressed: true, onPress: press, children: () => <Text>Auto</Text> },
  busy: { variant: "primary", busy: true, onPress: press, children: () => <Text>Saving</Text> },
  disabled: { disabled: true, onPress: press, children: () => <Text>Locked</Text> },
};
