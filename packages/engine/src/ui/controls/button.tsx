import { Text, View, type UiEvent } from "@origingame/engine/ui";

export interface ButtonProps {
  /** The look: a plain button, the primary action, a destructive one, a ghost (rim only) or quiet (no ground). */
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
  "|": "text-fg bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] transition duration-317 py-0 px-4 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] active:bg-none active:bg-raised-pressed active:border-border active:shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_2px_5px_0px_#00000099] active:py-0 active:px-4 hover:bg-none hover:bg-raised-hover-lo hover:border-border hover:shadow-[inset_0px_0px_0px_1px_#8892a8,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff29,0px_6px_14px_-4px_#00000099] data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
  "|small": "text-fg bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] transition duration-317 py-0 px-3 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] active:bg-none active:bg-raised-pressed active:border-border active:shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_2px_5px_0px_#00000099] active:py-0 active:px-3 hover:bg-none hover:bg-raised-hover-lo hover:border-border hover:shadow-[inset_0px_0px_0px_1px_#8892a8,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff29,0px_6px_14px_-4px_#00000099] data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
  "primary|": "text-accent-fg bg-linear-to-b from-white/42 to-transparent to-48% bg-accent transition duration-317 py-0 px-4 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] active:bg-linear-to-b active:from-black/16 active:to-transparent active:to-60% active:bg-accent active:border-transparent active:py-0 active:px-4 hover:bg-linear-to-b hover:from-white/62 hover:to-white/14 hover:to-60% hover:bg-accent hover:border-transparent hover:shadow-none data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
  "primary|small": "text-accent-fg bg-linear-to-b from-white/42 to-transparent to-48% bg-accent transition duration-317 py-0 px-3 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] active:bg-linear-to-b active:from-black/16 active:to-transparent active:to-60% active:bg-accent active:border-transparent active:py-0 active:px-3 hover:bg-linear-to-b hover:from-white/62 hover:to-white/14 hover:to-60% hover:bg-accent hover:border-transparent hover:shadow-none data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
  "danger|": "text-danger bg-danger/10 shadow-[inset_0px_0px_0px_1px_#ff959973,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] transition duration-317 py-0 px-4 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] hover:text-danger hover:bg-none hover:bg-danger/15 active:bg-none active:bg-danger/15 active:py-0 active:px-4 data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
  "danger|small": "text-danger bg-danger/10 shadow-[inset_0px_0px_0px_1px_#ff959973,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] transition duration-317 py-0 px-3 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] hover:text-danger hover:bg-none hover:bg-danger/15 active:bg-none active:bg-danger/15 active:py-0 active:px-3 data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
  "ghost|": "text-fg border-border-strong shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] transition duration-317 py-0 px-3.75 border data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] active:bg-none active:bg-fill active:shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_2px_5px_0px_#00000099] active:py-0 hover:bg-none hover:bg-fill hover:shadow-[inset_0px_0px_0px_1px_#8892a8,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff29,0px_6px_14px_-4px_#00000099] data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
  "ghost|small": "text-fg border-border-strong shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] transition duration-317 py-0 px-3.75 border data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] active:bg-none active:bg-fill active:shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_2px_5px_0px_#00000099] active:py-0 active:px-2.75 hover:bg-none hover:bg-fill hover:shadow-[inset_0px_0px_0px_1px_#8892a8,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff29,0px_6px_14px_-4px_#00000099] data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
  "quiet|": "text-fg shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] transition duration-317 py-0 px-4 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] active:bg-none active:bg-fill active:border-border active:shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_2px_5px_0px_#00000099] active:py-0 active:px-4 hover:bg-none hover:bg-fill hover:shadow-none hover:border-0 data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
  "quiet|small": "text-fg shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] transition duration-317 py-0 px-3 data-pressed:text-fg disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:border-border disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] active:bg-none active:bg-fill active:border-border active:shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_2px_5px_0px_#00000099] active:py-0 active:px-3 hover:bg-none hover:bg-fill hover:shadow-none hover:border-0 data-pressed:hover:text-fg data-pressed:hover:bg-none data-pressed:hover:bg-border-strong data-pressed:hover:border-border data-pressed:hover:shadow-[inset_0px_0px_0px_1.5px_#e8e8ed,inset_0px_2px_5px_0px_#00000099]",
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
