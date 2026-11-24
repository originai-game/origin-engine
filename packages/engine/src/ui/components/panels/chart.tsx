import { For, Line, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./chart.en.ogt";

export interface ChartProps {
  /** A title over it; empty is none. */
  title?: string;
  /** A line under the title. */
  subtitle?: string;
  /** The value axis' top (its bottom is 0). Default 1. */
  max?: number;
  /** Bars, one per category, left to right: each { id, label, value, tone }, tone a role (danger, success, warn, info, gold) to set one apart, empty the accent. */
  bars: ReadonlyArray<{ id: string; label: string; value: number; tone: string }>;
  /** A line series over the bars, one value per bar (on the same axis); empty is none. */
  line?: readonly number[];
  /** Value axis gridlines: each { id, at, label }, at a fraction of max (0 to 1). */
  ticks?: ReadonlyArray<{ id: string; at: number; label: string }>;
}

/**
 * A HUD plate: the title under an accent kicker, bars standing on a
 * hairline grid with the axis in faint tabular figures, a line series in
 * gold over them through each bar's middle, and each bar's name under it
 * (every other name when there are more than eight). Reports nothing.
 */
export function Chart(props: ChartProps) {
  const max = () => (props.max ?? 1) > 0 ? props.max ?? 1 : 1;
  const n = () => Math.max(props.bars.length, props.line?.length ?? 0);
  const linePoints = () => (props.line ?? []).flatMap((v, i) => [i + 0.5, max() - Math.min(Math.max(v, 0), max())]);
  return (
    <View class={`w-120 max-w-full gap-1 p-4 ${packClass("Panel", "frame")}`} role="figure" label={props.title || t("component.chart.name")}>
      <Show when={props.subtitle}>
        <Text class="text-caption font-bold tracking-label uppercase text-hud-accent-ink">{props.subtitle}</Text>
      </Show>
      <Show when={props.title}>
        <Text class="text-base font-bold text-fg">{props.title}</Text>
      </Show>
      <View class="flex-row h-48 mt-2 gap-2">
        <View class="relative w-8 shrink-0">
          <For each={props.ticks ?? []} key={(k) => k.id}>
            {(k) => (
              <Text class="absolute right-0 -mb-2 text-caption tabular-nums text-fg-muted" style={{ bottom: `${k().at * 100}%` }}>{k().label}</Text>
            )}
          </For>
        </View>
        <View class="relative grow min-w-0 border-l border-border">
          <For each={props.ticks ?? []} key={(k) => k.id}>
            {(k) => (
              <View class="absolute inset-x-0 h-px bg-rim" style={{ bottom: `${k().at * 100}%` }} />
            )}
          </For>
          <View class="absolute inset-0 flex-row items-end">
            <For each={props.bars} key={(b) => b.id}>
              {(b) => (
                <View class="grow basis-0 h-full justify-end px-1" label={`${b().label} ${formatNumber(b().value)}`}>
                  <View
                    class="w-full -skew-x-6 bg-linear-to-b from-hud-accent-hi to-hud-accent data-tone-danger:from-danger data-tone-danger:to-danger data-tone-success:from-success data-tone-success:to-success data-tone-warn:from-warn data-tone-warn:to-warn data-tone-info:from-info data-tone-info:to-info data-tone-gold:from-gold-hi data-tone-gold:to-gold"
                    style={{ height: `${Math.min(Math.max(b().value / max(), 0), 1) * 100}%` }}
                    data-tone-danger={b().tone === "danger"}
                    data-tone-success={b().tone === "success"}
                    data-tone-warn={b().tone === "warn"}
                    data-tone-info={b().tone === "info"}
                    data-tone-gold={b().tone === "gold"}
                  />
                </View>
              )}
            </For>
          </View>
          <Show when={(props.line?.length ?? 0) > 1}>
            <Line class="absolute inset-0 size-full stroke-3 stroke-gold" points={linePoints()} viewBox={[n(), max()]} />
          </Show>
          <Show when={!props.bars.length && !(props.line?.length)}>
            <Text class="absolute inset-0 p-4 text-fg-muted" role="status">{t("component.chart.empty")}</Text>
          </Show>
        </View>
      </View>
      <View class="flex-row pl-10">
        <For each={props.bars} key={(b) => b.id}>
          {(b, i) => (
            <Text class="grow basis-0 min-w-0 text-center whitespace-nowrap text-caption font-semibold text-fg-muted data-skip:opacity-0" data-skip={props.bars.length > 8 && i() % 2 === 1}>{b().label}</Text>
          )}
        </For>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ChartProps>> = {
  default: {"title":"Weekly kills","subtitle":"Last 7 days","max":100,"line":[24,47,36,76,64,95,85],"ticks":[{"id":"0","at":0,"label":"0"},{"id":"50","at":0.5,"label":"50"},{"id":"100","at":1,"label":"100"}],"bars":[{"id":"mon","label":"Mon","value":32,"tone":""},{"id":"tue","label":"Tue","value":51,"tone":""},{"id":"wed","label":"Wed","value":44,"tone":""},{"id":"thu","label":"Thu","value":70,"tone":""},{"id":"fri","label":"Fri","value":62,"tone":"danger"},{"id":"sat","label":"Sat","value":88,"tone":""},{"id":"sun","label":"Sun","value":79,"tone":""}]},
  dense: {"title":"Daily matches","subtitle":"Last 12 days","max":20,"line":[],"ticks":[{"id":"0","at":0,"label":"0"},{"id":"10","at":0.5,"label":"10"},{"id":"20","at":1,"label":"20"}],"bars":[{"id":"1","label":"1 Nov","value":4,"tone":""},{"id":"2","label":"2 Nov","value":7,"tone":""},{"id":"3","label":"3 Nov","value":12,"tone":""},{"id":"4","label":"4 Nov","value":9,"tone":""},{"id":"5","label":"5 Nov","value":15,"tone":""},{"id":"6","label":"6 Nov","value":18,"tone":"gold"},{"id":"7","label":"7 Nov","value":6,"tone":""},{"id":"8","label":"8 Nov","value":11,"tone":""},{"id":"9","label":"9 Nov","value":13,"tone":""},{"id":"10","label":"10 Nov","value":8,"tone":""},{"id":"11","label":"11 Nov","value":16,"tone":""},{"id":"12","label":"12 Nov","value":14,"tone":""}]},
  empty: {"title":"Weekly kills","subtitle":"","max":100,"line":[],"ticks":[],"bars":[]},
};
