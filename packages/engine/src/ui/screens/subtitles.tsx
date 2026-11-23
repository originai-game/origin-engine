import { For, Show, Text, View, createMemo, packClass } from "@origingame/engine/ui";

export interface SubtitlesProps {
  /** The game's own lines, oldest first, each { id, speaker, text }; the game drops a line when it is done. */
  lines?: ReadonlyArray<{ id: string; speaker: string; text: string }> | undefined;
  /** The captions the timelines are saying (og.timeline.captions(world), set each tick), after the game's lines: a film segment's SRT track, a cutscene's caption track. */
  captions?: ReadonlyArray<{ id: string; text: string }> | undefined;
}

/**
 * The latest lines at the foot of the screen, centred in HUD type over a
 * soft dark band: the speaker in gold small capitals over the words in
 * white. An older line steps back, smaller. Nothing shows without a line.
 */
export function Subtitles(props: SubtitlesProps) {
  const all = createMemo(() => [
    ...(props.lines ?? []).map((l) => ({ id: `line:${l.id}`, speaker: l.speaker, text: l.text })),
    ...(props.captions ?? []).map((c) => ({ id: `caption:${c.id}`, speaker: "", text: c.text })),
  ]);
  return (
    <Show when={all().length > 0}>
      <View class={`absolute inset-0 justify-end items-center pt-12 pb-[calc(env(hud-inset-bottom)+48px)] pl-[calc(env(hud-inset-left)+24px)] pr-[calc(env(hud-inset-right)+24px)] ${packClass("Screen", "frame")}`} role="log" live="polite">
        <View class={`items-center gap-2 max-w-240 py-6 px-12 bg-glass compact:px-4 ${packClass("Plate", "frame")}`}>
          <For each={all()} key={(l) => l.id}>
            {(l, i) => (
              <View class="group/line items-center gap-0.5" data-past={i() < all().length - 1}>
                <Show when={l().speaker}>
                  <Text class="text-sm font-bold uppercase tracking-wide text-gold text-shadow-xs text-shadow-bg">{l().speaker}</Text>
                </Show>
                <Text class="text-2xl font-semibold leading-snug text-center text-fg text-shadow-sm text-shadow-bg group-data-past/line:text-sm compact:text-xl">{l().text}</Text>
              </View>
            )}
          </For>
        </View>
      </View>
    </Show>
  );
}

export const fixtures: Record<string, Partial<SubtitlesProps>> = {
  default: { lines: [{ id: "1", speaker: "Keeper", text: "The lamps went dark three nights ago." }], captions: [] },
  captions: { lines: [], captions: [{ id: "7:0:2", text: "你在等谁？" }, { id: "7:0:3", text: "Who are you waiting for?" }] },
};
