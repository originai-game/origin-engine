import { og, type Entity, type FlowView, type Texture } from "@origingame/engine";
import { Image, Show, Text, View, createMemo, packClass, t, useClock, useWorld, type UiEvent } from "@origingame/engine/ui";
import { FlowChoices } from "./flow-choices";
import { Subtitles } from "./subtitles";
import "./flow-screen.en.ogt";

export interface FlowPictureProps {
  /** The footage's picture: FlowView.texture, or a cutscene's og.VideoPlayer.texture (0 before its first frame). */
  picture: Texture | 0;
  /** The picture dissolving out under it (FlowView.previous; 0: none). */
  previous?: Texture | 0 | undefined;
  /** How far the dissolve is: the picture's opacity over the previous one (FlowView.mix). */
  mix?: number | undefined;
  /** Given, a Skip button at the top corner calls it: a cutscene the player has seen (og.flow.skip, or the game's own). */
  onSkip?: ((e: UiEvent) => void) | undefined;
}

/**
 * The footage filling the surface, cropped to cover it, over a deep shade
 * (so a live scene under the flow shows only where the footage has none):
 * the previous picture under it while a dissolve runs. FlowChoices'
 * hotspot fractions land on it because it fills the same surface.
 */
export function FlowPicture(props: FlowPictureProps) {
  return (
    <View class="absolute inset-0">
      <Show when={props.previous}>
        <Image class="absolute inset-0 size-full object-cover" src={props.previous} />
      </Show>
      <Show when={props.picture}>
        <Image class="absolute inset-0 size-full object-cover" src={props.picture} style={{ opacity: props.previous ? (props.mix ?? 1) : 1 }} />
      </Show>
      <Show when={props.onSkip}>
        <View class={`absolute top-[calc(env(hud-inset-top)+16px)] right-[calc(env(hud-inset-right)+16px)] min-h-11 px-4 flex-row items-center justify-center ${packClass("Button", "plate")}`} role="button" onPress={(e) => props.onSkip?.(e)}>
          <Text class="text-sm font-semibold uppercase tracking-wide">{t("screen.flow-screen.skip")}</Text>
        </View>
      </Show>
    </View>
  );
}

export interface FlowScreenProps {
  /** The flow it draws; left out, og.flow.current's (project.ogt's `flow`). */
  flow?: Entity | undefined;
}

/**
 * A flow as the player sees it, what the engine mounts for project.ogt's
 * `flow`: its picture with its dissolve (a segment with no footage leaves
 * the live scene showing), the captions its timelines say, and its choice
 * window with hotspots and quick-time progress, each press passed to
 * og.flow.choose. Read from og.flow.view each tick; nothing of it is
 * world state.
 */
export function FlowScreen(props: FlowScreenProps) {
  const world = useWorld();
  const clock = useClock();
  const view = createMemo((): FlowView | null => {
    clock();
    const flow = props.flow ?? og.flow.current(world);
    return flow === null ? null : og.flow.view(world, flow);
  });
  const captions = createMemo(() => (clock(), og.timeline.captions(world)));
  return (
    <View class="absolute inset-0">
      <FlowPicture picture={view()?.texture ?? 0} previous={view()?.previous ?? 0} mix={view()?.mix ?? 1} />
      <Subtitles captions={captions()} />
      <FlowChoices view={view()} onChoose={(e, index) => { const v = view(); if (v) void og.flow.choose(e.world, v.flow, index); }} />
    </View>
  );
}

export const fixtures: Record<string, Partial<FlowPictureProps>> = {
  default: { picture: 0, previous: 0, mix: 1 },
  skip: { picture: 0, onSkip: () => {} },
};
