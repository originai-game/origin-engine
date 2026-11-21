import { Text, View, type UiEvent } from "@origingame/engine/ui";

export interface ButtonProps {
  /** The look: a plain button, the primary action, a destructive one, a ghost (a rim on the plain surface) or quiet (the plain surface, no rim). */
  variant?: "primary" | "danger" | "ghost" | "quiet" | undefined;
  /** The smaller padding a toolbar or a row uses. */
  small?: boolean | undefined;
  /** A toggle that is on (aria-pressed / aria-checked). */
  pressed?: boolean | undefined;
  disabled?: boolean | undefined;
  /** What a screen reader calls it when its content is not words. */
  label?: string | undefined;
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

/** Each variant's look, from the old core's built-in button (default.css) in the default style. */
const LOOK: Record<string, string> = {
  "|": "text-fg bg-raised-lo shadow-raise inset-shadow-sunk inset-ring-1 inset-ring-rim transition duration-317 py-0 px-4 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:inset-ring-1 disabled:inset-ring-rim-soft active:bg-none active:bg-raised-pressed active:border-border active:inset-shadow-sunk active:inset-ring-1 active:inset-ring-rim active:py-0 active:px-4 hover:bg-none hover:bg-raised-hover-lo hover:border-border hover:shadow-plate hover:inset-shadow-sunk hover:inset-ring-1 hover:inset-ring-well-rim data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
  "|small": "text-fg bg-raised-lo shadow-raise inset-shadow-sunk inset-ring-1 inset-ring-rim transition duration-317 py-0 px-3 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:inset-ring-1 disabled:inset-ring-rim-soft active:bg-none active:bg-raised-pressed active:border-border active:inset-shadow-sunk active:inset-ring-1 active:inset-ring-rim active:py-0 active:px-3 hover:bg-none hover:bg-raised-hover-lo hover:border-border hover:shadow-plate hover:inset-shadow-sunk hover:inset-ring-1 hover:inset-ring-well-rim data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
  "primary|": "text-accent-fg bg-linear-to-b from-sheen to-transparent to-48% bg-accent transition duration-317 py-0 px-4 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:inset-ring-1 disabled:inset-ring-rim-soft active:bg-linear-to-b active:from-shade active:to-transparent active:to-60% active:bg-accent active:border-transparent active:py-0 active:px-4 hover:bg-linear-to-b hover:from-sheen hover:to-lit hover:to-60% hover:bg-accent hover:border-transparent hover:shadow-none data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
  "primary|small": "text-accent-fg bg-linear-to-b from-sheen to-transparent to-48% bg-accent transition duration-317 py-0 px-3 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:inset-ring-1 disabled:inset-ring-rim-soft active:bg-linear-to-b active:from-shade active:to-transparent active:to-60% active:bg-accent active:border-transparent active:py-0 active:px-3 hover:bg-linear-to-b hover:from-sheen hover:to-lit hover:to-60% hover:bg-accent hover:border-transparent hover:shadow-none data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
  "danger|": "text-danger bg-glass-lo shadow-raise inset-ring-1 inset-ring-entry-grain transition duration-317 py-0 px-4 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:inset-ring-1 disabled:inset-ring-rim-soft hover:text-danger hover:bg-glass active:bg-raised-pressed active:py-0 active:px-4 data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
  "danger|small": "text-danger bg-glass-lo shadow-raise inset-ring-1 inset-ring-entry-grain transition duration-317 py-0 px-3 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:inset-ring-1 disabled:inset-ring-rim-soft hover:text-danger hover:bg-glass active:bg-raised-pressed active:py-0 active:px-3 data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
  "ghost|": "text-fg bg-surface border-border-strong shadow-raise inset-shadow-sunk inset-ring-1 inset-ring-rim transition duration-317 py-0 px-3.75 border data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:inset-ring-1 disabled:inset-ring-rim-soft active:bg-none active:bg-raised-pressed active:inset-shadow-sunk active:inset-ring-1 active:inset-ring-rim active:py-0 hover:bg-none hover:bg-surface-hover hover:shadow-plate hover:inset-shadow-sunk hover:inset-ring-1 hover:inset-ring-well-rim data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
  "ghost|small": "text-fg bg-surface border-border-strong shadow-raise inset-shadow-sunk inset-ring-1 inset-ring-rim transition duration-317 py-0 px-3.75 border data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:inset-ring-1 disabled:inset-ring-rim-soft active:bg-none active:bg-raised-pressed active:inset-shadow-sunk active:inset-ring-1 active:inset-ring-rim active:py-0 active:px-2.75 hover:bg-none hover:bg-surface-hover hover:shadow-plate hover:inset-shadow-sunk hover:inset-ring-1 hover:inset-ring-well-rim data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
  "quiet|": "text-fg bg-surface shadow-raise inset-shadow-sunk inset-ring-1 inset-ring-rim transition duration-317 py-0 px-4 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:inset-ring-1 disabled:inset-ring-rim-soft active:bg-none active:bg-raised-pressed active:border-border active:inset-shadow-sunk active:inset-ring-1 active:inset-ring-rim active:py-0 active:px-4 hover:bg-none hover:bg-surface-hover hover:shadow-none hover:border-0 data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
  "quiet|small": "text-fg bg-surface shadow-raise inset-shadow-sunk inset-ring-1 inset-ring-rim transition duration-317 py-0 px-3 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:inset-ring-1 disabled:inset-ring-rim-soft active:bg-none active:bg-raised-pressed active:border-border active:inset-shadow-sunk active:inset-ring-1 active:inset-ring-rim active:py-0 active:px-3 hover:bg-none hover:bg-surface-hover hover:shadow-none hover:border-0 data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:inset-shadow-sunk data-pressed:hover:inset-ring-2 data-pressed:hover:inset-ring-accent",
};

/**
 * A button: a row of its content, centred, at least a touch target tall, in the
 * button type. A pack restyles it through the "Button.base" variants.
 */
export function Button(props: ButtonProps) {
  return (
    <View
      {...props}
      role="button"
      data-pressed={props.pressed}
      class={`flex-row items-center justify-center gap-2 min-h-11 text-sm font-semibold ${LOOK[`${props.variant ?? ""}|${props.small ? "small" : ""}`]} ${props.class ?? ""}`}
    />
  );
}

export const fixtures: Record<string, Partial<ButtonProps>> = {
  default: { children: () => <Text>Continue</Text> },
  primary: { variant: "primary", children: () => <Text>Start</Text> },
  danger: { variant: "danger", children: () => <Text>Delete save</Text> },
  ghost: { variant: "ghost", children: () => <Text>Cancel</Text> },
  quiet: { variant: "quiet", small: true, children: () => <Text>Skip</Text> },
  pressed: { pressed: true, children: () => <Text>Auto</Text> },
  disabled: { disabled: true, children: () => <Text>Locked</Text> },
};
