import type { Texture } from "@origingame/engine";
import {Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import "./tutorial.en.ogt";

export interface TutorialScreenProps {
  /** The step's title. */
  title: string;
  /** What to do. */
  text: string;
  /** The step, from 1. */
  step: number;
  /** Steps in the lesson. */
  steps: number;
  /** The step waits for the player to act (Next hidden). */
  waits?: boolean;
  /** The card's start edge, CSS px from the screen's start (the right, right to left). */
  x: number;
  /** The card's top, CSS px. */
  y: number;
  /** The step's art (0: none). */
  texture: Texture | 0;
  /** The id of the element the step points at, in any document under this one: the rest of the screen dims and takes presses, the element stays lit and takes its own, and the card stands against it in place of x and y. Empty: nothing is dimmed. The TSX host has no spotlight surface yet, so the card stands at x and y regardless. */
  target?: string;
  /** Content the caller supplies (was the highlight snippet). */
  highlight?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the hint snippet). */
  hint?: (() => JSX.Element) | undefined;
  /** The next event, with the row's key where it comes from a row. */
  onNext?: (e: UiEvent, key?: string) => void;
  /** The skip event, with the row's key where it comes from a row. */
  onSkip?: (e: UiEvent, key?: string) => void;
}

/**
 * A tutorial step: a dense smoked plate lit in the accent, at the game's x
 * and y, with the step count and the lesson's progress, the step's art,
 * its words, and Skip beside Next. The Svelte screen's spotlight (the dim
 * over the rest of the screen and the card anchored to the target element)
 * has no TSX surface yet: the card always stands at x and y, and nothing
 * is dimmed.
 */
function StepCard(props: TutorialScreenProps) {
  return (
    <View class={`bg-linear-to-b from-[#171f30f5] to-[#080c16f7] shadow-[inset_0px_0px_0px_1px_#38e8ff73,inset_0px_2px_0px_#38e8ff,0px_20.576px_45.712px_#00000099] text-fg w-full max-w-[28.571rem] transition duration-240 ease-out absolute py-4 px-6 gap-2 narrow:w-auto narrow:max-w-none narrow:inset-x-3 starting:opacity-0 starting:scale-60 ${packClass("Panel", "frame")}`} style={{ left: `${props.x}px`, top: `${props.y}px` }}>
      <View class="flex-row items-center gap-x-3">
        <Text class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase tabular-nums text-fg m-0 shrink`}>
          {t("screen.tutorial.Step", { step: props.step, steps: props.steps })}
        </Text>
        <View class="grow shrink basis-0 min-w-0 h-0.5 bg-[#3d465e]">
          <View class="h-full bg-[#38e8ff] shadow-[0px_0px_4.571px_#38e8ff]" style={{ width: `${(props.step / (props.steps || 1)) * 100}%` }} />
        </View>
      </View>
      <Show when={props.texture !== 0}>
        <Art kind="tutorial" src={props.texture} />
      </Show>
      <Text class={`text-[1.714rem] leading-tight font-bold tracking-xs text-fg m-0`}>
        {props.title}
      </Text>
      <Text class={`leading-normal text-fg-muted m-0`}>
        {props.text}
      </Text>
      {props.hint?.()}
      <View class="flex-wrap items-center justify-between gap-y-2 gap-x-2 mt-1">
        <Button class={`text-fg-muted compact:-ml-2 shrink ${packClass("Button", "plate")}`} onPress={(e) => props.onSkip?.(e)}>
          <Text>{t("screen.tutorial.skip")}</Text>
        </Button>
        <Show when={!props.waits}>
          <Button class={`text-accent nine-plate-cut ml-auto min-w-[8.571rem] h-9 shrink ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onNext?.(e)}>
            <Text>{t("screen.tutorial.next")}</Text>
          </Button>
        </Show>
      </View>
    </View>
  );
}

export function TutorialScreen(props: TutorialScreenProps) {
  return (
    <View class="relative h-full">
      <View class="absolute inset-0">{props.highlight?.()}</View>
      <StepCard {...props} />
    </View>
  );
}

export const fixtures: Record<string, Partial<TutorialScreenProps>> = {
  default: {"step":2,"steps":5,"target":"","text":"Press the dodge button just before an attack lands to slip past it.","texture":0,"title":"Dodge","waits":false,"x":64,"y":120},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Next","back":"Skip"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"tutorial","layer":"popover","focus":"","under":"none","input":"all","pause":false,"capture":false,"remember":true,"back":"skip"} as const;
