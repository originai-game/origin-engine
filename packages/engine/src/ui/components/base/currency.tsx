import { Icon, Show, Text, View, formatCompact, t, packClass } from "@origingame/engine/ui";
import { Spinner } from "./spinner";
import "./currency.en.ogt";

export interface CurrencyProps {
  /** Resource name; empty uses Currency. */
  name: string;
  /** Atlas icon; empty omits it. */
  icon: string;
  /** Amount held; zero remains visible. Ignored when valueText is supplied. */
  amount: number;
  /** Game-formatted amount; empty uses the compact amount. */
  valueText: string;
  /** Game-formatted capacity; empty omits it. The game decides whether the store is full. */
  limit: string;
  /** Optional income or drain, including its sign and unit. */
  rate: string;
  /** At capacity; reads Full in words. */
  full?: boolean;
  /** Awaiting the amount; hides stale figures and reads Loading. */
  loading?: boolean;
  /** The balance could not be read; hides stale figures and reads Unavailable. */
  unavailable?: boolean;
}

/**
 * A readout, not a button. A game can put it beside a separately named
 * purchase control; Currency never invents an economy or sends an intent.
 * A chip: the resource's icon in a socket, its name over the amount, and
 * beside the amount its capacity, its rate and Full. It is as wide as
 * what it holds.
 */
export function Currency(props: CurrencyProps) {
  return (
    <View class={`flex-row items-center pr-3 pl-1 w-fit max-w-full py-1 gap-2 disabled:text-fg-subtle ${packClass("Plate", "frame")}`} label={props.name || t("currency.name")} busy={props.loading} disabled={props.unavailable} data-full={props.full} data-unavailable={props.unavailable}>
      <Show when={props.icon}>
        <View class={`bg-tile shadow-sunk justify-center items-center gap-2 size-9 rounded-control`}>
          <Icon class={`size-6`} name={props.icon} />
        </View>
      </Show>
      <View class="shrink gap-0">
        <View class={`text-caption text-fg-muted`}>
          <Text>{props.name || t("currency.name")}</Text>
        </View>
        <Show when={props.loading} fallback={
            <Show when={props.unavailable} fallback={
                <View class="flex-row flex-wrap items-baseline gap-1">
                  <View class={`text-lg font-bold text-fg shrink`}>
                    <Text class={`tabular-nums ${packClass("Count", "roll")}`}>{props.valueText || formatCompact(props.amount)}</Text>
                  </View>
                  <Show when={props.limit}>
                    <View class={`text-caption text-fg-muted shrink`}>
                      <Text>{t("currency.capacity", { limit: props.limit })}</Text>
                    </View>
                  </Show>
                  <Show when={props.full}>
                    <View class={`bg-warn text-warn-fg font-bold text-caption py-0.5 px-2 rounded-capsule animate-pop shrink`}>
                      <Text>{t("currency.full")}</Text>
                    </View>
                  </Show>
                  <Show when={props.rate}>
                    <View class={`text-caption bg-raised-lo inset-ring-hairline inset-ring-border text-fg transition duration-control py-0 px-2 rounded-control shrink`}>
                      <Text class="tabular-nums">{props.rate}</Text>
                    </View>
                  </Show>
                </View>
            }>
              <View class={`font-semibold text-lg text-fg-muted`} role="status">
                <Text>{t("currency.unavailable")}</Text>
              </View>
            </Show>
        }>
          <View class="flex-row items-center gap-1">
            <View class={`shrink`}>
              <Spinner size={18} name={t("currency.loading")} />
            </View>
            <View class={`font-semibold text-lg text-fg-muted shrink`} role="status">
              <Text>{t("currency.loading")}</Text>
            </View>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CurrencyProps>> = {
  default: {"name":"Gold","icon":"coin","amount":12540,"valueText":"","limit":"","rate":"+48/s","full":false,"loading":false,"unavailable":false},
  empty: {"name":"Gems","icon":"gem","amount":0},
  full: {"name":"Energy","icon":"bolt","amount":120,"limit":"120","full":true},
  loading: {"name":"Gold","icon":"coin","amount":999,"loading":true},
  unavailable: {"name":"Gold","icon":"coin","amount":999,"unavailable":true},
  overflow: {"name":"Sovereigns of the Eastern Treasury","icon":"crown","valueText":"123.45 quattuordecillion","limit":"500 quattuordecillion","rate":"−4.2 million per second"},
};
