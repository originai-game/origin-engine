import { Show, Text, View, t } from "@origingame/engine/ui";
import type { Entity } from "@origingame/engine";
import "./world.en.ogt";

export interface SpeechBubbleProps {
  /** Who speaks, by its bits (0: use at). */
  entity?: number;
  /** A world point it speaks from without an entity (x y z). */
  at?: string;
  /** World units above the point (over the head). */
  height?: number;
  /** Who speaks, over the line in their reaction's ink; empty is none. */
  speaker?: string;
  /** The speaker's colour: friendly (empty), hostile, neutral or player. */
  reaction?: string;
  /** The line; it wraps at 27 characters. */
  text: string;
  /** A shout: a warm rim and a bolder line. */
  shout?: boolean;
  /** How much of the line's time is left, 0 to 1, a thin line along the bubble's foot; negative draws none. The game times the line out and removes the bubble. */
  left?: number;
}

/**
 * A line spoken over a head: the HUD's glass with a cut corner, the
 * speaker in their colour, the line wrapping at 27 characters and a thin
 * line of the time left. It pops from its tail. Read only; the game
 * decides when it shows and when it goes. The old tail (a clip-path
 * triangle) has no TSX surface: it stands in as a small diamond under the
 * box. A nine-cut box takes no rim or outer shadow (they paint as
 * rectangles around the cut), so the glass's rim, the drop shadow and a
 * shout's warm rim are dropped; a shout keeps its bolder line. The 27
 * character measure stands as a fixed 12.6rem (the ch unit and the
 * viewport room caps have no class surface). Bubbles stack clear of each
 * other (the anchor's declutter).
 */
export function SpeechBubble(props: SpeechBubbleProps) {
  const reaction = () => ((props.reaction ?? "") == "" ? "friendly" : (props.reaction ?? ""));
  const speaker = () => props.speaker ?? "";
  const left = () => props.left ?? -1;
  const point = () => ((props.at ?? "0 0 0").split(/\s+/).map(Number) as [number, number, number]);
  return (
    <View
      anchor={props.entity ? (props.entity as Entity) : point()}
      lift={[0, props.height ?? 0, 0]}
      declutter="stack"
      class="-translate-x-1/2 -translate-y-full group flex-col items-center animate-pop text-fg"
      role="status"
      label={speaker() !== "" ? t("world.speech.name", { speaker: speaker() }) : props.text}
      data-shout={props.shout}
      data-reaction-hostile={reaction() === "hostile"}
      data-reaction-neutral={reaction() === "neutral"}
      data-reaction-player={reaction() === "player"}
    >
      <View class="relative flex-col gap-y-[0.143rem] min-w-0 max-w-[12.6rem] py-2 px-[0.929rem] text-[1.036rem] leading-[1.32] nine-plate-cut bg-plate-hi">
        <Show when={speaker() !== ""}>
          <Text class="min-w-0 max-w-full text-[0.75rem] font-bold tracking-[0.13em] uppercase whitespace-nowrap truncate text-success text-shadow-2xs group-data-reaction-hostile:text-danger group-data-reaction-neutral:text-gold group-data-reaction-player:text-hud-accent">{speaker()}</Text>
        </Show>
        <Text class="min-w-0 group-data-shout:font-bold">{props.text}</Text>
        <Show when={left() >= 0}>
          <View class="absolute left-[0.714rem] right-[0.714rem] bottom-[0.214rem] h-[2px] bg-white/12">
            <View class="h-full bg-white/55" style={{ width: `${Math.min(Math.max(left(), 0), 1) * 100}%` }} />
          </View>
        </Show>
      </View>
      <View class="size-[0.714rem] rotate-45 -mt-[1px] bg-plate-hi" />
    </View>
  );
}

export const fixtures: Record<string, Partial<SpeechBubbleProps>> = {
  default: {"entity":0,"at":"0 0 0","height":2.2,"speaker":"Mira","reaction":"","text":"The Colossus guards the gate. Break its shield, then strike the runes!","shout":false,"left":0.64},
  short: {"text":"Over here!","left":0.3},
  shout: {"speaker":"Vorric","reaction":"hostile","text":"Kneel before the ash!","shout":true,"left":0.8},
  nameless: {"speaker":"","text":"Wait for me at the bridge.","left":-1},
  overflow: {"speaker":"Keeper of the Northern Lighthouse","text":"The lamps went dark three nights ago, and since then the ships have stopped coming; cross the gate before the tide turns.","left":0.12},
};
