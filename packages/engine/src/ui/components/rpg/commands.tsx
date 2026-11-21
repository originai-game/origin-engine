import { For, Icon, packClass, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgCommandsProps {
  /** The commands in order, keyed rows of { id, label, icon, enabled }: label as the game words it (Attack, Skills, Spells, Items, Defend, Flee). */
  commands?: ReadonlyArray<{ id: string; label: string; icon: string; enabled: boolean }>;
  /** The command the focus rests on; it takes the primary plate. */
  selected: string;
  /** The choose event, with the command's key. */
  onChoose?: (e: UiEvent, id: string) => void;
}

/**
 * A turn's commands as a row of plates, each an icon over its label; the
 * selected one is the primary plate. Reports choose (a command's key).
 */
export function RpgCommands(props: RpgCommandsProps) {
  return (
    <View
      class={`flex-col w-full text-fg nine-panel-cut bg-[#1e2637e5] p-2 gap-y-2 ${packClass("Panel", "frame")}`}
      label={t("rpg.commands.name")}
    >
      <View class={`flex-row items-center justify-center w-full shrink-0 min-h-[2.286rem] py-[0.143rem] px-[0.857rem] shadow-[inset_0px_-1px_0px_#343d4f]`}>
        <Text
          role="heading"
          level={3}
          class={`text-[1rem] font-bold text-fg whitespace-nowrap text-stroke-3 text-stroke-bg`}
        >
          {t("rpg.commands.name")}
        </Text>
      </View>
      <View class={`grid grid-flow-col auto-cols-fr gap-1 compact:grid-flow-row compact:grid-cols-3`} role="toolbar" label={t("rpg.commands.name")}>
        <For each={props.commands ?? []} key={(c) => c.id}>
          {(c) => (
          <View
            class={`group/cmd flex h-[5.486rem] min-w-0 p-0 bg-tile bg-linear-to-b from-shade to-transparent to-55% rounded-[6px] shadow-[inset_0px_0px_0px_1px_#343d4f] hover:shadow-[inset_0px_0px_0px_1px_#8892a8] data-selected:shadow-[inset_0px_0px_0px_2px_#e8e8ed,inset_0px_3px_0px_#ffffff80] disabled:text-fg-subtle ${packClass("Button", "plate", c().id === props.selected ? "primary" : undefined)}`}
            role="button"
            label={c().label}
            selected={c().id === props.selected}
            disabled={!c().enabled}
            data-selected={c().id === props.selected}
            onPress={(e) => props.onChoose?.(e, c().id)}
          >
            <View class={`flex-col justify-center items-center gap-1 w-full h-full`}>
              <Icon class={`size-[3.429rem] text-fg group-disabled/cmd:text-fg-subtle`} name={c().icon} />
              <Text
                class={`max-w-full px-1 text-[0.929rem] font-semibold whitespace-nowrap overflow-hidden text-ellipsis text-stroke-3 text-stroke-bg`}
              >
                {c().label}
              </Text>
            </View>
          </View>
          )}
        </For>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgCommandsProps>> = {
  default: {"commands":[{"id":"attack","label":"Attack","icon":"sword","enabled":true},{"id":"flee","label":"Flee","icon":"back","enabled":false}],"selected":"attack"},
};
