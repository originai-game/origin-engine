import { For, View, t, type UiEvent } from "@origingame/engine/ui";
import { AbilitySlot } from "./ability-slot";
import "./ability-bar.en.ogt";

export interface AbilityBarProps {
  /** The slots, in order: each { id, icon, name, action, hotkey, cooldown, cooldownMax, cost, unaffordable, locked, active, count }, cooldown in seconds (0: ready); count above 0 shows how many are left (a quick item: potions, scrolls). A slot's full set of states is the AbilitySlot component. */
  slots: ReadonlyArray<{ id: string; icon: string; name: string; action: string; hotkey: string; cooldown: number; cooldownMax: number; cost: string; unaffordable: boolean; locked: boolean; active: boolean; count?: number }>;
  /** The slots run top to bottom; false is a row. */
  column?: boolean;
  /** Use. */
  onUse?: (e: UiEvent, id: string) => void;
}

/**
 * A strip of AbilitySlot blocks, and nothing else: the slot draws
 * itself, the strip places them. Each place is --ability-bar-slot-size
 * wide and, where the strip is given less, shrinks alike down to
 * --ability-bar-slot-min; under that the strip breaks into lines, so it
 * never overflows. A game that wants another arrangement (two rows,
 * groups, an ultimate at the end) lays AbilitySlot out itself in the
 * same way.
 */
export function AbilityBar(props: AbilityBarProps) {
  return (
    <View class={`flex-row flex-wrap items-start justify-center p-0 m-0 gap-1 data-column:flex-col data-column:flex-nowrap`} role="toolbar" label={t("component.ability-bar.name")} data-column={props.column}>
      <For each={props.slots}>
        {(s) => (
        <View class="shrink basis-16 w-16 min-w-11 items-center gap-2">
          <AbilitySlot icon={s().icon} name={s().name} action={s().action} hotkey={s().hotkey} cooldown={s().cooldown} cooldownMax={s().cooldownMax} charges={0} maxCharges={0} count={s().count ?? 0} cost={s().cost} unaffordable={s().unaffordable} outOfRange={false} active={s().active} locked={s().locked} disabled={false} ultimate={-1} level={0} maxLevel={0} fit={true} onUse={(e) => props.onUse?.(e, s().id)} />
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<AbilityBarProps>> = {
  default: {"column":false,"slots":[{"id":"q","icon":"sword","name":"Cleave","action":"","hotkey":"Q","cooldown":0,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"w","icon":"shield","name":"Guard","action":"","hotkey":"W","cooldown":3.4,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false},{"id":"e","icon":"bolt","name":"Dash","action":"","hotkey":"E","cooldown":0,"cooldownMax":5,"cost":"60","unaffordable":true,"locked":false,"active":false},{"id":"r","icon":"fire","name":"Inferno","action":"","hotkey":"R","cooldown":0,"cooldownMax":60,"cost":"","unaffordable":false,"locked":true,"active":false},{"id":"t","icon":"potion","name":"Elixir","action":"","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":true,"count":9}]},
};
