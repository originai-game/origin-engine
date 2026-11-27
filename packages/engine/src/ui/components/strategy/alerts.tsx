import { For, Show, Text, View, t } from "@origingame/engine/ui";
import "./strategy.en.strings.ts";

export interface StrategyAlertsProps {
  /** The alerts and notices, newest first, keyed rows of { id, kind, text, key, faded }: kind danger (under attack), done (research complete) or info, on data-kind, each with its own mark and edge; text the game's words; key the input that jumps to it as the game names it (Space), on a key cap, empty none, hidden under a touch screen; faded an older notice, set back (data-faded). */
  alerts?: ReadonlyArray<{ id: string; kind: string; text: string; key: string; faded: boolean }>;
}

/**
 * Alerts and notices at the top left under the resources: one feed row
 * each, newest first, lit from the edge it hangs on, a mark and an edge in
 * danger, done or info, the game's words and the key that jumps to it; an
 * older notice set back. The game owns which alerts stand and for how
 * long; read only.
 */
export function StrategyAlerts(props: StrategyAlertsProps) {
  return (
    <View class={`group/alerts w-fit max-w-full min-w-0 flex-col items-start text-fg p-0 m-0 gap-y-1.25 compact:w-full compact:gap-y-0.75`} role="log" label={t("strategy.alerts.name")}>
      <For each={props.alerts ?? []} key={(a) => a.id}>
        {(a) => (
        <View class={`group/alert flex-row items-center gap-x-2 max-w-full min-w-0 pt-1 pr-3 pb-1 pl-0 compact:gap-x-1.5 compact:pt-0.75 compact:pr-2 compact:pb-0.75 bg-well`} label={a().kind === "danger" ? t("strategy.alerts.danger", { text: a().text }) : a().kind === "done" ? t("strategy.alerts.done", { text: a().text }) : a().text} data-danger={a().kind === "danger"} data-done={a().kind === "done"} data-info={a().kind === "info"} data-faded={a().faded}>
          <View class={`shrink self-stretch w-0.75 mr-1 bg-sheen group-data-danger/alert:bg-danger group-data-done/alert:bg-success group-data-info/alert:bg-hud-accent`} />
          <View class={`shrink rotate-45 size-2 bg-sheen ring-regular ring-shade-deep group-data-danger/alert:bg-danger group-data-done/alert:bg-success group-data-info/alert:bg-hud-accent`} />
          <Text class={`min-w-0 text-sm font-semibold whitespace-nowrap overflow-hidden text-stroke-3 text-stroke-bg group-data-faded/alert:text-fg-muted compact:text-caption`}>
            {a().text}
          </Text>
          <Show when={a().key}>
            <Text class={`shrink min-w-5.5 max-w-2/5 py-0 px-1 leading-normal text-xs font-bold text-center text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key whitespace-nowrap overflow-hidden pointer-coarse:hidden`}>
              {a().key}
            </Text>
          </Show>
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategyAlertsProps>> = {
  default: {"alerts":[{"id":"a1","kind":"danger","text":"The east tower is under attack","key":"Space","faded":false},{"id":"a2","kind":"done","text":"Research complete: Steel arrowheads","key":"","faded":true}]},
  info: {"alerts":[{"id":"a1","kind":"info","text":"Your ally needs help at the ford","key":"F1","faded":false},{"id":"a2","kind":"done","text":"Barracks complete","key":"","faded":false},{"id":"a3","kind":"danger","text":"Villager killed","key":"","faded":true}]},
  overflow: {"alerts":[{"id":"a1","kind":"danger","text":"The Grand Cathedral of the Seventh Concord on the eastern isle is under attack by siege engines","key":"Left Shoulder + Space","faded":false},{"id":"a2","kind":"done","text":"Research complete: Masterwork Steel Arrowheads of the Royal Fletchers Guild","key":"","faded":true},{"id":"a3","kind":"info","text":"Market prices changed","key":"","faded":true},{"id":"a4","kind":"info","text":"A trade caravan arrived","key":"","faded":true}]},
};
