import { og, type Entity, type FlowView, type Texture } from "@origingame/engine";
import { Image, Show, Text, View, createMemo, packClass, t, useClock, useWorld, type UiEvent } from "@origingame/engine/ui";
import { Progress } from "../controls/progress";
import { FlowChoices, type VideoBox } from "./flow-choices";
import { Subtitles } from "./subtitles";
import "./flow-screen.en.ogt";

export interface FlowPictureProps {
  /** The footage's picture: FlowView.texture, or a cutscene's og.VideoPlayer.texture (0 before its first frame). */
  picture: Texture | 0;
  /** The picture dissolving out under it (FlowView.previous; 0: none). */
  previous?: Texture | 0 | undefined;
  /** How far the dissolve is: the picture's opacity over the previous one (FlowView.mix). */
  mix?: number | undefined;
  /** How the footage fits the surface (`"cover"` crops to fill; `"contain"` letterboxes/pillarboxes cleanly). Default `"cover"`. */
  fit?: "cover" | "contain" | undefined;
  /** Given, a Skip button at the top corner calls it: a cutscene the player has seen (og.flow.skip, or the game's own). */
  onSkip?: ((e: UiEvent) => void) | undefined;
}

/**
 * Computes the rendered image box in normalized surface coordinates (`0..1`)
 * matching CSS `object-cover` or `object-contain`, so hotspots land on exact
 * video features on landscape desktop, ultrawide, and 9:16 mobile portrait.
 */
export function computeVideoBox(
  viewportWidth: number,
  viewportHeight: number,
  videoWidth: number,
  videoHeight: number,
  fit: "cover" | "contain" = "cover",
): VideoBox {
  if (viewportWidth <= 0 || viewportHeight <= 0 || videoWidth <= 0 || videoHeight <= 0) {
    return { left: 0, top: 0, width: 1, height: 1 };
  }
  const scale =
    fit === "contain"
      ? Math.min(viewportWidth / videoWidth, viewportHeight / videoHeight)
      : Math.max(viewportWidth / videoWidth, viewportHeight / videoHeight);
  const width = (videoWidth * scale) / viewportWidth;
  const height = (videoHeight * scale) / viewportHeight;
  return {
    left: (1 - width) * 0.5,
    top: (1 - height) * 0.5,
    width,
    height,
  };
}

/**
 * The footage filling the surface (`object-cover` or `object-contain`), never
 * blocking pointer input to live 3D scenes underneath: the previous picture
 * sits under it while a dissolve runs.
 */
export function FlowPicture(props: FlowPictureProps) {
  const objectFit = () => (props.fit === "contain" ? "object-contain" : "object-cover");
  return (
    <View class={`absolute inset-0 pointer-events-none ${packClass("Screen", "frame")}`}>
      <Show when={props.previous}>
        <Image class={`absolute inset-0 size-full pointer-events-none ${objectFit()}`} src={props.previous} />
      </Show>
      <Show when={props.picture}>
        <Image
          class={`absolute inset-0 size-full pointer-events-none ${objectFit()}`}
          src={props.picture}
          style={{ opacity: props.previous ? (props.mix ?? 1) : 1 }}
        />
      </Show>
      <Show when={props.onSkip}>
        <View class="absolute top-0 right-0 pt-[env(hud-inset-top)] pr-[env(hud-inset-right)] pointer-events-none">
          <View class={`mt-4 mr-4 min-h-11 px-4 flex-row items-center justify-center pointer-events-auto ${packClass("Button", "plate")}`} role="button" onPress={(e) => props.onSkip?.(e)}>
            <Text class="text-sm font-semibold uppercase tracking-wide">{t("screen.flow-screen.skip")}</Text>
          </View>
        </View>
      </Show>
    </View>
  );
}

export interface FlowScreenProps {
  /** The flow it draws; left out, og.flow.current's (project.ts's `flow`). */
  flow?: Entity | undefined;
  /** How the footage fits the play surface (`"cover"` default, or `"contain"`). */
  fit?: "cover" | "contain" | undefined;
  /** Optional skip handler shown at the top corner. */
  onSkip?: ((e: UiEvent) => void) | undefined;
}

/**
 * A flow as the player sees it, what the engine mounts for project.ts's
 * `flow`: its picture with dissolve, subtitles lifted clear of active choices,
 * aspect-accurate hotspots (choice & inspect), swipe/drag touch gestures,
 * rich QTEs, and a subtle buffering badge while waiting on network chunks.
 * It sits over a picture, so it resolves in the dark scheme in both schemes.
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
  const videoBox = createMemo((): VideoBox => {
    const v = view();
    if (!v || v.videoWidth <= 0 || v.videoHeight <= 0) {
      return { left: 0, top: 0, width: 1, height: 1 };
    }
    const vp = og.ui.viewport(world);
    return computeVideoBox(vp.width, vp.height, v.videoWidth, v.videoHeight, props.fit ?? "cover");
  });
  const wantsGesture = createMemo(() =>
    Boolean(view()?.qtes.some((q) => (q.kind === "swipe" || q.kind === "drag") && !q.done)),
  );

  // Ephemeral pointer drag state (presentation-only; feeds deterministic `og.flow.gesture`).
  let dragStart: { x: number; y: number } | null = null;
  const onDragStart = (e: UiEvent) => {
    dragStart = { x: e.x, y: e.y };
  };
  const onDragMove = (e: UiEvent) => {
    const v = view();
    if (!dragStart || !v) return;
    const dx = e.x - dragStart.x;
    const dy = e.y - dragStart.y;
    const dist = Math.hypot(dx, dy);
    if (dist < 12) return;
    const direction = Math.abs(dx) >= Math.abs(dy) ? (dx > 0 ? "right" : "left") : (dy > 0 ? "down" : "up");
    const progress = Math.min(1, dist / 160);
    void og.flow.gesture(e.world, v.flow, { kind: "drag", direction, progress });
  };
  const onDragEnd = () => {
    dragStart = null;
  };

  return (
    <View class="absolute inset-0 scheme-dark pointer-events-none">
      <FlowPicture
        picture={view()?.texture ?? 0}
        previous={view()?.previous ?? 0}
        mix={view()?.mix ?? 1}
        fit={props.fit}
        onSkip={props.onSkip}
      />
      <Show when={wantsGesture()}>
        <View
          class="absolute inset-0 pointer-events-auto"
          onSwipe={(e) => {
            const v = view();
            if (v && (e.key === "up" || e.key === "down" || e.key === "left" || e.key === "right")) {
              void og.flow.gesture(e.world, v.flow, { kind: "swipe", direction: e.key, progress: 1 });
            }
          }}
          onPointerDown={onDragStart}
          onPointerMove={onDragMove}
          onPointerUp={onDragEnd}
        />
      </Show>
      <Subtitles captions={captions()} raised={Boolean(view()?.window)} />
      <FlowChoices
        view={view()}
        videoBox={videoBox()}
        onChoose={(e, index) => {
          const v = view();
          if (v) void og.flow.choose(e.world, v.flow, index);
        }}
        onInspect={(e, hotspot) => {
          const v = view();
          if (v) void og.flow.inspect(e.world, v.flow, hotspot);
        }}
      />
      <Show when={view()?.buffering}>
        <View class="absolute inset-0 items-center justify-center pointer-events-none">
          <View class={`items-center gap-2 py-3 px-5 rounded-panel ${packClass("Plate", "frame")}`}>
            <Text class="text-sm font-semibold uppercase tracking-wide text-fg">{t("screen.flow-screen.buffering")}</Text>
            <Progress value={(view()?.bufferProgress ?? 0) * 100} class="w-40 h-1.5 rounded-none" />
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<FlowPictureProps>> = {
  default: { picture: 0, previous: 0, mix: 1 },
  contain: { picture: 0, previous: 0, mix: 1, fit: "contain" },
  skip: { picture: 0, onSkip: () => {} },
};
