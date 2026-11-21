import { For, Icon, packClass, t, Text, View, type UiEvent } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgMenuBarProps {
  /** The menu's screens, keyed rows of { id, label } (Character, Inventory, Skills, Quests, Map). */
  tabs?: ReadonlyArray<{ id: string; label: string }>;
  /** The screen shown. */
  tab: string;
  /** Choose a screen (its key). */
  onChoose?: (e: UiEvent, id: string) => void;
  /** Previous screen. */
  onTabPrev?: (e: UiEvent) => void;
  /** Next screen. */
  onTabNext?: (e: UiEvent) => void;
  /** Close (the button, Escape or B). */
  onClose?: (e: UiEvent) => void;
}

/**
 * The menu's top bar: its screens as tabs and a close button. Reports
 * choose (a screen's key), tabPrev, tabNext and close (the button, or the
 * tabPrev, tabNext and back input actions).
 */
export function RpgMenuBar(props: RpgMenuBarProps) {
  return (
    <View class={`flex-row items-center w-full text-fg gap-x-2 compact:gap-x-0`} role="navigation" label={t("rpg.menu.name")}
      onAction={{ tabPrev: (e) => props.onTabPrev?.(e), tabNext: (e) => props.onTabNext?.(e), back: (e) => props.onClose?.(e) }}>
      <View class="relative flex-1 min-w-0 shadow-[inset_0px_-1px_0px_#ffffff24]">
        <View class="flex-row w-full">
          <For each={props.tabs ?? []} key={(c) => c.id}>
            {(c) => (
            <View class={`group/tab flex-1 min-w-0 compact:flex-none compact:basis-auto flex-row items-center justify-center min-h-11 px-4 compact:px-3 whitespace-nowrap text-base compact:text-caption font-bold tracking-wide compact:tracking-sm uppercase bg-glass text-fg-muted hover:bg-surface-hover data-selected:text-fg data-selected:bg-surface-hover data-selected:shadow-[inset_0px_-3px_0px_#38e8ff] focus-visible:shadow-[inset_0px_0px_0px_2px_#ffffff] ${packClass("Button", "plate")}`} role="tab" label={c().label} selected={c().id === props.tab} data-selected={c().id === props.tab} onPress={(e) => props.onChoose?.(e, c().id)}>
              <Text class="whitespace-nowrap overflow-hidden">
                {c().label}
              </Text>
            </View>
            )}
          </For>
        </View>
      </View>
      <View class={`shrink-0 size-11 flex-row items-center justify-center nine-plate-cut bg-glass-hi hover:bg-raised-hi text-fg ${packClass("Button", "plate")}`} role="button" label={t("rpg.menu.close")} onPress={(e) => props.onClose?.(e)}>
        <Icon class="size-1/2" name="close" />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgMenuBarProps>> = {
  default: {"tabs":[{"id":"char","label":"Character"},{"id":"inv","label":"Inventory"}],"tab":"char"},
};
