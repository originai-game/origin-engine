import type { Texture } from "@origingame/engine";
import { For, Icon, Image, packClass, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgCharacterProps {
  /** A heading band over the figure (Equipment); empty shows the name header instead. */
  title: string;
  /** The character's name; with an empty title it heads the sheet beside the portrait. */
  name: string;
  /** The class and level line as the game words it (Rogue · Level 12). */
  line: string;
  /** A head-and-shoulders texture (og.load.texture); 0 hides it. */
  portrait: Texture | 0;
  /** A full-figure texture shown between the slot columns; 0 leaves the ground. */
  figure: Texture | 0;
  /** The equipment slots, keyed rows of { id, side, label, icon, rarity }: side is start or end (the column it stands in, mirrored right to left); an empty icon is an empty slot showing its label. */
  slots?: ReadonlyArray<{ id: string; side: string; label: string; icon: string; rarity: string }>;
  /** The selected slot's id; empty is none. */
  selected: string;
  /** The attributes, keyed rows of { id, group, icon, label, value }: group a or b, the two columns; empty hides them (a page with RpgStats beside it). */
  attributes?: ReadonlyArray<{ id: string; group: string; icon: string; label: string; value: string }>;
  /** Its place is narrow (under about 300 px): the attributes (their split goes) tighten. The place decides, not the screen. */
  compact: boolean;
  /** A wide but short place (a laptop window): smaller slots and tighter attributes. */
  dense: boolean;
  /** The choose event, with the slot's key. */
  onChoose?: (e: UiEvent, id: string) => void;
  /** Unequip, from the focused slot (X or pad Y, the details action), with the slot's key. */
  onUnequip?: (e: UiEvent, id: string) => void;
}

/**
 * A character sheet: a portrait with name and class, two columns of
 * equipment slots either side of the full figure, and the attributes in two
 * columns beneath. The game owns what fits where; the sheet reports choose
 * (a slot's key) and unequip (X or pad Y on the focused slot, its key).
 */
export function RpgCharacter(props: RpgCharacterProps) {
  return (
    <View
      class={`group/sheet flex-col text-fg p-4 nine-panel-cut bg-glass/96 compact:p-2 ${packClass("Panel", "frame")}`}
      label={props.title !== "" ? props.title : props.name}
      data-narrow={props.compact}
      data-dense={props.dense}
    >
      <Show when={props.title !== ""} fallback={
          <View class={`flex-row items-start w-full gap-x-[0.857rem] pb-2 shadow-[inset_0px_-1px_0px_#ffffff24]`}>
            <Show when={props.portrait !== 0}>
              <Image
                class={`size-[4.571rem] shrink-0 rounded-[0.571rem] compact:size-[3.429rem]`}
                src={props.portrait}
                label=""
              />
            </Show>
            <View class={`flex-col items-start min-w-0 gap-y-[0.143rem]`}>
              <Text
                class={`max-w-full text-3xl leading-[1.05] font-bold tracking-[0.14em] uppercase text-fg compact:text-xl`}
              >
                {props.name}
              </Text>
              <Text
                class={`max-w-full text-xs leading-[1.3] font-semibold tracking-[0.14em] uppercase text-fg/78`}
              >
                {props.line}
              </Text>
              <View class={`w-full h-px bg-white/14`} role="separator" />
            </View>
          </View>
      }>
        <View class={`flex-row items-center justify-center w-full shrink-0`}>
          <View class={`w-[3.429rem] h-4 shrink-0`} />
          <Text
            class={`min-w-0 text-[1.714rem] leading-[1.1] font-bold tracking-[0.14em] uppercase text-fg text-center whitespace-normal`}
          >
            {props.title}
          </Text>
          <View class={`w-[3.429rem] h-4 shrink-0`} />
        </View>
      </Show>
      <View class={`relative flex-row justify-between w-full px-1 py-2 bg-linear-to-b from-hud-accent/10 to-transparent`}>
        <Show when={props.figure !== 0}>
          <View class={`absolute left-[18%] top-0 bottom-0 w-[64%]`}>
            <Image class={`size-full`} src={props.figure} label="" />
          </View>
        </Show>
        <View class={`relative flex-col justify-between self-stretch gap-y-1`} role="group">
          <For each={props.slots ?? []} key={(s) => s.id}>
            {(s) => (
            <Show when={s().side === "start"}>
              <View
                class={`flex-row items-center justify-center gap-1 group/cell flex size-[5.429rem] p-1 nine-plate-cut bg-tile text-fg data-empty:bg-tile data-empty:text-fg-muted compact:size-[3.857rem] compact:p-[0.143rem] group-data-dense/sheet:size-[3.429rem] group-data-dense/sheet:p-[0.143rem] ${packClass("Button", "plate")}`}
                role="button"
                label={s().label}
                selected={s().id === props.selected}
                data-empty={!s().icon}
                data-selected={s().id === props.selected}
                data-rarity-common={!!s().icon && s().rarity === "common"}
                data-rarity-uncommon={!!s().icon && s().rarity === "uncommon"}
                data-rarity-rare={!!s().icon && s().rarity === "rare"}
                data-rarity-epic={!!s().icon && s().rarity === "epic"}
                data-rarity-legendary={!!s().icon && s().rarity === "legendary"}
                data-rarity-mythic={!!s().icon && s().rarity === "mythic"}
                onPress={(e) => props.onChoose?.(e, s().id)}
                onAction={{ details: (e) => props.onUnequip?.(e, s().id) }}
              >
                <View class={`flex-col justify-center items-center w-full h-full`}>
                  <Icon
                    class={`size-full group-data-empty/cell:size-1/2`}
                    name={s().icon || "plus"}
                  />
                </View>
              </View>
            </Show>
            )}
          </For>
        </View>
        <View class={`relative flex-col justify-between self-stretch gap-y-1`} role="group">
          <For each={props.slots ?? []} key={(s) => s.id}>
            {(s) => (
            <Show when={s().side === "end"}>
              <View
                class={`flex-row items-center justify-center gap-1 group/cell flex size-[5.429rem] p-1 nine-plate-cut bg-tile text-fg data-empty:bg-tile data-empty:text-fg-muted compact:size-[3.857rem] compact:p-[0.143rem] group-data-dense/sheet:size-[3.429rem] group-data-dense/sheet:p-[0.143rem] ${packClass("Button", "plate")}`}
                role="button"
                label={s().label}
                selected={s().id === props.selected}
                data-empty={!s().icon}
                data-selected={s().id === props.selected}
                data-rarity-common={!!s().icon && s().rarity === "common"}
                data-rarity-uncommon={!!s().icon && s().rarity === "uncommon"}
                data-rarity-rare={!!s().icon && s().rarity === "rare"}
                data-rarity-epic={!!s().icon && s().rarity === "epic"}
                data-rarity-legendary={!!s().icon && s().rarity === "legendary"}
                data-rarity-mythic={!!s().icon && s().rarity === "mythic"}
                onPress={(e) => props.onChoose?.(e, s().id)}
                onAction={{ details: (e) => props.onUnequip?.(e, s().id) }}
              >
                <View class={`flex-col justify-center items-center w-full h-full`}>
                  <Icon
                    class={`size-full group-data-empty/cell:size-1/2`}
                    name={s().icon || "plus"}
                  />
                </View>
              </View>
            </Show>
            )}
          </For>
        </View>
      </View>
      <Show when={(props.attributes ?? []).length}>
        <View
          class={`flex flex-wrap items-stretch w-full gap-x-[0.857rem] gap-y-1 pt-2 shadow-[inset_0px_1px_0px_#ffffff24] group-data-dense/sheet:gap-x-2`}
          role="table"
          label={t("rpg.character.attributes")}
        >
          <View class={`flex-col self-stretch min-w-0 grow basis-0`}>
            <For each={props.attributes ?? []} key={(a) => a.id}>
              {(a) => (
              <Show when={a().group === "a"}>
                <View class={`flex-row items-center gap-2 py-1 shadow-[inset_0px_-1px_0px_#ffffff24]`} role="row">
                  <Icon class={`text-fg/78 size-[1.143rem] shrink-0 group-data-dense/sheet:hidden`} name={a().icon} />
                  <Text class={`min-w-0 grow text-caption font-semibold text-fg/78`} role="rowheader">
                    {a().label}
                  </Text>
                  <Text class={`shrink-0 whitespace-nowrap text-lg font-bold tabular-nums`} role="cell">
                    {a().value}
                  </Text>
                </View>
              </Show>
              )}
            </For>
          </View>
          <View class={`w-px self-stretch group-data-narrow/sheet:hidden`} />
          <View class={`flex-col self-stretch min-w-0 grow basis-0`}>
            <For each={props.attributes ?? []} key={(a) => a.id}>
              {(a) => (
              <Show when={a().group === "b"}>
                <View class={`flex-row items-center gap-2 py-1 shadow-[inset_0px_-1px_0px_#ffffff24]`} role="row">
                  <Icon class={`text-fg/78 size-[1.143rem] shrink-0 group-data-dense/sheet:hidden`} name={a().icon} />
                  <Text class={`min-w-0 grow text-caption font-semibold text-fg/78`} role="rowheader">
                    {a().label}
                  </Text>
                  <Text class={`shrink-0 whitespace-nowrap text-lg font-bold tabular-nums`} role="cell">
                    {a().value}
                  </Text>
                </View>
              </Show>
              )}
            </For>
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgCharacterProps>> = {
  default: {"title":"","name":"Brenna Vale","line":"Ranger · Level 12","portrait":0,"figure":0,"slots":[{"id":"hand","side":"start","label":"Weapon","icon":"sword","rarity":"legendary"},{"id":"neck","side":"end","label":"Amulet","icon":"","rarity":"common"}],"selected":"hand","attributes":[{"id":"vit","group":"a","icon":"heart","label":"Vitality","value":"142"},{"id":"str","group":"b","icon":"sword","label":"Strength","value":"14"}],"compact":false,"dense":false},
};
