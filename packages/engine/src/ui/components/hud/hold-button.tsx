import { Icon, Show, Text, View, createSignal, type UiEvent } from "@origingame/engine/ui";

export interface HoldButtonProps {
  /** Its words and accessible name (Delete save). */
  label: string;
  /** What it says while it is held (Keep holding…); empty keeps its label. */
  holding?: string;
  /** A line under it, what to do (Hold to delete); empty for none. */
  hint?: string;
  /** An icon of the atlas before the label; empty for none. */
  icon?: string;
  /** An og-input action whose hold holds it too (confirm); empty for none. */
  action?: string;
  /** Its role: danger (the default, a destructive confirm), primary or secondary. Default danger. */
  variant?: string;
  /** Shown but inert. */
  disabled?: boolean;
  /** Confirmed, by holding it 1.2 s or the input action it names. */
  onConfirm?: (e: UiEvent) => void;
}

/** Each variant's look, from the old core's built-in button (default.css). */
const LOOK: Record<string, string> = {
  danger: "text-danger bg-[#2c2837] shadow-[inset_0px_0px_0px_1px_#ff959973,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] active:bg-none active:bg-[#372e3d]",
  primary: "text-[#011e0f] bg-linear-to-b from-white/42 to-transparent to-48% bg-[#55e39a] active:bg-linear-to-b active:from-black/16 active:to-transparent active:to-60% active:bg-[#55e39a] active:border-transparent",
  secondary: "text-fg bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] active:bg-none active:bg-raised-pressed",
};

/** The hold's progress fill, in the variant's ink. */
const FILL: Record<string, string> = {
  danger: "bg-danger",
  primary: "bg-[#011e0f]",
  secondary: "bg-fg",
};

/**
 * A confirm that must be held (Delete save): its icon and words, what it
 * says while it is held, and a line under it saying what to do. Holding
 * it for 1.2 s confirms, as does the input action it names; the hold's
 * progress runs along the button's foot and the words swap while held.
 */
export function HoldButton(props: HoldButtonProps) {
  const variant = props.variant ?? "danger";
  const progress = createSignal(0);
  const held = () => progress() > 0;
  return (
    <View class="flex-col items-center max-w-full">
      <View
        class={`relative flex-row items-center justify-center gap-2 min-h-11 max-w-full py-1 px-4 overflow-hidden transition duration-317 disabled:text-fg-subtle disabled:bg-none disabled:bg-[#7379851d] disabled:border-border disabled:shadow-[inset_0px_0px_0px_1px_#343d4f] ${LOOK[variant] ?? LOOK.secondary}`}
        role="button"
        hold={1.2}
        onHold={(e) => progress.set(e.value === "0" ? 0 : Math.min(Number(e.value), 1))}
        onPress={(e) => {
          progress.set(0);
          props.onConfirm?.(e);
        }}
        onAction={props.action ? { confirm: (e) => props.onConfirm?.(e) } : undefined}
        data-action={props.action}
        label={props.label}
        disabled={props.disabled}
      >
        <Show when={props.icon}>
          <Icon class="size-4 shrink-0" name={props.icon} />
        </Show>
        <Show when={held() && (props.holding ?? "") != ""} fallback={<Text class="text-center">{props.label}</Text>}>
          <Text class="text-center">{props.holding}</Text>
        </Show>
        <Show when={held()}>
          <View class={`absolute bottom-0 left-0 h-[2px] ${FILL[variant] ?? FILL.secondary}`} style={{ width: `${progress() * 100}%` }} />
        </Show>
      </View>
      <Show when={props.hint}>
        <Text class="text-center text-caption text-fg-muted">{props.hint}</Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<HoldButtonProps>> = {
  default: {"label":"Delete save","holding":"Keep holding…","hint":"Hold to delete","icon":"close","action":"","variant":"danger","disabled":false},
  "label-only": {"label":"Secure checkpoint","holding":"","hint":"Hold to secure","icon":"","action":"","variant":"primary","disabled":false},
  disabled: {"label":"Delete save","holding":"Keep holding…","hint":"Unavailable during combat","icon":"close","action":"","variant":"danger","disabled":true},
  overflow: {"label":"Abandon the current expedition","holding":"Keep holding to confirm","hint":"Hold to abandon the expedition","icon":"close","action":"","variant":"danger","disabled":false},
};
