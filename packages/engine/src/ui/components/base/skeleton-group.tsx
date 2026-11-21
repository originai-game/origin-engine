import { For, Show, View, t } from "@origingame/engine/ui";
import "./skeleton-group.en.ogt";

export interface SkeletonGroupProps {
  /** What it stands in for: lines (empty), avatar, block, row or card. */
  shape: string;
  /** One entry per repeat (rows of a list), each with a stable id. */
  items: ReadonlyArray<{ id: string }>;
  /** What a screen reader says while it loads; empty is Loading. */
  name: string;
}

/**
 * One band of light crosses the whole group: every bone's sheen is the
 * same width (a share of the group's) and starts from the group's edge,
 * so the bones light together as it passes.
 */
export function SkeletonGroup(props: SkeletonGroupProps) {
  return (
    <View class="group/skeletongroup w-full gap-3" role="progressbar" busy={true} label={props.name || t("component.skeleton-group.loading")} data-avatar={props.shape === 'avatar'} data-block={props.shape === 'block'} data-row={props.shape === 'row'} data-card={props.shape === 'card'}>
      <For each={props.items}>
        {(item) => (
        <View class="gap-1 group-data-row/skeletongroup:flex-row group-data-row/skeletongroup:items-center group-data-row/skeletongroup:gap-3">
          <Show when={props.shape === 'avatar' || props.shape === 'block' || props.shape === 'row' || props.shape === 'card'}>
            <View class={`bg-bg-subtle rounded-control overflow-hidden group-data-avatar/skeletongroup:size-12 group-data-avatar/skeletongroup:rounded-capsule group-data-row/skeletongroup:size-11 group-data-row/skeletongroup:rounded-capsule group-data-block/skeletongroup:h-32 group-data-card/skeletongroup:h-32`}>
              <View class={`absolute left-0 w-1/4 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 animate-shimmer`} />
            </View>
          </Show>
          <Show when={props.shape !== 'avatar' && props.shape !== 'block'}>
            <View class="grow shrink basis-0 gap-1 group-data-row/skeletongroup:gap-3">
              <View class={`bg-bg-subtle w-full h-3 rounded-capsule overflow-hidden group-data-row/skeletongroup:w-3/5 group-data-card/skeletongroup:w-[55%] group-data-card/skeletongroup:h-4`}>
                <View class={`absolute left-0 w-1/4 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 group-data-row/skeletongroup:-left-14 animate-shimmer`} />
              </View>
              <View class={`bg-bg-subtle w-full h-3 rounded-capsule overflow-hidden`}>
                <View class={`absolute left-0 w-1/4 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 group-data-row/skeletongroup:-left-14 animate-shimmer`} />
              </View>
              <Show when={props.shape !== 'row'}>
                <View class={`bg-bg-subtle w-[62%] h-3 rounded-capsule overflow-hidden`}>
                  <View class={`absolute left-0 w-1/4 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 group-data-row/skeletongroup:-left-14 animate-shimmer`} />
                </View>
              </Show>
            </View>
          </Show>
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<SkeletonGroupProps>> = {
  default: {"shape":"row","items":[{"id":"a"},{"id":"b"},{"id":"c"}],"name":""},
};
