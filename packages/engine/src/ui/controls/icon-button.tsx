import { Icon, Show, View, packClass, type UiEvent } from "@origingame/engine/ui";
import type { ButtonVariant } from "./button";

export interface IconButtonProps {
  /** What a screen reader calls it: required, since it has no words. */
  label: string;
  /** The atlas icon it shows (close, settings, play). */
  icon: string;
  /** The look, as Button's: `ghost` (the default), `neutral`, `primary`, `danger`, `outline` … */
  variant?: ButtonVariant | undefined;
  /** `sm` (44 px target, 16 px icon), `md` (44 px, 20 px icon, the default) or `lg` (56 px, 24 px icon). */
  size?: "sm" | "md" | "lg" | undefined;
  /** A toggle that is on (mute, pin). */
  pressed?: boolean | undefined;
  /** Its action is running: the pack's spinner replaces the icon. */
  busy?: boolean | undefined;
  disabled?: boolean | undefined;
  /** What pressing it does, read after its name. */
  description?: string | undefined;
  /** Classes after its own. */
  class?: string | undefined;
  onPress?: ((e: UiEvent) => void) | undefined;
}

const BOX = { sm: "size-11", md: "size-11", lg: "size-14" };
const ICON = { sm: "size-4", md: "size-5", lg: "size-6" };

/**
 * A square press target with only an icon (close, settings, mute): always
 * at least the 44 px touch target, always named for screen readers. The
 * pack dresses it as a Button (`Button.plate.<variant>`), every state
 * included; `IconButton.frame` adds what is its own (the square, the
 * radius).
 */
export function IconButton(props: IconButtonProps) {
  const variant = () => {
    const v = props.variant ?? "ghost";
    return v === "quiet" ? "ghost-muted" : v === "neutral" ? undefined : v;
  };
  const size = () => props.size ?? "md";
  return (
    <View
      role="button"
      label={props.label}
      description={props.description}
      pressed={props.pressed}
      busy={props.busy}
      disabled={props.disabled}
      data-pressed={props.pressed}
      class={`flex-row items-center justify-center p-0 shrink-0 ${BOX[size()]} ${packClass("Button", "plate", variant())} ${packClass("IconButton", "frame")} ${props.class ?? ""}`}
      onPress={(e) => props.onPress?.(e)}
    >
      <Show when={props.busy} fallback={<Icon class={ICON[size()]} name={props.icon} />}>
        <View class={packClass("Button", "spinner")} />
      </Show>
    </View>
  );
}

const press = () => {};

export const ground = "panel";

export const fixtures: Record<string, Partial<IconButtonProps>> = {
  ghost: { label: "Close", icon: "close", onPress: press },
  neutral: { label: "Settings", icon: "settings", variant: "neutral", onPress: press },
  primary: { label: "Play", icon: "play", variant: "primary", size: "lg", onPress: press },
  pressed: { label: "Mute", icon: "volume", pressed: true, onPress: press },
  disabled: { label: "Locked", icon: "lock", disabled: true, onPress: press },
};
