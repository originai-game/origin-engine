import { Show, Text, View } from "@origingame/engine/ui";

export interface InputGlyphProps {
  /** The input action whose binding it shows (interact, jump): the pack's icon for it on the device used last. Wins over `keyLabel`. */
  action?: string | undefined;
  /** A fixed key cap's words (E, Space, LMB) when there is no action to look up. */
  keyLabel?: string | undefined;
  /** `sm` (20 px), `md` (26 px, the default) or `lg` (32 px). */
  size?: "sm" | "md" | "lg" | undefined;
  /** A gamepad face button: a round cap. */
  pad?: boolean | undefined;
  /** The key is held: the cap sinks and lights. */
  pressed?: boolean | undefined;
  /** The action cannot be taken now: the cap greys. */
  disabled?: boolean | undefined;
  /** Classes after its own (a margin, a placement). */
  class?: string | undefined;
}

const CAP = {
  sm: "min-w-5 h-5 px-1 text-2xs leading-5",
  md: "min-w-6.5 h-6.5 px-1.25 text-sm leading-6.5",
  lg: "min-w-8 h-8 px-1.5 text-base leading-8",
};
const PAD = { sm: "w-5 px-0", md: "w-6.5 px-0", lg: "w-8 px-0" };
const GLYPH = { sm: "text-base", md: "text-xl", lg: "text-2xl" };

/**
 * One input binding, drawn the same everywhere a prompt names a key: given
 * an action, the binding's own glyph on the device used last (the engine
 * resolves it; it draws unboxed, since the glyph is its own cap); given only
 * words, a key cap on the key gradient in the key ink. Pressed sinks and
 * lights the cap; disabled greys it. Purely presentational: it takes no
 * presses.
 */
export function InputGlyph(props: InputGlyphProps) {
  const size = () => props.size ?? "md";
  return (
    <View
      class={`group/glyph flex-row items-center justify-center shrink-0 pointer-events-none ${props.class ?? ""}`}
      data-pressed={props.pressed === true}
      data-disabled={props.disabled === true}
      data-pad={props.pad === true}
      hidden
    >
      <Show
        when={props.action}
        fallback={
          <Text
            class={`${CAP[size()]} ${props.pad ? `${PAD[size()]} rounded-full` : "rounded-control"} text-center font-bold whitespace-nowrap text-key-fg bg-linear-to-b from-key-hi to-key-lo shadow-key transition duration-instant group-data-pressed/glyph:translate-y-0.5 group-data-pressed/glyph:from-hud-accent-hi group-data-pressed/glyph:to-hud-accent group-data-pressed/glyph:text-hud-accent-fg group-data-pressed/glyph:shadow-glow group-data-disabled/glyph:opacity-dim`}
          >
            {props.keyLabel ?? ""}
          </Text>
        }
      >
        <Text
          class={`${GLYPH[size()]} leading-none text-fg text-shadow-tag transition duration-instant group-data-pressed/glyph:translate-y-0.5 group-data-pressed/glyph:text-hud-accent-ink group-data-disabled/glyph:opacity-dim`}
          glyph={props.action}
        />
      </Show>
    </View>
  );
}

export const ground = "hud";

export const fixtures: Record<string, Partial<InputGlyphProps>> = {
  action: { action: "interact" },
  key: { keyLabel: "E" },
  wide: { keyLabel: "Space", size: "lg" },
  pad: { keyLabel: "A", pad: true },
  pressed: { keyLabel: "F", pressed: true },
  disabled: { keyLabel: "Q", disabled: true },
};
