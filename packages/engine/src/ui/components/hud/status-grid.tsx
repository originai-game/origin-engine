import { For, Icon, Show, Text, View, formatDuration, formatNumber, t } from "@origingame/engine/ui";
import "./status-grid.en.ogt";

export interface StatusGridEffect {
  id: string;
  /** An icon of the atlas. */
  icon: string;
  /** What a reader calls it (Haste). */
  name: string;
  /** How many are stacked; 1 is plain. */
  stacks?: number;
  /** Seconds left (0 lasts until removed). */
  remaining?: number;
  /** Seconds it runs (0 lasts until removed; past it, an extended effect, sweeps nothing). */
  duration?: number;
  /** In its last five seconds. */
  expiring?: boolean;
  /** magic, curse, poison, disease or bleed (debuffs only). */
  type?: string;
}

export interface StatusGridProps {
  /** The buffs, in the game's order. */
  buffs?: ReadonlyArray<StatusGridEffect>;
  /** The debuffs, in their own row and bordered by type. */
  debuffs?: ReadonlyArray<StatusGridEffect>;
  /** Effects past the shown ones, folded into a +n tile (0: none). */
  more?: number;
}

/**
 * The effects on the hero, drawn over the scene: their icons in tiles,
 * buffs and debuffs in their own rows, a debuff rimmed by its type, each
 * with its stacks and the time left, expiring ones pulsing and a tile's
 * time sweeping as it runs out. The style wipes the time round the tile
 * (an SVG ring); here it is the pack's tile sweep, the time already run
 * darkened clockwise. The type notch is dropped (the rim carries the
 * type).
 */
const EDGES: Record<string, string> = {
  magic: "#5cc8ff",
  curse: "#c86ff5",
  poison: "#45e08f",
  disease: "#ffa53d",
  bleed: "#ff9599",
};

function effectTime(remaining: number): string {
  return remaining < 10 ? t("component.status-grid.seconds", { value: remaining }) : formatDuration(remaining);
}

function EffectTile(props: { effect: StatusGridEffect; debuff: boolean }) {
  const remaining = () => props.effect.remaining ?? 0;
  const duration = () => props.effect.duration ?? 0;
  const stacks = () => props.effect.stacks ?? 0;
  const edge = () => (props.debuff ? EDGES[props.effect.type ?? ""] ?? "#ff9599" : "#8892a8");
  return (
    <View
      class="group/statusgrid flex-col items-center gap-y-0.5 min-w-9"
      label={props.effect.name}
      data-expiring={props.effect.expiring === true}
      data-type-magic={props.effect.type === "magic"}
      data-type-curse={props.effect.type === "curse"}
      data-type-poison={props.effect.type === "poison"}
      data-type-disease={props.effect.type === "disease"}
      data-type-bleed={props.effect.type === "bleed"}
    >
      <View
        class={`relative size-8 overflow-hidden rounded-control bg-linear-to-b from-raised-lo to-surface shadow-[0px_0px_0px_1px_#00000073] flex-col items-center justify-center border`}
        style={{ borderColor: edge() }}
      >
        <Icon class="size-6 text-fg text-shadow-2xs group-data-expiring/statusgrid:animate-pulse" name={props.effect.icon} />
        <Show when={duration() > 0 && remaining() < duration()}>
          <View class="absolute inset-0 nine-tile bg-shade-deep" wipe={1 - remaining() / duration()} />
        </Show>
      </View>
      <Show when={stacks() > 1 || remaining() > 0}>
        <View class="flex-row items-center gap-x-1">
          <Show when={stacks() > 1}>
            <Text class="bg-raised-pressed/88 shadow-[inset_0px_0px_0px_1px_#4e586c] rounded px-0.5 text-reading font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs">
              {t("component.status-grid.stacks", { count: stacks() })}
            </Text>
          </Show>
          <Show when={remaining() > 0}>
            <Text class="bg-raised-pressed/88 shadow-[inset_0px_0px_0px_1px_#4e586c] rounded px-0.5 text-reading font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs">
              {effectTime(remaining())}
            </Text>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export function StatusGrid(props: StatusGridProps) {
  const buffs = () => props.buffs ?? [];
  const debuffs = () => props.debuffs ?? [];
  return (
    <View class="flex-col gap-y-1">
      <View class="flex-row flex-wrap items-start max-w-70 gap-x-3 gap-y-3" role="list" label={t("component.status-grid.buffs")}>
        <For each={buffs()} key={(e) => e.id}>
          {(e) => <EffectTile effect={e()} debuff={false} />}
        </For>
        <Show when={(props.more ?? 0) > 0}>
          <View class="flex-col items-center gap-y-0.5 min-w-9">
            <View class="relative size-8 overflow-hidden rounded-control bg-linear-to-b from-raised-lo to-surface shadow-[0px_0px_0px_1px_#00000073] border border-rim-strong flex-row items-center justify-center">
              <Text class="text-reading font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs">{`+${formatNumber(props.more ?? 0)}`}</Text>
            </View>
          </View>
        </Show>
      </View>
      <Show when={debuffs().length > 0}>
        <View class="flex-row flex-wrap items-start max-w-70 gap-x-3 gap-y-3" role="list" label={t("component.status-grid.debuffs")}>
          <For each={debuffs()} key={(e) => e.id}>
            {(e) => <EffectTile effect={e()} debuff={true} />}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<StatusGridProps>> = {
  default: {"more":2,"buffs":[{"id":"haste","icon":"bolt","name":"Haste","stacks":0,"remaining":12,"duration":30,"expiring":false},{"id":"shield","icon":"shield","name":"Barrier","stacks":3,"remaining":4.2,"duration":10,"expiring":true},{"id":"regen","icon":"heart","name":"Regrowth","stacks":0,"remaining":0,"duration":0,"expiring":false}],"debuffs":[{"id":"burn","icon":"fire","name":"Burning","stacks":2,"remaining":6,"duration":8,"expiring":false,"type":"magic"},{"id":"venom","icon":"drop","name":"Venom","stacks":0,"remaining":18,"duration":20,"expiring":false,"type":"poison"}]},
};
