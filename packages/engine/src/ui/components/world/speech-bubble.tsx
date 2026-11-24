import { Show, Text, View, packClass, t } from "@origingame/engine/ui";
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
  /** Maximum world distance before hiding. */
  maxDistance?: number;
  /** Distance at which the bubble is drawn at scale 1. */
  scaleAt?: number;
  /** Minimum world-distance scale. */
  scaleMin?: number;
  /** Maximum world-distance scale. */
  scaleMax?: number;
  /** Behaviour when behind world geometry. */
  occlusion?: "fade" | "hide" | "none";
}

function parsePoint(at?: string): [number, number, number] {
  const parts = (at ?? "0 0 0").trim().split(/\s+/).map(Number);
  return [
    Number.isFinite(parts[0]) ? parts[0]! : 0,
    Number.isFinite(parts[1]) ? parts[1]! : 0,
    Number.isFinite(parts[2]) ? parts[2]! : 0,
  ];
}

/**
 * A line spoken over a head: a frosted glass plate, the speaker in their
 * colour, the line wrapping cleanly and a thin line of the time left.
 * Bubbles stack clear of each other (the anchor's declutter).
 */
export function SpeechBubble(props: SpeechBubbleProps) {
  const reaction = () => ((props.reaction ?? "") == "" ? "friendly" : (props.reaction ?? ""));
  const speaker = () => props.speaker ?? "";
  const left = () => props.left ?? -1;
  return (
    <View
      anchor={props.entity ? (props.entity as Entity) : parsePoint(props.at)}
      lift={[0, props.height ?? 0, 0]}
      declutter="stack"
      maxDistance={props.maxDistance}
      scaleAt={props.scaleAt}
      scaleMin={props.scaleMin}
      scaleMax={props.scaleMax}
      occlusion={props.occlusion}
      class="-translate-x-1/2 -translate-y-full pointer-events-none group flex-col items-center animate-pop text-fg"
      role="status"
      label={speaker() !== "" ? t("world.speech.name", { speaker: speaker() }) : props.text}
      data-shout={props.shout}
      data-reaction-hostile={reaction() === "hostile"}
      data-reaction-neutral={reaction() === "neutral"}
      data-reaction-player={reaction() === "player"}
    >
      <View class={`relative overflow-hidden flex-col gap-y-0.5 min-w-0 max-w-44 py-2 px-3.25 text-reading leading-snug group-data-shout:inset-ring-regular group-data-shout:inset-ring-warn ${packClass("Plate", "frame")}`}>
        <Show when={speaker() !== ""}>
          <Text class="min-w-0 max-w-full text-2xs font-bold tracking-wide uppercase whitespace-nowrap truncate text-success text-shadow-tag group-data-reaction-hostile:text-danger group-data-reaction-neutral:text-gold group-data-reaction-player:text-hud-accent-ink">{speaker()}</Text>
        </Show>
        <Text class="min-w-0 group-data-shout:font-bold">{props.text}</Text>
        <Show when={left() >= 0}>
          <View class="absolute left-2.5 right-2.5 bottom-0.75 h-0.5 rounded-capsule overflow-hidden bg-track">
            <View class="h-full rounded-capsule bg-sheen" style={{ width: `${Math.min(Math.max(left(), 0), 1) * 100}%` }} />
          </View>
        </Show>
      </View>
      <View class="size-2.5 rotate-45 -mt-1.25 bg-plate-lo border-r border-b border-border" />
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
