import { For, Icon, Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export type RichTone = "fg" | "muted" | "accent" | "success" | "warn" | "danger" | "info" | "gold" | "common" | "uncommon" | "rare" | "epic" | "legendary" | "mythic";

/** One run of a RichText: words in one style, an icon, or a glossary keyword. */
export interface RichSpan {
  /** Its words (empty with `icon` alone). */
  text?: string | undefined;
  /** Its colour; `fg` (the default) reads in the ground's ink. */
  tone?: RichTone | undefined;
  bold?: boolean | undefined;
  italic?: boolean | undefined;
  /** An atlas icon drawn inline, one em square, before the words. */
  icon?: string | undefined;
  /** A keyword the game explains: underlined, focusable, and its `gloss` shown in a tooltip on hover or focus. */
  keyword?: string | undefined;
  /** What the keyword means (Burn: 4 damage a second for 3 s). */
  gloss?: string | undefined;
}

export interface RichTextProps {
  /** The runs in reading order. */
  spans: ReadonlyArray<RichSpan>;
  /** Classes after its own (type size, alignment). */
  class?: string | undefined;
  /** A keyword focused or hovered (the game may show a glossary page). */
  onKeyword?: ((e: UiEvent, keyword: string) => void) | undefined;
}

const TONE: Record<RichTone, string> = {
  fg: "",
  muted: "text-fg-muted",
  accent: "text-accent-ink",
  success: "text-success",
  warn: "text-warn",
  danger: "text-danger",
  info: "text-info",
  gold: "text-gold",
  common: "text-common",
  uncommon: "text-uncommon",
  rare: "text-rare",
  epic: "text-epic",
  legendary: "text-legendary",
  mythic: "text-mythic",
};

/**
 * Structured inline text for tooltips, card rules and item descriptions:
 * runs of words in a tone, bold or italic, inline icons, and keywords that
 * take focus and show their gloss in a tooltip (`RichText.keyword`,
 * `RichText.gloss`). The runs wrap as one paragraph. It decides nothing:
 * the game writes the runs.
 */
export function RichText(props: RichTextProps) {
  return (
    <View class={`flex-row flex-wrap items-baseline gap-x-1 gap-y-0.5 min-w-0 ${props.class ?? ""}`}>
      <For each={props.spans}>
        {(s) => (
          <Show
            when={s().keyword}
            fallback={
              <View class="flex-row items-baseline gap-0.5 min-w-0">
                <Show when={s().icon}>
                  <Icon class={`size-4 self-center ${TONE[s().tone ?? "fg"]}`} name={s().icon ?? ""} />
                </Show>
                <Show when={s().text}>
                  <Text class={`${TONE[s().tone ?? "fg"]} ${s().bold ? "font-bold" : ""} ${s().italic ? "italic" : ""}`}>{s().text}</Text>
                </Show>
              </View>
            }
          >
            <View
              class={`group/kw relative flex-row items-baseline gap-0.5 ${packClass("RichText", "keyword")}`}
              role="button"
              tabIndex={0}
              label={s().keyword}
              description={s().gloss}
              onFocus={(e) => props.onKeyword?.(e, s().keyword ?? "")}
              onPress={(e) => props.onKeyword?.(e, s().keyword ?? "")}
            >
              <Show when={s().icon}>
                <Icon class={`size-4 self-center ${TONE[s().tone ?? "accent"]}`} name={s().icon ?? ""} />
              </Show>
              <Text class={`${TONE[s().tone ?? "accent"]} ${s().bold === false ? "" : "font-semibold"}`}>{s().text || s().keyword}</Text>
              <Show when={s().gloss}>
                <View class={`absolute bottom-full left-0 mb-1 w-max max-w-64 z-10 pointer-events-none hidden group-hover/kw:flex group-focus-visible/kw:flex ${packClass("RichText", "gloss")}`} hidden>
                  <Text class="font-bold">{s().keyword}</Text>
                  <Text class="text-fg-muted">{s().gloss}</Text>
                </View>
              </Show>
            </View>
          </Show>
        )}
      </For>
    </View>
  );
}

export const ground = "panel";

export const fixtures: Record<string, Partial<RichTextProps>> = {
  default: {
    spans: [
      { text: "Deal" },
      { text: "12", bold: true, tone: "danger" },
      { text: "damage and apply" },
      { keyword: "Burn", icon: "flame", tone: "warn", gloss: "Takes 4 damage each second for 3 seconds." },
      { text: "." },
    ],
  },
  rarity: { spans: [{ text: "Drops a" }, { text: "Legendary", tone: "legendary", bold: true }, { text: "relic." }] },
};
