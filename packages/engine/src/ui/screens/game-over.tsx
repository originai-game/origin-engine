import {Show, Text, packClass, type UiEvent, View, formatDuration, t} from "@origingame/engine/ui";
import { Button } from "../controls/button";
import "./game-over.en.ogt";

export interface GameOverScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** The defeat's line. */
  title: string;
  /** The run's score. */
  score: number;
  /** The best score. */
  best: number;
  /** A revive is offered. */
  offer?: boolean;
  /** The revive's price, as text (50 gems, Watch an ad). */
  price: string;
  /** Seconds the offer stays. */
  left: number;
  /** A checkpoint exists. */
  checkpoint?: boolean;
  /** Content the caller supplies (was the offerArt snippet). */
  offerArt?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the stats snippet). */
  stats?: (() => JSX.Element) | undefined;
  /** The checkpoint event, with the row's key where it comes from a row. */
  onCheckpoint?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The decline event, with the row's key where it comes from a row. */
  onDecline?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The quit event, with the row's key where it comes from a row. */
  onQuit?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The retry event, with the row's key where it comes from a row. */
  onRetry?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The revive event, with the row's key where it comes from a row. */
  onRevive?: ((e: UiEvent, key?: string) => void) | undefined;
}

/**
 * Game over: the defeat's line, the run's score and the best, an optional
 * revive offer with its price and the seconds it stays, and Retry,
 * Checkpoint and Quit. The game decides the offer and what each choice
 * costs.
 */
export function GameOverScreen(props: GameOverScreenProps) {
  return (
    <View class={`absolute inset-0 grow shrink basis-0 justify-center items-center w-full p-8 gap-2 compact:p-4 ${props.class ?? ""}`}>
      <View class={`text-fg items-center w-full max-w-105 p-0 gap-4 disabled:text-fg-subtle short:grid short:items-center short:gap-y-2 short:gap-x-6 short:max-w-210 short:grid-cols-[1fr_1fr] short:grid-rows-[1fr_auto_auto_auto_auto_1fr] short:justify-items-center ${packClass("Panel", "frame")}`}>
        <Text sound="error" class={`font-display text-display-lg compact:text-display-md font-bold leading-flat tracking-md uppercase text-danger max-w-full text-center m-0 short:text-8xl`}>
          {props.title}
        </Text>
        <View class="w-full h-0.5">
          <Text class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rotate-45 bg-danger shadow-glow shadow-foe size-2.25" />
        </View>
        <View class="flex-row flex-wrap justify-center gap-y-1 gap-x-6 items-end">
          <View class="items-center shrink">
            <Text class={`tracking-label uppercase text-fg-muted text-xs font-semibold m-0`}>
              {t("screen.game-over.score")}
            </Text>
            <Text class={`tabular-nums text-fg text-5xl leading-flat font-bold m-0`}>
              {t("screen.game-over.Value", { value: props.score })}
            </Text>
          </View>
          <Text class={`text-fg-muted text-title font-bold tabular-nums m-0 shrink`}>
            {t("screen.game-over.Best", { best: props.best })}
          </Text>
        </View>
        <View class="self-stretch">{props.stats?.()}</View>
        <Show when={props.offer} fallback={
            <View class="grid justify-items-stretch self-stretch gap-2 short:justify-self-stretch">
              <Show when={props.onRetry}>
                <Button class={`text-accent min-h-9 ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onRetry?.(e)}>
                  <Text>{t("screen.game-over.retry")}</Text>
                </Button>
              </Show>
              <Show when={props.checkpoint && props.onCheckpoint}>
                <Button class={`${packClass("Button", "plate")}`} onPress={(e) => props.onCheckpoint?.(e)}>
                  <Text>{t("screen.game-over.checkpoint")}</Text>
                </Button>
              </Show>
              <Show when={props.onQuit}>
                <Button class={`${packClass("Button", "plate")}`} onPress={(e) => props.onQuit?.(e)}><Text>{t("screen.game-over.quit")}</Text></Button>
              </Show>
            </View>
        }>
          <View class={`grid justify-items-stretch self-stretch text-center bg-plate-hi border border-rim nine-plate-cut p-4 gap-2 short:justify-self-stretch`}>
            <Text class="justify-self-center text-center leading-22 text-display-md font-bold tabular-nums text-fg inset-ring-6 inset-ring-gold m-0 size-22 rounded-full short:leading-16 short:size-16">
              {t("screen.game-over.Left", { value: formatDuration(props.left) })}
            </Text>
            <Text class="text-title font-bold tracking-md uppercase text-fg m-0">
              {t("screen.game-over.revive")}
            </Text>
            <View class="self-stretch">{props.offerArt?.()}</View>
            <Show when={props.onRevive}>
              <Button class={`text-accent justify-self-stretch min-h-9 ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onRevive?.(e)}>
                <Text>{t("screen.game-over.Revive", { price: props.price })}</Text>
              </Button>
            </Show>
            <Show when={props.onDecline}>
              <Button class={`justify-self-stretch ${packClass("Button", "plate")}`} onPress={(e) => props.onDecline?.(e)}>
                <Text>{t("screen.game-over.decline")}</Text>
              </Button>
            </Show>
          </View>
        </Show>
      </View>
    </View>
  );
}

/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<GameOverScreenProps>> = {
  default: {"best":24100,"checkpoint":true,"left":5,"offer":true,"price":"50 gems","score":18420,"title":"You died","onRetry":press,"onCheckpoint":press,"onQuit":press,"onRevive":press,"onDecline":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Retry"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"game-over","layer":"modal","focus":"Panel/Actions/Retry","under":"dim","input":"menu","pause":true,"capture":true,"remember":true,"back":"none"} as const;
