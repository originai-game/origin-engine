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
      class={`flex-col w-full p-2 gap-y-2 ${packClass("Panel", "frame")}`}
      label={t("rpg.commands.name")}
    >
      <View class={`flex-row items-center justify-center w-full shrink-0 min-h-8 py-0.5 px-3 inset-shadow-underline inset-shadow-rim-soft`}>
        <Text
          role="heading"
          level={3}
          class={`text-sm font-bold text-fg whitespace-nowrap text-stroke-3 text-stroke-bg`}
        >
          {t("rpg.commands.name")}
        </Text>
      </View>
      <View class={`grid grid-flow-col auto-cols-fr gap-1 compact:grid-flow-row compact:grid-cols-3`} role="toolbar" label={t("rpg.commands.name")}>
        <For each={props.commands ?? []} key={(c) => c.id}>
          {(c) => (
          <View
            class={`flex-row items-center justify-center gap-1 group/cmd flex h-19.25 min-w-0 p-0 hover:inset-ring-hairline hover:inset-ring-well-rim data-selected:inset-shadow-lit data-selected:inset-ring-regular data-selected:inset-ring-accent disabled:text-fg-subtle ${packClass("Button", "plate", c().id === props.selected ? "primary" : undefined)}`}
            role="button"
            label={c().label}
            selected={c().id === props.selected}
            disabled={!c().enabled}
            data-selected={c().id === props.selected}
            onPress={(e) => props.onChoose?.(e, c().id)}
          >
            <View class={`flex-col justify-center items-center gap-1 w-full h-full`}>
              <Icon class={`size-12 text-fg group-disabled/cmd:text-fg-subtle`} name={c().icon} />
              <Text
                class={`max-w-full px-1 text-caption font-semibold whitespace-nowrap overflow-hidden text-ellipsis text-stroke-3 text-stroke-bg`}
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
