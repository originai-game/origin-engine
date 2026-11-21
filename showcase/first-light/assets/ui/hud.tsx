import { Icon, Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";
import { AbilityBar } from "@origingame/engine/ui/components";
import { RpgQuestTracker } from "@origingame/engine/ui/components";

export interface HudProps {
  /** The HUD is drawn; H turns it off. */
  shown?: boolean;
  /** The time of day, as the clock shows it (06:54). */
  clock: string;
  /** The season and the time of day's name (Summer · Dawn). */
  day: string;
  /** The weather, in a word (Clear). */
  sky: string;
  /** The day's task list's name. */
  quest: string;
  /** Its tasks, keyed rows of { id, text, done }. */
  steps: ReadonlyArray<{ id: string; text: string; done: boolean }>;
  /** The hotbar's slots, as AbilityBar takes them. */
  slots: ReadonlyArray<{ id: string; icon: string; name: string; action: string; hotkey: string; cooldown: number; cooldownMax: number; cost: string; unaffordable: boolean; locked: boolean; active: boolean; count: number }>;
  /** A slot was pressed: its id. */
  onUse?: (e: UiEvent, id: string) => void;
}

/**
 * The cosy HUD: the clock top left, the day's tasks top right, the hotbar
 * along the bottom, each 16 px inside the HUD insets. The clock and the
 * tasks share the top: side by side where they fit, the tasks under the
 * clock where they do not (a phone held upright). H hides it.
 */
export function Hud(props: HudProps) {
  return (
    <Show when={props.shown}>
      <View class="absolute inset-0 justify-between pt-[calc(env(hud-inset-top)+16px)] pr-[calc(env(hud-inset-right)+16px)] pb-[calc(env(hud-inset-bottom)+16px)] pl-[calc(env(hud-inset-left)+16px)]">
        <View class="flex-row flex-wrap items-start gap-3">
          <View class={`bg-surface nine-plate-cut py-2 px-4 gap-0.5 ${packClass("Plate", "frame")}`} role="status">
            <View class="flex-row items-center gap-1">
              <Icon class="size-4 text-fg" name="clock" />
              <Text class="text-display-sm font-bold font-mono text-fg whitespace-nowrap">{props.clock}</Text>
            </View>
            <Text class="text-body text-fg whitespace-nowrap">{props.day}</Text>
            <Text class="text-caption text-fg-muted whitespace-nowrap">{props.sky}</Text>
          </View>
          <View class="ml-auto shrink basis-70 w-70">
            <RpgQuestTracker label="" name={props.quest} steps={props.steps} distance="" />
          </View>
        </View>
        <View class="flex-row justify-center">
          <AbilityBar slots={props.slots} column={false} onUse={(e, id) => props.onUse?.(e, id)} />
        </View>
      </View>
    </Show>
  );
}

export const fixtures: Record<string, Partial<HudProps>> = {
  default: {"shown":true,"clock":"06:54","day":"Summer · Dawn","sky":"Clear","quest":"Morning rounds","steps":[{"id":"a","text":"Watch the sun come up","done":true},{"id":"b","text":"Count the flock","done":false}],"slots":[{"id":"weather","icon":"drop","name":"Weather","action":"weather","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":false,"count":0}]},
};
