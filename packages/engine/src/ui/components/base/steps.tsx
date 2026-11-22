import { For, Icon, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./states.en.ogt";

export interface StepsProps {
  /** Optional visible heading; the accessible fallback is Progress steps. */
  title?: string;
  /** Keyed { id, label, detail, number, state, disabled } rows. number is the game's formatted marker, state upcoming/current/done/error. The game chooses order and allowed navigation. Default []. */
  steps?: ReadonlyArray<{ id: string; label: string; detail?: string; number: string; state: string; disabled?: boolean }>;
  /** Stack steps vertically instead of a wrapping horizontal sequence. Default false. */
  column?: boolean;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

/**
 * A rail of numbered marks: each step is a mark, the reach to the next
 * one (full once the step is done) and its words. It lies along a row
 * and stands up as a column where `column` asks or the container is
 * narrow.
 */
export function Steps(props: StepsProps) {
  return (
    <View class="grow shrink basis-0 h-auto gap-2">
      <View class={`group/steps text-fg p-0 gap-3 disabled:text-fg-subtle`} label={props.title || t("base.steps.title")} data-column={(props.column ?? false)}>
        <Show when={props.title}>
          <Text class={`font-display text-base font-bold leading-tight tracking-tight text-fg m-0`}>
            {props.title}
          </Text>
        </Show>
        <View class={`flex-row p-0 m-0 group-data-column/steps:flex-col compact:flex-col`}>
          <Show when={(props.steps ?? []).length} fallback={<View class={`text-caption text-fg-muted shrink`} role="status"><Text>{t("base.steps.empty")}</Text></View>}>
          <For each={(props.steps ?? [])}>
            {(step, i) => (
            <View class={`group/step flex-row grow shrink basis-0 min-w-0 group-data-column/steps:flex-none compact:flex-none`} data-last={i() === (props.steps ?? []).length - 1} data-state-done={(step().state) === 'done'} data-state-upcoming={(step().state) === 'upcoming'}>
              <View class={`text-fg transition duration-317 items-start w-full min-h-11 text-start p-2 rounded-control hover:bg-surface-hover hover:bg-none active:bg-raised-pressed active:bg-none data-selected:bg-transparent data-selected:bg-none data-selected:text-fg data-selected:shadow-none disabled:bg-transparent disabled:bg-none disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45 shrink`} role="button" disabled={step().disabled} current={(step().state === 'current' ? 'step' : null) != null} label={step().label} description={step().detail} onPress={(e) => props.onChoose?.(e, step().id)}>
                <View class="grid self-start grid-cols-1 gap-y-2 gap-x-3 w-full group-data-column/steps:grid-cols-[auto_minmax(0,1fr)] compact:grid-cols-[auto_minmax(0,1fr)]">
                  <View class="flex-row items-center gap-1 group-data-column/steps:flex-col compact:flex-col">
                    <View class={`bg-accent text-accent-fg justify-center items-center gap-2 size-8 rounded-capsule group-data-state-upcoming/step:border-rim-soft group-data-state-upcoming/step:bg-none group-data-state-upcoming/step:bg-tile group-data-state-upcoming/step:text-fg-muted group-data-state-upcoming/step:border data-tone-success:bg-none data-tone-success:bg-success data-tone-success:text-success-fg data-tone-danger:bg-none data-tone-danger:bg-danger data-tone-danger:text-danger-fg data-tone-primary:bg-none data-tone-primary:bg-accent data-tone-primary:text-accent-fg data-play-pop:animate-pop`} data-play-pop={(step().state === 'current' ? 'pop' : null) === "pop"} data-tone-success={(step().state === 'done' ? 'success' : step().state === 'error' ? 'danger' : step().state === 'current' ? 'primary' : null) === "success"} data-tone-danger={(step().state === 'done' ? 'success' : step().state === 'error' ? 'danger' : step().state === 'current' ? 'primary' : null) === "danger"} data-tone-primary={(step().state === 'done' ? 'success' : step().state === 'error' ? 'danger' : step().state === 'current' ? 'primary' : null) === "primary"}>
                      <Show when={step().state === 'done' || step().state === 'error'} fallback={
                          <Text>{step().number}</Text>
                      }>
                        <Icon class="size-4" name={step().state === 'done' ? 'check' : 'close'} />
                      </Show>
                    </View>
                    <View class={`flex-row bg-rim-soft grow shrink basis-0 min-w-3 h-0.75 -mr-3 rounded-capsule overflow-hidden group-data-last/step:invisible group-data-column/steps:min-w-0 group-data-column/steps:h-auto group-data-column/steps:mr-0 group-data-column/steps:flex-col group-data-column/steps:w-0.75 group-data-column/steps:min-h-3 group-data-column/steps:-mb-3 compact:min-w-0 compact:h-auto compact:mr-0 compact:flex-col compact:w-0.75 compact:min-h-3 compact:-mb-3`}>
                      <View class={`bg-success text-success basis-0 group-data-state-done/step:grow group-data-state-done/step:shrink group-data-state-done/step:basis-0`} />
                    </View>
                  </View>
                  <View class="gap-0.5">
                    <Text class={`text-caption text-fg-muted data-tone-success:text-success data-tone-danger:text-danger data-tone-primary:text-accent`} data-tone-success={(step().state === 'done' ? 'success' : step().state === 'error' ? 'danger' : step().state === 'current' ? 'primary' : null) === "success"} data-tone-danger={(step().state === 'done' ? 'success' : step().state === 'error' ? 'danger' : step().state === 'current' ? 'primary' : null) === "danger"} data-tone-primary={(step().state === 'done' ? 'success' : step().state === 'error' ? 'danger' : step().state === 'current' ? 'primary' : null) === "primary"}>
                      {step().state === 'done' ? t("base.steps.done") : step().state === 'error' ? t("base.steps.error") : step().state === 'current' ? t("base.steps.current") : t("base.steps.upcoming")}
                    </Text>
                    <Text class={`font-semibold text-lg text-fg group-disabled/row:text-fg-subtle group-data-state-upcoming/step:text-fg-muted`}>
                      {step().label}
                    </Text>
                    <Show when={step().detail}>
                      <Text class={`text-caption text-fg-muted`}>
                        {step().detail}
                      </Text>
                    </Show>
                  </View>
                </View>
              </View>
            </View>
            )}
          </For>
          </Show>
        </View>
      </View>
    </View>
  );
}

/** Where it belongs: a piece of a panel, drawn on the panel's ground and never straight over the scene; the gallery mounts it inside a panel's frame. */
export const ground = "panel" as const;

export const fixtures: Record<string, Partial<StepsProps>> = {
  default: {"title":"Prepare your expedition","steps":[{"id":"party","label":"Party","detail":"3 players ready","number":"1","state":"done","disabled":false},{"id":"route","label":"Route","detail":"Choose a destination","number":"2","state":"current","disabled":false},{"id":"launch","label":"Launch","detail":"Waiting for a route","number":"3","state":"upcoming","disabled":true}],"column":false},
  empty: {"steps":[]},
  column: {"title":"Prepare your expedition","column":true,"steps":[{"id":"party","label":"Party","detail":"3 players ready","number":"1","state":"done","disabled":false},{"id":"route","label":"Route","detail":"Choose a destination","number":"2","state":"current","disabled":false},{"id":"launch","label":"Launch","detail":"Waiting for a route","number":"3","state":"upcoming","disabled":true}]},
  error: {"steps":[{"id":"gear","label":"Equipment","number":"1","state":"done"},{"id":"network","label":"Connection","detail":"Reconnect to continue","number":"2","state":"error"},{"id":"ready","label":"Ready","number":"3","state":"upcoming","disabled":true}]},
  overflow: {"title":"An expedition across the northern territories","steps":[{"id":"one","label":"Choose your character and starting equipment","detail":"Every choice can be changed later in the lobby.","number":"1","state":"done"},{"id":"two","label":"Invite the rest of your expedition party","detail":"Waiting for players to accept their invitations.","number":"2","state":"current"},{"id":"three","label":"Confirm the expedition destination","number":"3","state":"upcoming","disabled":true}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Choose a step, by its stable key"} as const;
