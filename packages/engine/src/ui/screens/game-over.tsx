import {Show, Text, packClass, type UiEvent, View, formatDuration, t} from "@origingame/engine/ui";
import { Backdrop } from "../controls/backdrop";
import { Button } from "../controls/button";
import { HintBar } from "../controls/hint-bar";
import "./game-over.en.strings.ts";

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
    <View class={`absolute inset-0 flex-col justify-center items-center gap-8 pt-[calc(env(hud-inset-top)+2rem)] pb-[calc(env(hud-inset-bottom)+4.5rem)] px-8 text-fg compact:gap-5 compact:px-5 short:flex-row short:gap-10 short:pb-12 ${packClass("Screen", "frame")} ${props.class ?? ""}`}>
      <Backdrop art={0} over veil="full" />
      <View class="flex-col items-center gap-4 max-w-full short:shrink short:basis-0 short:grow">
        <Text sound="error" class={`max-w-full text-center m-0 ${packClass("Screen", "heading")} text-display-3xl tracking-display-3xl compact:text-display-xl short:text-display-lg`}>
          {props.title}
        </Text>
        <View class="flex-col items-center gap-1">
          <Text class={`m-0 ${packClass("Screen", "note")} text-base font-bold uppercase tracking-kicker`}>
            {t("screen.game-over.score")}
          </Text>
          <Text class={`m-0 ${packClass("Screen", "figure")} ${packClass("Count", "roll")} compact:text-display-xl`}>
            {t("screen.game-over.Value", { value: props.score })}
          </Text>
          <Text class={`m-0 ${packClass("Screen", "note")} text-lg font-bold tabular-nums`}>
            {t("screen.game-over.Best", { best: props.best })}
          </Text>
        </View>
        <View class="self-stretch">{props.stats?.()}</View>
      </View>
      <View class="flex-col items-stretch gap-3 w-100 max-w-full">
        <Show when={props.offer} fallback={
          <>
            <Show when={props.onRetry}>
              <Button variant="primary" size="lg" onPress={(e) => props.onRetry?.(e)} autofocus>
                <Text>{t("screen.game-over.retry")}</Text>
              </Button>
            </Show>
            <Show when={props.checkpoint && props.onCheckpoint}>
              <Button onPress={(e) => props.onCheckpoint?.(e)}>
                <Text>{t("screen.game-over.checkpoint")}</Text>
              </Button>
            </Show>
            <Show when={props.onQuit}>
              <Button variant="ghost-muted" onPress={(e) => props.onQuit?.(e)}><Text>{t("screen.game-over.quit")}</Text></Button>
            </Show>
          </>
        }>
          <View class="flex-row items-center justify-center gap-4">
            <Text class="text-center leading-18 text-display-sm font-bold tabular-nums m-0 size-18 rounded-full inset-ring-marker inset-ring-accent text-fg short:leading-14 short:size-14">
              {t("screen.game-over.Left", { value: formatDuration(props.left) })}
            </Text>
            <Text class={`m-0 shrink ${packClass("Screen", "note")} text-xl font-bold`}>
              {t("screen.game-over.revive")}
            </Text>
          </View>
          <View class="self-stretch">{props.offerArt?.()}</View>
          <Show when={props.onRevive}>
            <Button variant="primary" size="lg" onPress={(e) => props.onRevive?.(e)} autofocus>
              <Text>{t("screen.game-over.Revive", { price: props.price })}</Text>
            </Button>
          </Show>
          <Show when={props.onDecline}>
            <Button variant="ghost-muted" onPress={(e) => props.onDecline?.(e)}>
              <Text>{t("screen.game-over.decline")}</Text>
            </Button>
          </Show>
        </Show>
      </View>
      <View class="absolute inset-x-0 bottom-0 flex-row justify-center pointer-events-none pb-[calc(env(hud-inset-bottom)+1.5rem)] compact:pb-[calc(env(hud-inset-bottom)+0.75rem)]">
        <HintBar hints={[{ action: "confirm", label: t("screen.common.select") }]} />
      </View>
    </View>
  );
}

/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<GameOverScreenProps>> = {
  default: {"best":24100,"checkpoint":true,"left":5,"offer":true,"price":"50 gems","score":18420,"title":"You died","onRetry":press,"onCheckpoint":press,"onQuit":press,"onRevive":press,"onDecline":press},
  retry: {"best":24100,"checkpoint":true,"left":0,"offer":false,"price":"","score":18420,"title":"You died","onRetry":press,"onCheckpoint":press,"onQuit":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Retry"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"game-over","layer":"modal","focus":"Panel/Actions/Retry","under":"dim","input":"menu","pause":true,"capture":true,"remember":true,"back":"none"} as const;
