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
  /** The `name` of the element the step points at, anywhere in the UI (name a `View` around an `Icon`): the rest of the screen dims and takes every press, the element stays lit and takes its own, and the card stands under it in place of x and y. Empty: nothing is dimmed and the card stands at x and y. While no element of that name shows, nothing is dimmed and the card stands at x and y. */
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
 * A tutorial step's card: a dense smoked plate lit in the accent, with the
 * step count and the lesson's progress, the step's art, its words, and Skip
 * beside Next. It stands under the target when there is one, else at the
 * game's x and y.
 */
function StepCard(props: TutorialScreenProps) {
  const target = () => props.target ?? "";
  return (
    <View
      anchor={target() || undefined}
      lift={target() ? [0, 1, 0] : undefined}
      class={`data-target:translate-y-3 bg-linear-to-b from-glass-hi to-glass-lo shadow-overlay inset-shadow-overline inset-ring-1 inset-ring-hud-accent text-fg w-full max-w-100 transition duration-240 ease-out absolute py-4 px-6 gap-2 narrow:w-auto narrow:max-w-none narrow:inset-x-3 starting:opacity-0 starting:scale-60 ${packClass("Panel", "frame")}`}
      data-target={target() !== ""}
      style={target() ? undefined : { left: `${props.x}px`, top: `${props.y}px` }}
    >
      <View class="flex-row items-center gap-x-3">
        <Text class={`text-xs font-bold tracking-label uppercase tabular-nums text-fg m-0 shrink`}>
          {t("screen.tutorial.Step", { step: props.step, steps: props.steps })}
        </Text>
        <View class="grow shrink basis-0 min-w-0 h-0.5 bg-raised-hi">
          <View class="h-full bg-hud-accent shadow-glow-sm" style={{ width: `${(props.step / (props.steps || 1)) * 100}%` }} />
        </View>
      </View>
      <Show when={props.texture !== 0}>
        <Art kind="tutorial" src={props.texture} />
      </Show>
      <Text class={`text-title leading-tight font-bold tracking-xs text-fg m-0`}>
        {props.title}
      </Text>
      <Text class={`leading-normal text-fg-muted m-0`}>
        {props.text}
      </Text>
      {props.hint?.()}
      <View class="flex-row flex-wrap items-center justify-between gap-y-2 gap-x-2 mt-1">
        <Button class={`text-fg-muted compact:-ml-2 shrink ${packClass("Button", "plate")}`} onPress={(e) => props.onSkip?.(e)}>
          <Text>{t("screen.tutorial.skip")}</Text>
        </Button>
        <Show when={!props.waits}>
          <Button class={`text-accent nine-plate-cut ml-auto min-w-30 h-9 shrink ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onNext?.(e)}>
            <Text>{t("screen.tutorial.next")}</Text>
          </Button>
        </Show>
      </View>
    </View>
  );
}

/**
 * A tutorial step: a card with the step's title, what to do, its art and
 * the step count, and Next (hidden while the step waits for the player) and
 * Skip. Given a target that shows, the screen dims around that element, which
 * keeps its own presses, and the card stands under it; else (no target, or
 * one not on screen) nothing dims and the card stands where the game puts it.
 */
export function TutorialScreen(props: TutorialScreenProps) {
  return (
    <View class={`relative h-full ${packClass("Screen", "frame")}`}>
      <View class="absolute inset-0">{props.highlight?.()}</View>
      <Show when={props.target} fallback={<StepCard {...props} />}>
        {/* data-lit: the target shows. Until it does, no shade, and the card stands at x and y. */}
        <View class="group/spot absolute inset-0 data-lit:bg-scrim" spotlight={props.target}>
          <StepCard {...props} />
          <View class="absolute inset-0 group-data-lit/spot:hidden">
            <StepCard {...props} target="" />
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<TutorialScreenProps>> = {
  default: {"step":2,"steps":5,"target":"","text":"Press the dodge button just before an attack lands to slip past it.","texture":0,"title":"Dodge","waits":false,"x":64,"y":120},
  spotlight: {"step":3,"steps":5,"target":"dodge-button","text":"This is your dodge. Press it just before an attack lands.","texture":0,"title":"Your dodge","waits":true,"x":64,"y":120},
  targetMissing: {"step":3,"steps":5,"target":"dodge-button-offscreen","text":"This is your dodge. Press it just before an attack lands.","texture":0,"title":"Your dodge","waits":true,"x":64,"y":120},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Next","back":"Skip"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"tutorial","layer":"popover","focus":"","under":"none","input":"all","pause":false,"capture":false,"remember":true,"back":"skip"} as const;
