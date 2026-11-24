import { Icon, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import type { Entity } from "@origingame/engine";
import "./world.en.ogt";

export interface InteractionPromptProps {
  /** What it is over, by its bits (0: use at). */
  entity?: number;
  /** A world point it sits over without an entity (x y z). */
  at?: string;
  /** World units above the point. */
  height?: number;
  /** The action whose glyph the key cap shows, its binding on the device used last (interact); empty shows key. */
  action?: string;
  /** The key as the game names it (E), shown when action is empty. */
  key?: string;
  /** A gamepad's face button: a round cap. */
  pad?: boolean;
  /** What it does, in condensed capitals (Open, Talk, Revive); it always fits whole. */
  verb: string;
  /** What it is under the verb (Gilded Chest); empty is none. */
  name?: string;
  /** The key is down: the cap sinks 2 px and lights. */
  pressed?: boolean;
  /** A hold's progress 0 to 1: an accent ring round the cap on a dark disc (1 is done: the cap lights); negative draws none. */
  hold?: number;
  /** It cannot be used now: the cap greys and the reason shows in red with a lock. */
  disabled?: boolean;
  /** Why it cannot (Needs the Warden's key). */
  reason?: string;
  /** Hidden past this many world units (6). */
  maxDistance?: number;
  /** Distance at which the prompt is drawn at scale 1. */
  scaleAt?: number;
  /** Minimum world-distance scale. */
  scaleMin?: number;
  /** Maximum world-distance scale. */
  scaleMax?: number;
  /** Behaviour when behind world geometry. */
  occlusion?: "fade" | "hide" | "none";
  /** Optional tap/click handler for direct touch or pointer interaction. */
  onPress?: (e: UiEvent) => void;
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
 * What the player can do with what is in front of them: a key cap (or
 * device-adaptive action glyph) beside a frosted glass plate, the verb in
 * condensed capitals and the object under it; a hold's ring round the cap.
 */
export function InteractionPrompt(props: InteractionPromptProps) {
  const hold = () => props.hold ?? -1;
  const action = () => props.action ?? "";
  const key = () => props.key ?? "";
  const name = () => props.name ?? "";
  const reason = () => props.reason ?? "";
  const lit = () => props.pressed || hold() >= 1;
  return (
    <View
      anchor={props.entity ? (props.entity as Entity) : parsePoint(props.at)}
      lift={[0, props.height ?? 0, 0]}
      declutter="stack"
      priority={1}
      maxDistance={props.maxDistance}
      scaleAt={props.scaleAt}
      scaleMin={props.scaleMin}
      scaleMax={props.scaleMax}
      occlusion={props.occlusion}
      class="-translate-x-1/2 -translate-y-full group flex-row items-center animate-pop text-fg"
      role={props.onPress ? "button" : "status"}
      disabled={props.onPress ? props.disabled : undefined}
      onPress={props.onPress ? (e) => props.onPress?.(e) : undefined}
      label={`${props.verb}${name() !== "" ? ` ${name()}` : ""}`}
      description={props.disabled && reason() !== "" ? reason() : ""}
      data-pad={props.pad}
      data-pressed={lit()}
      data-disabled={props.disabled}
    >
      <View class="relative z-1 shrink-0 flex-row items-center justify-center size-9">
        <Show when={hold() >= 0}>
          <View class="absolute inset-0 rounded-full bg-scrim" />
          <View class="absolute inset-0 nine-ring bg-hud-accent transition-layout duration-fast ease-out" wipe={Math.min(Math.max(hold(), 0), 1)} role="meter" label={t("world.prompt.hold")} valueMin={0} valueMax={1} valueNow={Math.min(Math.max(hold(), 0), 1)} />
        </Show>
        <Show
          when={key() !== "" || action() === ""}
          fallback={
            <Text class="z-1 text-xl leading-none text-fg text-shadow-tag group-data-disabled:opacity-dim" glyph={action()} />
          }
        >
          <Text class="z-1 min-w-6.5 h-6.5 px-1.25 text-center text-sm leading-6.5 font-bold whitespace-nowrap text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key group-data-pressed:translate-y-0.5 group-data-pressed:bg-linear-to-b group-data-pressed:from-hud-accent-hi group-data-pressed:to-hud-accent group-data-pressed:shadow-glow-lg group-data-pressed:inset-shadow-lip group-data-disabled:text-fg-subtle group-data-disabled:bg-linear-to-b group-data-disabled:from-fg-subtle group-data-disabled:to-fg-subtle group-data-pad:px-0 group-data-pad:w-6.5 group-data-pad:min-w-0 group-data-pad:text-uncommon group-data-pad:rounded-full group-data-pad:bg-fg group-data-pad:shadow-key group-data-pad:ring-regular group-data-pad:ring-rim-strong">{key()}</Text>
        </Show>
      </View>
      <View class={`flex-col min-w-0 -ml-2.5 pt-1 px-3 pb-1.25 pl-4 ${packClass("Plate", "frame")}`}>
        <Text class="text-reading leading-tight font-bold tracking-md uppercase whitespace-nowrap group-data-disabled:text-fg-muted">{props.verb}</Text>
        <Show when={name() !== ""}>
          <Text class="min-w-0 max-w-50 text-2xs leading-tight font-semibold tracking-wide uppercase whitespace-nowrap truncate text-fg-muted">{name()}</Text>
        </Show>
        <Show when={props.disabled && reason() !== ""}>
          <View class="flex-row items-center gap-x-1 min-w-0 text-2xs leading-tight font-bold tracking-wide uppercase whitespace-nowrap text-danger">
            <Icon class="shrink-0 size-3" name="lock" />
            <Text class="min-w-0 max-w-50 whitespace-nowrap truncate">{reason()}</Text>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<InteractionPromptProps>> = {
  default: {"entity":0,"at":"0 0 0","height":1.2,"action":"","key":"E","pad":false,"verb":"Open","name":"Gilded Chest","pressed":false,"hold":-1,"disabled":false,"reason":"","maxDistance":6},
  pressed: {"pressed":true},
  hold: {"key":"F","verb":"Hold · Revive","name":"Brannoch","pressed":true,"hold":0.45},
  done: {"key":"F","verb":"Revive","name":"Brannoch","hold":1},
  disabled: {"verb":"Open","name":"Iron Door","disabled":true,"reason":"Needs the Warden’s key"},
  gamepad: {"key":"A","pad":true,"verb":"Talk","name":"Mira Ashgrove"},
  action: {"action":"interact","key":""},
  overflow: {"key":"Left Shoulder","verb":"Pick up and equip","name":"Blade of the Ninefold King’s Last Oath, Reforged in Ember"},
};
