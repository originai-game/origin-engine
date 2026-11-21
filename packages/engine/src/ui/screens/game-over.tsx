import {Show, Text, packClass, type UiEvent, View, formatDuration, t} from "@origingame/engine/ui";
import { Button } from "../controls/button";
import "./game-over.en.ogt";

export interface GameOverScreenProps {
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
  onCheckpoint?: (e: UiEvent, key?: string) => void;
  /** The decline event, with the row's key where it comes from a row. */
  onDecline?: (e: UiEvent, key?: string) => void;
  /** The quit event, with the row's key where it comes from a row. */
  onQuit?: (e: UiEvent, key?: string) => void;
  /** The retry event, with the row's key where it comes from a row. */
  onRetry?: (e: UiEvent, key?: string) => void;
  /** The revive event, with the row's key where it comes from a row. */
  onRevive?: (e: UiEvent, key?: string) => void;
}

export function GameOverScreen(props: GameOverScreenProps) {
  return (
    <View class="grow shrink basis-0 justify-center items-center w-full p-8 gap-2 compact:p-4">
      <View class={`text-fg items-center w-full max-w-105 p-0 gap-4 disabled:text-fg-subtle short:grid short:items-center short:gap-y-2 short:gap-x-6 short:max-w-210 short:grid-cols-[1fr_1fr] short:grid-rows-[1fr_auto_auto_auto_auto_1fr] short:justify-items-center ${packClass("Panel", "frame")}`}>
        <Text class={`font-display font-bold leading-[0.95] tracking-md uppercase text-transparent max-w-full text-center bg-linear-to-b from-glass from-10% via-danger via-48% to-critical m-0 short:text-[4.429rem]`}>
          {props.title}
        </Text>
        <View class="w-full h-[1.5px]">
          <Text class="absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] rotate-45 bg-danger shadow-[0px_0px_11.424px_#ff4d5e] size-[0.643rem]" />
        </View>
        <View class="flex-row flex-wrap justify-center gap-y-1 gap-x-6 items-end">
          <View class="items-center shrink">
            <Text class={`tracking-[0.14em] uppercase text-fg-muted text-[0.857rem] font-semibold m-0`}>
              {t("screen.game-over.score")}
            </Text>
            <Text class={`tabular-nums text-fg text-[2.714rem] leading-flat font-bold m-0`}>
              {t("screen.game-over.Value", { value: props.score })}
            </Text>
          </View>
          <Text class={`text-fg-muted text-[1.714rem] font-bold tabular-nums m-0 shrink`}>
            {t("screen.game-over.Best", { best: props.best })}
          </Text>
        </View>
        <View class="self-stretch">{props.stats?.()}</View>
        <Show when={props.offer} fallback={
            <View class="grid justify-items-stretch self-stretch gap-2 short:justify-self-stretch">
              <Button class={`text-accent min-h-9 ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onRetry?.(e)}>
                <Text>{t("screen.game-over.retry")}</Text>
              </Button>
              <Show when={props.checkpoint}>
                <Button class={`${packClass("Button", "plate")}`} onPress={(e) => props.onCheckpoint?.(e)}>
                  <Text>{t("screen.game-over.checkpoint")}</Text>
                </Button>
              </Show>
              <Button class={`${packClass("Button", "plate")}`} onPress={(e) => props.onQuit?.(e)}><Text>{t("screen.game-over.quit")}</Text></Button>
            </View>
        }>
          <View class={`grid justify-items-stretch self-stretch text-center bg-plate-hi border border-white/18 nine-plate-cut p-4 gap-2 short:justify-self-stretch`}>
            <Text class="justify-self-center text-center leading-[6.286rem] text-[2.429rem] font-bold tabular-nums text-fg shadow-[inset_0px_0px_0px_5.712px_#ffcf5a,inset_0px_0px_0px_8px_#060910cc] m-0 size-[6.286rem] rounded-full short:leading-[4.571rem] short:size-[4.571rem]">
              {t("screen.game-over.Left", { value: formatDuration(props.left) })}
            </Text>
            <Text class="text-[1.714rem] font-bold tracking-md uppercase text-fg m-0">
              {t("screen.game-over.revive")}
            </Text>
            <View class="self-stretch">{props.offerArt?.()}</View>
            <Button class={`text-accent justify-self-stretch min-h-9 ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onRevive?.(e)}>
              <Text>{t("screen.game-over.Revive", { price: props.price })}</Text>
            </Button>
            <Button class={`justify-self-stretch ${packClass("Button", "plate")}`} onPress={(e) => props.onDecline?.(e)}>
              <Text>{t("screen.game-over.decline")}</Text>
            </Button>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<GameOverScreenProps>> = {
  default: {"best":24100,"checkpoint":true,"left":5,"offer":true,"price":"50 gems","score":18420,"title":"You died"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Retry"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"game-over","layer":"modal","focus":"Panel/Actions/Retry","under":"dim","input":"menu","pause":true,"capture":true,"remember":true,"back":"none"} as const;
