import { For, Icon, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import { Spinner } from "./spinner";
import "./pips.en.ogt";

export interface PipsProps {
  /** The resource's name; empty uses Resource. */
  label?: string;
  /** The atlas icon for a unit: heart, bullet, bolt or a game's own icon. Default "heart". */
  icon?: string;
  /** Keyed units { id, value, label }: value is the filled fraction, 0 to 1; label names the unit for accessibility. The game supplies capacity, ordering and fractional units, with no fixed maximum. */
  items?: ReadonlyArray<{ id: string; value: number; label?: string }>;
  /** Waiting for resource data; units are replaced by Loading. Default false. */
  loading?: boolean;
  /** The resource is unavailable; its values remain readable. Default false. */
  disabled?: boolean;
  /** The full units' colour: empty is the style's pip colour (a life's red); success, warn, danger, info or primary names another. */
  tone?: string;
  class?: string;
}

const FILL: Record<string, string> = {
  success: packClass("Pips", "fill", "success"),
  warn: packClass("Pips", "fill", "warn"),
  info: packClass("Pips", "fill", "info"),
  primary: packClass("Pips", "fill", "primary"),
};

/**
 * A counted resource on one line: its name, then one icon per unit. An
 * empty unit is its socket; a full one is cut to its fraction and wears
 * the tone. Unavailable, the units lose their colour and dim, full ones
 * still brighter than their sockets.
 */
export function Pips(props: PipsProps) {
  return (
    <View class={`group/pips flex-row flex-wrap items-center text-fg max-w-full p-0 gap-2 disabled:text-fg-subtle ${props.class ?? ""}`} role="group" label={props.label || t("base.pips.resource")} busy={(props.loading ?? false)} disabled={(props.disabled ?? false)} data-off={(props.disabled ?? false)}>
      <View class={`shrink ${packClass("Pips", "label")}`}>
        <Text>{props.label || t("base.pips.resource")}</Text>
      </View>
      <Show when={(props.loading ?? false)} fallback={
        <>
          <Show when={(props.items ?? []).length} fallback={
              <View class={`text-caption text-fg-muted shrink`}>
                <Text>{t("base.pips.empty")}</Text>
              </View>
          }>
            <View class={`flex-row flex-wrap items-center shrink ${packClass("Pips", "units")}`}>
              <For each={props.items ?? []}>
                {(item) => (
                <View class={`shrink ${packClass("Pips", "pip")}`} role="meter" label={item().label || props.label || t("base.pips.resource")} valueMin={0} valueMax={1} valueNow={item().value < 0 ? 0 : item().value > 1 ? 1 : item().value} valueText={`${formatNumber((item().value < 0 ? 0 : item().value > 1 ? 1 : item().value) * 100, { decimals: 0 })}%`}>
                  <Icon class={`absolute inset-0 size-full`} name={(props.icon ?? "heart")} />
                  <View class={`absolute inset-y-0 left-0 overflow-hidden ${FILL[props.tone ?? ""] ?? packClass("Pips", "fill")}`} style={{ width: `${Math.min(Math.max(item().value, 0), 1) * 100}%` }}>
                    <Icon class={`aspect-square h-full group-data-off/pips:text-fg-muted`} name={(props.icon ?? "heart")} />
                  </View>
                </View>
                )}
              </For>
            </View>
          </Show>
          <Show when={(props.disabled ?? false)}>
            <View class={`text-caption bg-raised-lo inset-ring-hairline inset-ring-border text-fg transition duration-control py-0 px-2 rounded-capsule shrink`}>
              <Text class="tabular-nums">{t("base.pips.disabled")}</Text>
            </View>
          </Show>
        </>
      }>
        <View class={`flex-row items-center gap-1 shrink`}>
          <Spinner size={18} name={t("base.pips.loading")} />
          <View class={`text-caption text-fg-muted shrink`} role="status">
            <Text>{t("base.pips.loading")}</Text>
          </View>
        </View>
      </Show>
    </View>
  );
}

/** Where it belongs: a piece of a panel, drawn on the panel's ground and never straight over the scene; the gallery mounts it inside a panel's frame. */
export const ground = "panel" as const;

export const fixtures: Record<string, Partial<PipsProps>> = {
  default: {"label":"Lives","icon":"heart","tone":"","items":[{"id":"life-a","value":1,"label":"First life"},{"id":"life-b","value":0.5,"label":"Second life"},{"id":"life-c","value":0,"label":"Third life"}],"loading":false,"disabled":false},
  full: {"label":"Action points","icon":"bolt","tone":"warn","items":[{"id":"ap-a","value":1},{"id":"ap-b","value":1},{"id":"ap-c","value":1}]},
  empty: {"label":"Lives","items":[]},
  depleted: {"label":"Lives","items":[{"id":"last","value":0}]},
  loading: {"label":"Lives","loading":true},
  disabled: {"label":"Action points","icon":"bolt","tone":"warn","disabled":true,"items":[{"id":"ap-a","value":1}]},
};
