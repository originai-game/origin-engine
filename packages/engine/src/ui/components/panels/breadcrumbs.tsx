import { For, Icon, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./breadcrumbs.en.ogt";

export interface BreadcrumbsProps {
  /** The crumbs, the current page last: each { id, label, icon, current }, current true on the last. */
  items: ReadonlyArray<{ id: string; label: string; icon: string; current: boolean }>;
  /** What a screen reader calls it; empty is Breadcrumb. */
  label: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

export function Breadcrumbs(props: BreadcrumbsProps) {
  return (
    <View class={`text-accent-hover p-0`} label={props.label || t("component.breadcrumbs.name")}>
      <View class="flex-row flex-wrap items-center gap-y-0 gap-x-1 p-0 m-0">
        <For each={props.items}>
          {(crumb) => (
          <View class={`flex-row items-center max-w-full min-h-[2.286rem] gap-1 data-current:min-w-0 data-current:pb-0.5 data-current:text-[1.071rem] data-current:font-bold data-current:leading-snug data-current:tracking-normal data-current:normal-case data-current:text-accent-hover data-current:border-b-[2.288px] data-current:border-[#38e8ff] pointer-coarse:min-h-11 shrink`} data-current={crumb().current}>
            <Show when={crumb().icon}>
              <Icon class={`text-accent-hover/74 size-[1.286rem]`} name={crumb().icon} />
            </Show>
            <Show when={crumb().current} fallback={
              <>
                <View class={`font-semibold text-[1.071rem] leading-snug text-accent-hover/74 hover:text-accent-hover shrink`} href={crumb().id} onPress={(e) => props.onChoose?.(e, crumb().id)}>
                  <Text class="no-underline">{crumb().label}</Text>
                </View>
                <Icon name="play" class="size-[0.571rem] text-accent-hover/32 rtl:-scale-x-100" />
              </>
            }>
              <Text class={`text-accent-hover pb-0.5 text-[1.071rem] font-bold leading-snug border-b-[2.288px] border-[#38e8ff] shrink`} current={false}>
                {crumb().label}
              </Text>
            </Show>
          </View>
          )}
        </For>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<BreadcrumbsProps>> = {
  default: {"label":"","items":[{"id":"home","label":"Home","icon":"home","current":false},{"id":"armory","label":"Armory","icon":"","current":false},{"id":"swords","label":"Swords","icon":"","current":false},{"id":"ember","label":"Ember Blade","icon":"","current":true}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Go back"} as const;
