import type { Texture } from "@origingame/engine";
import { For, formatNumber, Image, packClass, Show, t, Text, View, type UiEvent } from "@origingame/engine/ui";
import "./strategy.en.ogt";

export interface StrategyGroupsProps {
  /** The control groups and quick picks (the idle villagers, the army) in the game's order, keyed rows of { id, label, key, icon, count, active, alert }: label for a reader (Group 1, Idle villagers); key the group's key as the game names it (1), on a key cap; icon a picture (og.load.texture) before the count, 0 none; count what it holds; active the group selected now, lit (aria-pressed); alert one that wants the player (idle villagers waiting), its count and edge in gold (data-alert). */
  groups?: ReadonlyArray<{ id: string; label: string; key: string; icon: Texture | 0; count: number; active: boolean; alert: boolean }>;
  /** Pick. */
  onPick?: (e: UiEvent, id: string) => void;
}

/**
 * The control groups over the selection: one small plate each with its key
 * cap and the units it holds, the group selected now lit; a quick pick (the
 * idle villagers) may carry a picture, its count in gold while it wants the
 * player. Reports pick (the group's key); the game owns the groups.
 */
export function StrategyGroups(props: StrategyGroupsProps) {
  return (
    <View class={`flex-row flex-wrap items-center w-fit max-w-full min-w-0 gap-x-1.5 gap-y-1 text-fg compact:gap-x-1`} role="toolbar" label={t("strategy.groups.name")}>
      <For each={props.groups ?? []} key={(g) => g.id}>
        {(g) => (
        <View class={`flex-row items-center justify-center gap-1 group/grp flex py-0.75 px-2 compact:py-0.5 compact:px-0.5 text-fg ${packClass("Button", "plate")}`} role="button" label={t("strategy.groups.value", { label: g().label, count: g().count })} pressed={g().active} data-selected={g().active} data-alert={g().alert} onPress={(e) => props.onPick?.(e, g().id)}>
          <View class="flex-row items-center gap-x-1.5 min-w-0">
            <Show when={g().key}>
              <Text class={`shrink min-w-4.5 max-w-19.25 py-0 px-1 leading-normal text-xs font-bold text-center text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key group-data-selected/grp:from-hud-accent-hi group-data-selected/grp:to-hud-accent group-data-selected/grp:text-gold-fg whitespace-nowrap overflow-hidden`}>
                {g().key}
              </Text>
            </Show>
            <Show when={g().icon}>
              <Image class={`shrink size-6 rounded-control`} src={g().icon} />
            </Show>
            <Text class={`shrink text-base font-bold tabular-nums whitespace-nowrap group-data-alert/grp:text-gold compact:text-caption`}>
              {formatNumber(g().count)}
            </Text>
          </View>
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategyGroupsProps>> = {
  default: {"groups":[{"id":"g1","label":"Group 1","key":"1","icon":0,"count":12,"active":true,"alert":false},{"id":"g2","label":"Group 2","key":"2","icon":0,"count":6,"active":false,"alert":false},{"id":"g3","label":"Group 3","key":"3","icon":0,"count":4,"active":false,"alert":false}]},
  idle: {"groups":[{"id":"idle","label":"Idle villagers","key":".","icon":0,"count":3,"active":false,"alert":true},{"id":"army","label":"Whole army","key":"Q","icon":0,"count":38,"active":false,"alert":false},{"id":"g1","label":"Group 1","key":"1","icon":0,"count":12,"active":false,"alert":false}]},
  overflow: {"groups":[{"id":"g1","label":"Group 1","key":"1","icon":0,"count":1240,"active":true,"alert":false},{"id":"g2","label":"Group 2","key":"2","icon":0,"count":6,"active":false,"alert":false},{"id":"g3","label":"Group 3","key":"3","icon":0,"count":4,"active":false,"alert":false},{"id":"g4","label":"Group 4","key":"4","icon":0,"count":98,"active":false,"alert":false},{"id":"g5","label":"Group 5","key":"5","icon":0,"count":15,"active":false,"alert":false},{"id":"g6","label":"Group 6","key":"6","icon":0,"count":2,"active":false,"alert":false},{"id":"g7","label":"Group 7","key":"7","icon":0,"count":7,"active":false,"alert":false},{"id":"g8","label":"Group 8","key":"8","icon":0,"count":11,"active":false,"alert":false},{"id":"g9","label":"Group 9","key":"9","icon":0,"count":3,"active":false,"alert":false},{"id":"g0","label":"Group 10","key":"Left Shoulder + 0","icon":0,"count":24,"active":false,"alert":true}]},
};
