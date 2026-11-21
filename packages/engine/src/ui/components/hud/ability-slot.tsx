import { Icon, Show, Text, View, formatDuration, formatNumber, t, type UiEvent } from "@origingame/engine/ui";
import "./ability-slot.en.ogt";

export interface AbilitySlotProps {
  /** An icon of the atlas, drawn large on a gradient. */
  icon: string;
  /** What a screen reader calls it (Fireball). */
  name: string;
  /** The action whose glyph the hotkey cap shows on the device used last (it follows a rebinding); empty is none. */
  action: string;
  /** A label for the cap instead of the action's glyph (Q, LB); empty shows the action's. */
  hotkey: string;
  /** Seconds of cooldown left; 0 is ready. */
  cooldown: number;
  /** The cooldown's length; the sweep shows cooldown over it. Default 1. */
  cooldownMax?: number;
  /** Charges ready, in a corner badge when maxCharges is over 1. */
  charges: number;
  /** Charges it holds; 0 or 1 is none. */
  maxCharges: number;
  /** How many are left (a quick item: potions, scrolls), at the bottom's end; 0 shows none. */
  count: number;
  /** Its cost (40) at the bottom left while ready; empty is none. */
  cost: string;
  /** Not enough resource: the art tinted red and the cost in danger. */
  unaffordable?: boolean;
  /** The target is out of range: the art tinted red. */
  outOfRange?: boolean;
  /** Toggled on or channelling: a breathing glow and a primary edge. */
  active?: boolean;
  /** Not learnt yet: dark art under a lock. */
  locked?: boolean;
  /** Cannot be used now: the art washed out under a red slash. */
  disabled?: boolean;
  /** An ultimate's charge 0 to 1: larger, a gold ring filling round it and a percent; negative is a plain slot. Default -1. */
  ultimate?: number;
  /** Ranks learnt (pips right under it). */
  level: number;
  /** It fills the place it is given (a strip's), rather than standing --ability-slot-size wide. */
  fit?: boolean;
  /** Ranks it has; 0 draws no pips. */
  maxLevel: number;
  /** Use. */
  onUse?: (e: UiEvent) => void;
}

export function AbilitySlot(props: AbilitySlotProps) {
  return (
    <View class={`group/abilityslot w-[3.429rem] max-w-full items-center p-0 gap-0.5 data-fit:w-full`} data-ultimate={(props.ultimate ?? -1) >= 0} data-fit={props.fit}>
      <View class={`group/slot border-rim-strong bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-[0px_0px_0px_1px_#00000073,inset_0px_1px_0px_0px_#ffffff14,0px_6px_18px_-6px_#00000099] transition duration-317 w-full aspect-square p-0 rounded-button border overflow-hidden data-selected:border-accent data-done:border-success data-active:border-accent data-active:shadow-[0px_0px_12px_#e8e8ed] data-full:border-gold data-shape-round:rounded-capsule data-shape-square:rounded-button active:bg-none active:bg-tile-pressed active:scale-96 hover:shadow-[inset_0px_0px_0px_1px_#8892a8,inset_0px_1px_0px_0px_#ffffff29,0px_6px_14px_-4px_#00000099] data-empty:shadow-sunk data-locked:shadow-sunk data-locked:scale-100 disabled:shadow-sunk disabled:scale-100 data-down:opacity-45 data-play-pulse:animate-pulse`} role="button" onPress={(e) => props.onUse?.(e)} label={props.name} valueText={props.locked ? t("component.ability-slot.locked") : props.disabled ? t("component.ability-slot.unavailable") : props.cooldown > 0 ? formatNumber(props.cooldown) : t("component.ability-slot.ready")} disabled={props.locked || props.disabled} data-play-pulse={(props.active || (props.ultimate ?? -1) >= 1 ? 'pulse' : null) === "pulse"} data-cooling={props.cooldown > 0} data-tinted={props.unaffordable || props.outOfRange} data-full={(props.ultimate ?? -1) >= 1} data-active={props.active || (props.ultimate ?? -1) >= 1} data-locked={props.locked} data-shape-round={((props.ultimate ?? -1) >= 0 ? 'round' : 'square') === "round"} data-shape-square={((props.ultimate ?? -1) >= 0 ? 'round' : 'square') === "square"}>
        <View class="gap-2 size-full">
          <View class={`absolute bg-tile shadow-sunk justify-center items-center inset-0.5 gap-2 rounded-[7px] group-data-ultimate/abilityslot:rounded-capsule group-data-tinted/slot:bg-linear-to-t group-data-tinted/slot:from-danger/45 group-data-tinted/slot:to-transparent group-data-tinted/slot:to-75% group-data-tinted/slot:bg-tile`}>
            <Icon class="text-fg text-shadow-2xs size-[66%] group-data-ultimate/abilityslot:size-1/2 group-disabled/slot:text-fg-subtle" name={props.icon} />
          </View>
          <Show when={(props.ultimate ?? -1) >= 0}>
            <View class="absolute inset-x-0 bottom-0 bg-gold/35 rounded-b-capsule" style={{ height: `${Math.min(1, props.ultimate ?? 0) * 100}%` }} />
          </Show>
          <Show when={props.cooldown > 0}>
            <View class="absolute inset-x-0 top-0 bg-black/70" style={{ height: `${Math.min(1, props.cooldown / (props.cooldownMax ?? 1)) * 100}%` }} />
            <View class="items-center justify-center max-w-105 absolute mx-auto inset-0 gap-2">
              <Text class={`tabular-nums text-fg text-shadow-2xs text-[20.64px]`}>
                {props.cooldown < 10 ? formatNumber(props.cooldown) : formatDuration(props.cooldown)}
              </Text>
            </View>
          </Show>
          <Show when={(props.ultimate ?? -1) >= 0 && (props.ultimate ?? -1) < 1}>
            <View class="flex-row items-center absolute bottom-0 justify-center inset-x-0 gap-2">
              <Text class={`text-caption tabular-nums bg-black/69 shadow-[inset_0px_0px_0px_1px_#4e586c] text-gold transition duration-317 text-shadow-2xs py-0 px-2 rounded-capsule shrink`}>
                {`${formatNumber((props.ultimate ?? 0) * 100, { decimals: 0 })}%`}
              </Text>
            </View>
          </Show>
          <Show when={props.disabled}>
            <View class="absolute inset-0 items-center justify-center">
              <View class="w-[110%] h-1.25 shrink-0 bg-danger rounded-capsule -rotate-45" />
            </View>
          </Show>
          <Show when={props.locked}>
            <View class="items-center justify-center max-w-105 absolute mx-auto inset-0 gap-2">
              <Icon class="text-fg text-shadow-2xs size-1/2 group-disabled/slot:text-fg-subtle" name="lock" />
            </View>
          </Show>
          <Show when={props.cost && props.cooldown <= 0 && !props.locked}>
            <Show when={(props.ultimate ?? -1) >= 0} fallback={
                <View class="flex-row items-center absolute bottom-0.5 left-0.5 gap-2">
                  <Text class={`text-caption tabular-nums bg-surface-raised shadow-[inset_0px_0px_0px_1px_#4e586c] text-mana transition duration-317 text-shadow-2xs py-0 px-2 rounded-capsule data-short:text-danger shrink`} data-short={props.unaffordable}>
                    {props.cost}
                  </Text>
                </View>
            }>
              <View class="flex-row items-center absolute top-0 justify-center inset-x-0 gap-2">
                <Text class={`text-caption tabular-nums bg-surface-raised shadow-[inset_0px_0px_0px_1px_#4e586c] text-mana transition duration-317 text-shadow-2xs py-0 px-2 rounded-capsule data-short:text-danger shrink`} data-short={props.unaffordable}>
                  {props.cost}
                </Text>
              </View>
            </Show>
          </Show>
          <Show when={props.count > 0}>
            <Text class={`absolute right-1.5 bottom-1.5 tabular-nums text-fg text-shadow-2xs text-[14.88px]`}>
              {formatNumber(props.count)}
            </Text>
          </Show>
          <Show when={props.maxCharges > 1 && props.cooldown <= 0 && !props.locked}>
            <Text class={`absolute top-0.5 right-0.5 text-caption tabular-nums bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c] text-fg transition duration-317 min-w-4 text-shadow-2xs py-0 px-2 rounded-capsule`}>
              {formatNumber(props.charges)}
            </Text>
          </Show>
        </View>
      </View>
      <Show when={props.maxLevel > 0}>
        <View class={`bg-tile shadow-sunk h-1 rounded-capsule`} style={{ width: `${((12 * (props.maxLevel)) + -4)}px` }}>
          <View class={`bg-gold h-1`} style={{ width: `${((12 * (props.level)) + -4)}px` }} />
        </View>
      </Show>
      <Show when={props.hotkey || props.action}>
        <Text class={`text-caption tabular-nums bg-surface-raised shadow-[inset_0px_0px_0px_1px_#4e586c] text-fg transition duration-317 max-w-full whitespace-nowrap text-shadow-2xs py-0 px-2 rounded-capsule overflow-hidden`} glyph={props.hotkey ? undefined : props.action}>
          {props.hotkey}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<AbilitySlotProps>> = {
  default: {"icon":"fire","name":"Fireball","action":"","hotkey":"Q","cooldown":4.2,"cooldownMax":8,"charges":2,"maxCharges":3,"count":0,"cost":"40","unaffordable":false,"outOfRange":false,"active":false,"locked":false,"disabled":false,"ultimate":-1,"level":2,"maxLevel":3},
};
