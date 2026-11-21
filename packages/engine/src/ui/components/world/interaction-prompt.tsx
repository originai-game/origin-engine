import { Icon, Show, Text, View, t } from "@origingame/engine/ui";
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
  /** A hold's progress 0 to 1: a white ring round the cap on a dark disc (1 is done: the cap lights); negative draws none. Released, the game sets 0 and the ring drains in 200 ms. */
  hold?: number;
  /** It cannot be used now: the cap greys and the reason shows in red with a lock. */
  disabled?: boolean;
  /** Why it cannot (Needs the Warden's key). */
  reason?: string;
  /** Hidden past this many world units (6). */
  maxDistance?: number;
}

/**
 * What the player can do with what is in front of them: a key cap beside
 * a glass plate, the verb in condensed capitals and the object under it;
 * a hold's ring round the cap. It rises in when in range. The game decides
 * when it shows and what the input does; read only. The old SVG hold ring
 * becomes a nine-ring with the hold's share as its wipe (the 200 ms drain
 * on release is dropped); a nine-cut plate takes no rim (it paints as a
 * rectangle around the cut), so the glass's rim is dropped. The key cap's
 * action glyph has no TSX surface: the cap shows the game's key when it
 * gives one and keeps the action's name as a state otherwise, and the old
 * anchor's avoidance and max distance are dropped too. It keeps its place
 * over the plates and labels that stack (the anchor's declutter, priority
 * 1).
 */
export function InteractionPrompt(props: InteractionPromptProps) {
  const hold = () => props.hold ?? -1;
  const action = () => props.action ?? "";
  const key = () => props.key ?? "";
  const name = () => props.name ?? "";
  const reason = () => props.reason ?? "";
  const lit = () => props.pressed || hold() >= 1;
  const point = () => ((props.at ?? "0 0 0").split(/\s+/).map(Number) as [number, number, number]);
  return (
    <View
      anchor={props.entity ? (props.entity as Entity) : point()}
      lift={[0, props.height ?? 0, 0]}
      declutter="stack"
      priority={1}
      class="-translate-x-1/2 -translate-y-full group flex-row items-center animate-pop text-fg"
      role="status"
      label={`${props.verb}${name() !== "" ? ` ${name()}` : ""}`}
      description={props.disabled && reason() !== "" ? reason() : ""}
      data-pad={props.pad}
      data-disabled={props.disabled}
    >
      <View class="relative z-1 shrink-0 flex-row items-center justify-center size-[2.571rem]">
        <Show when={hold() >= 0}>
          <View class="absolute inset-0 rounded-full bg-fg/78" />
          <View class="absolute inset-0 nine-ring bg-sheen" wipe={hold() >= 1 ? 1 : hold()} role="meter" label={t("world.prompt.hold")} valueMin={0} valueMax={1} valueNow={hold() >= 1 ? 1 : hold()} />
        </Show>
        <Text class="z-1 min-w-[1.857rem] h-[1.857rem] px-[0.357rem] text-center text-sm leading-[1.857rem] font-bold whitespace-nowrap text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-[0.286rem] shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] group-data-pressed:translate-y-[2px] group-data-pressed:bg-linear-to-b group-data-pressed:from-hud-accent-hi group-data-pressed:to-hud-accent group-data-pressed:shadow-[0px_0px_1rem_#38e8ff,inset_0px_-0.143rem_0px_#0000002e] group-data-disabled:text-fg-subtle group-data-disabled:bg-linear-to-b group-data-disabled:from-fg-subtle group-data-disabled:to-fg-subtle group-data-pad:px-0 group-data-pad:w-[1.857rem] group-data-pad:min-w-0 group-data-pad:text-uncommon group-data-pad:rounded-full group-data-pad:bg-fg/95 group-data-pad:shadow-[0px_0px_0px_1.5px_#ffffff99,0px_2px_0px_#00000099]" data-action={action() || undefined}>{key()}</Text>
      </View>
      <View class="flex-col min-w-0 -ml-[0.714rem] pt-[0.286rem] px-[0.857rem] pb-[0.357rem] pl-[1.143rem] nine-plate-cut bg-plate-hi">
        <Text class="text-reading leading-[1.15] font-bold tracking-[0.07em] uppercase whitespace-nowrap group-data-disabled:text-fg-muted">{props.verb}</Text>
        <Show when={name() !== ""}>
          <Text class="min-w-0 max-w-[14.286rem] text-[0.75rem] leading-[1.2] font-semibold tracking-[0.13em] uppercase whitespace-nowrap truncate text-fg-muted">{name()}</Text>
        </Show>
        <Show when={props.disabled && reason() !== ""}>
          <View class="flex-row items-center gap-x-[0.286rem] min-w-0 text-[0.75rem] leading-[1.2] font-bold tracking-[0.13em] uppercase whitespace-nowrap text-danger">
            <Icon class="shrink-0 size-3" name="lock" />
            <Text class="min-w-0 max-w-[14.286rem] whitespace-nowrap truncate">{reason()}</Text>
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
