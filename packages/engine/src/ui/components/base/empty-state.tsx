import { Icon, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import "./states.en.ogt";

export interface EmptyStateProps {
  /** Heading; empty uses Nothing here yet. */
  title?: string;
  /** Why this area is empty. */
  body?: string;
  /** An optional next step, in quieter text. */
  hint?: string;
  /** An icon above the heading; empty hides it. Default "bag". */
  icon?: string;
  /** The optional action's words; empty hides the button. */
  actionLabel?: string;
  /** The action is unavailable. Default false. */
  disabled?: boolean;
  /** The activate event, with the row's key where it comes from a row. */
  onActivate?: (e: UiEvent, key?: string) => void;
}

/**
 * Where content would be, there is a recess: a mark on a disc, why the
 * area is empty, and the one thing to do about it.
 */
export function EmptyState(props: EmptyStateProps) {
  return (
    <View class={`bg-tile shadow-sunk text-fg items-center text-center p-6 gap-4 rounded-block disabled:text-fg-subtle transition duration-base ease-out starting:opacity-0 starting:translate-y-2`} role="status" label={props.title || t("base.empty.title")}>
      <Show when={(props.icon ?? "bag")}>
        <View class={`bg-card border-border text-fg justify-center items-center gap-4 size-16 rounded-capsule border`}>
          <Icon class={`text-fg-muted size-8`} name={(props.icon ?? "bag")} />
        </View>
      </Show>
      <View class="items-center max-w-105 gap-1">
        <Text class={`font-semibold text-lg text-fg m-0`}>
          {props.title || t("base.empty.title")}
        </Text>
        <Show when={props.body}>
          <Text class={`text-fg-muted text-center m-0`}>{props.body}</Text>
        </Show>
        <Show when={props.hint}>
          <Text class={`text-caption text-fg-muted text-center m-0`}>
            {props.hint}
          </Text>
        </Show>
      </View>
      <Show when={props.actionLabel}>
        <Button variant="primary" class="max-w-full" disabled={(props.disabled ?? false)} onPress={(e) => props.onActivate?.(e)}>
          <Text class="whitespace-normal">{props.actionLabel}</Text>
        </Button>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<EmptyStateProps>> = {
  default: {"title":"No saved builds","body":"Save your equipment to switch builds between matches.","hint":"Your current equipment will not change.","icon":"bag","actionLabel":"Save this build","disabled":false},
  minimal: {},
  disabled: {"title":"No saved builds","body":"Finish the tutorial before saving a build.","actionLabel":"Save build","disabled":true},
  overflow: {"title":"No discoveries in the northern reaches of the Sunken Kingdom","body":"Explore the coast and investigate the old watchtowers to add discoveries to this collection.","hint":"Progress is shared across every character on your account.","actionLabel":"Return to the world map and choose a destination"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"activate":"Take the suggested action"} as const;
