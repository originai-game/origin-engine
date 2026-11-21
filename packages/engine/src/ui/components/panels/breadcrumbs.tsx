import { For, Icon, Line, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./breadcrumbs.en.ogt";

export interface BreadcrumbsProps {
  /** The crumbs, the current page last: each { id, label, icon, current }, current true on the last. */
  items: ReadonlyArray<{ id: string; label: string; icon: string; current: boolean }>;
  /** What a screen reader calls it; empty is Breadcrumb. */
  label: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

/**
 * A trail in type: dim links, faint chevrons, the current page bold on a
 * lit underline. The trail wraps between crumbs, and a long current page
 * takes a line of its own and wraps between its words. Reports choose (a
 * crumb's key).
 *
 * Dropped from the .svelte: nothing (the separator's svg path is a `Line`).
 */
export function Breadcrumbs(props: BreadcrumbsProps) {
  return (
    <View class={`text-fg p-0`} role="navigation" label={props.label || t("component.breadcrumbs.name")}>
      <View class="flex-row flex-wrap items-center gap-y-0 gap-x-1 p-0 m-0" role="list">
        <For each={props.items} key={(crumb) => crumb.id}>
          {(crumb) => (
          <View class={`flex-row items-center max-w-full min-h-8 gap-1 shrink pointer-coarse:min-h-11`} role="listitem" data-current={crumb().current}>
            <Show when={crumb().icon}>
              <Icon class={`text-fg-muted size-[1.286rem] shrink-0`} name={crumb().icon} />
            </Show>
            <Show when={crumb().current} fallback={
                <>
                  <View class={`font-semibold text-[1.071rem] leading-snug text-fg-muted hover:text-fg focus-visible:text-fg shrink-0 min-w-0`} role="link" onPress={(e) => props.onChoose?.(e, crumb().id)}>
                    <Text class="min-w-0">{crumb().label}</Text>
                  </View>
                  <Line class={`text-fg-subtle w-2 h-3 shrink-0 rtl:-scale-x-100`} points={[2, 1, 6, 6, 2, 11]} viewBox="0 0 8 12" />
                </>
            }>
              <Text class={`text-fg pb-0.5 text-[1.071rem] font-bold leading-snug tracking-normal normal-case border-b-2 border-hud-accent shrink-0 min-w-0`} current={true}>
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
  long: {"label":"","items":[{"id":"home","label":"Home","icon":"home","current":false},{"id":"realm","label":"The Sunken Marches of the Ninth Realm","icon":"","current":false},{"id":"ember","label":"Ember Blade of the Colossus Bane and the Ashen Oath","icon":"","current":true}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Go back"} as const;
