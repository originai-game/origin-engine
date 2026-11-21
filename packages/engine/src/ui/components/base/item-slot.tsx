import { Icon, Show, Text, View, formatDuration, formatNumber, packClass, t, type UiEvent } from "@origingame/engine/ui";
import { Spinner } from "./spinner";
import "./item-slot.en.ogt";

export interface ItemSlotProps {
  /** The item name, also its accessible name and tooltip. Empty slots use the localized empty-slot name. */
  name?: string;
  /** An atlas icon. Empty is no item art. */
  icon?: string;
  /** The quantity; a badge appears above 1. Default 1. */
  count?: number;
  /** The item's rarity: common, uncommon, rare, epic, legendary or mythic; a pack selects data-rarity. Default "common". */
  rarity?: string;
  /** Seconds remaining, supplied by the game. A cooling item can still be selected for inspection. Default 0. */
  cooldown?: number;
  /** The cooldown's full duration; the overlay shows the remaining fraction. Default 1. */
  cooldownMax?: number;
  /** A shortcut label; empty hides it. */
  keyLabel?: string;
  /** The game-selected slot, exposed as aria-pressed. Default false. */
  selected?: boolean;
  /** Unavailable; selection is disabled. Default false. */
  disabled?: boolean;
  /** Locked; shown in words and selection is disabled. Default false. */
  locked?: boolean;
  /** Waiting for item data; selection is disabled. Default false. */
  loading?: boolean;
  /** An empty slot. It can be selected so the game can offer an equip action. Default false. */
  empty?: boolean;
  /** Show the wrapping item name below the slot. Default false. */
  showName?: boolean;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

/**
 * Reports choose. In a keyed collection the intent keeps its enclosing
 * row key; otherwise the game's stable component instance identifies it.
 */
export function ItemSlot(props: ItemSlotProps) {
  return (
    <View class={`text-fg w-24 p-0 gap-1 disabled:text-fg-subtle`}>
      <View class={`flex-row items-center justify-center gap-1 group/slot border-rim-soft bg-tile shadow-sunk text-fg transition duration-317 w-full h-24 min-h-11 p-2 rounded-button border overflow-hidden data-selected:border-accent data-done:border-success active:border-accent active:shadow-glow active:shadow-accent active:bg-none active:bg-tile-pressed active:scale-96 hover:shadow-plate hover:inset-ring-1 hover:inset-ring-well-rim data-empty:shadow-sunk data-locked:shadow-sunk data-locked:scale-100 disabled:shadow-sunk disabled:text-fg-subtle disabled:scale-100 data-down:opacity-45 ${packClass("Slot", "frame")}`} role="button" data-rarity-uncommon={props.rarity === "uncommon"} data-rarity-rare={props.rarity === "rare"} data-rarity-epic={props.rarity === "epic"} data-rarity-legendary={props.rarity === "legendary"} data-rarity-mythic={props.rarity === "mythic"} data-empty={(props.empty ?? false)} label={t("base.item-slot.label", { name: props.name || t("base.item-slot.empty"), state: (props.loading ?? false) ? t("base.item-slot.loading") : (props.locked ?? false) ? t("base.item-slot.locked") : (props.disabled ?? false) ? t("base.item-slot.disabled") : (props.cooldown ?? 0) > 0 ? t("base.item-slot.cooldown", { time: formatDuration((props.cooldown ?? 0)) }) : (props.empty ?? false) ? t("base.item-slot.empty") : t("base.item-slot.count", { count: (props.count ?? 1) }) })} pressed={(props.selected ?? false)} busy={(props.loading ?? false)} disabled={(props.disabled ?? false) || (props.locked ?? false) || (props.loading ?? false)} onPress={(e) => props.onChoose?.(e)} data-cooling={(props.cooldown ?? 0) > 0 && !(props.empty ?? false) && !(props.loading ?? false)}>
        <Show when={!(props.empty ?? false)}>
          <View class="absolute inset-0 pointer-events-none nine-socket-rim bg-common group-data-rarity-uncommon/slot:bg-uncommon group-data-rarity-rare/slot:bg-rare group-data-rarity-epic/slot:bg-epic group-data-rarity-legendary/slot:bg-legendary group-data-rarity-mythic/slot:bg-mythic" />
        </Show>
        <Show when={props.icon && !(props.empty ?? false) && !(props.loading ?? false)}>
          <Icon class={`size-16/25 group-data-cooling/slot:opacity-12`} name={props.icon ?? ""} />
        </Show>
        <Show when={((props.empty ?? false) || !props.icon) && !(props.loading ?? false)}>
          <Icon class={`text-fg-muted size-9/25`} name="plus" />
        </Show>
        <Show when={(props.loading ?? false)}>
          <View class={`grow shrink basis-0 justify-center items-center absolute inset-0 gap-2`}>
            <Spinner name={t("base.item-slot.loading")} />
          </View>
        </Show>
        <Show when={(props.cooldown ?? 0) > 0 && !(props.empty ?? false) && !(props.loading ?? false)}>
          <View class={`bg-shade-deep absolute bottom-0 pointer-events-none inset-x-0`} style={{ height: `${Math.min(Math.max((props.cooldownMax ?? 1) > 0 ? (props.cooldown ?? 0) / (props.cooldownMax ?? 1) : 0, 0), 1) * 100}%` }} />
          <View class={`items-center justify-center max-w-105 absolute text-lg font-bold text-fg mx-auto inset-0 gap-2`}>
            <Text class="tabular-nums">{formatDuration((props.cooldown ?? 0))}</Text>
          </View>
        </Show>
        <Show when={(props.count ?? 1) > 1 && !(props.empty ?? false) && !(props.loading ?? false)}>
          <View class={`text-caption bg-raised-lo inset-ring-1 inset-ring-rim text-fg transition duration-317 absolute right-0.5 bottom-0.5 py-0 px-1 rounded-capsule`}>
            <Text class="tabular-nums">{formatNumber((props.count ?? 1))}</Text>
          </View>
        </Show>
        <Show when={props.keyLabel}>
          <View class={`text-caption bg-raised-lo inset-ring-1 inset-ring-rim text-fg transition duration-317 absolute left-0.5 top-0.5 py-0 px-1 rounded-capsule`}>
            <Text class="tabular-nums">{props.keyLabel}</Text>
          </View>
        </Show>
        <Show when={((props.locked ?? false) || (props.disabled ?? false)) && !(props.loading ?? false)}>
          <View class={`text-caption bg-raised-lo inset-ring-1 inset-ring-rim text-fg transition duration-317 absolute bottom-0.5 text-center p-0.5 inset-x-0.5 rounded-capsule overflow-hidden`}>
            <Text class="tabular-nums whitespace-nowrap text-ellipsis">{(props.locked ?? false) ? t("base.item-slot.locked") : t("base.item-slot.disabled")}</Text>
          </View>
        </Show>
      </View>
      <Show when={(props.showName ?? false)}>
        <Text class={`text-caption text-fg-muted text-center`}>
          {props.name || ((props.loading ?? false) ? t("base.item-slot.loading") : t("base.item-slot.empty"))}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ItemSlotProps>> = {
  default: {"name":"Healing draught","icon":"potion","count":12,"rarity":"rare","cooldown":0,"cooldownMax":8,"keyLabel":"Q","selected":false,"disabled":false,"locked":false,"loading":false,"empty":false,"showName":true},
  selected: {"name":"Storm crystal","icon":"gem","count":3,"rarity":"epic","cooldown":3,"cooldownMax":8,"keyLabel":"2","selected":true,"disabled":false,"locked":false,"loading":false,"empty":false,"showName":true},
  locked: {"name":"Sealed relic","icon":"key","rarity":"legendary","locked":true,"showName":true},
  disabled: {"name":"Spent potion","icon":"potion","disabled":true,"showName":true},
  loading: {"loading":true,"showName":true},
  empty: {"empty":true,"showName":true},
};
