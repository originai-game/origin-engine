import type { TextSpan } from "@origingame/engine";
import { For, Show, Text, View, createMemo, packClass } from "@origingame/engine/ui";

/** One active timeline caption line (matching `TimelineCaption` from `og.timeline.captions(world)`). */
export interface SubtitleCaption {
  /** Stable identifier while the caption shows. */
  id: string;
  /** Spoken text in the player's locale. */
  text: string;
  /** Optional speaker name (WebVTT `<v Speaker>` or caption track `speaker`). */
  speaker?: string | undefined;
  /** Optional rich inline style spans (`strong`, `emphasis`, `color`). */
  spans?: readonly TextSpan[] | undefined;
  /** Placement hint: `"bottom"` (default) or `"top"` (`{\an8}` or WebVTT `line:0`). */
  position?: string | undefined;
}

export interface SubtitlesProps {
  /** The game's own lines, oldest first, each { id, speaker, text }; the game drops a line when it is done. */
  lines?: ReadonlyArray<{ id: string; speaker: string; text: string; spans?: readonly TextSpan[] | undefined }> | undefined;
  /** The captions the timelines are saying (og.timeline.captions(world), set each tick), after the game's lines: a film segment's SRT/WebVTT track, a cutscene's caption track. */
  captions?: ReadonlyArray<SubtitleCaption> | undefined;
  /** Lift bottom captions clear of a docked choice bar (FlowScreen sets this while a choice window is open). */
  raised?: boolean | undefined;
}

interface TextRun {
  key: number;
  text: string;
  bold: boolean;
  italic: boolean;
  color: string;
}

/** Splits `text` along Unicode scalar boundaries for `spans` (`strong`, `emphasis`, `color`). */
function buildRuns(text: string, spans?: readonly TextSpan[] | undefined): TextRun[] {
  if (!spans || spans.length === 0 || text.length === 0) {
    return [{ key: 0, text, bold: false, italic: false, color: "" }];
  }
  const chars = [...text];
  const cuts = new Set<number>([0, chars.length]);
  for (const s of spans) {
    const a = Math.max(0, Math.min(chars.length, s.position));
    const b = Math.max(a, Math.min(chars.length, s.position + s.length));
    cuts.add(a);
    cuts.add(b);
  }
  const edges = [...cuts].sort((a, b) => a - b);
  const runs: TextRun[] = [];
  for (let i = 0; i + 1 < edges.length; i++) {
    const a = edges[i]!;
    const b = edges[i + 1]!;
    if (b <= a) continue;
    let bold = false;
    let italic = false;
    let color = "";
    for (const s of spans) {
      if (s.position <= a && s.position + s.length >= b) {
        if (s.name === "strong" || s.name === "b") bold = true;
        else if (s.name === "emphasis" || s.name === "i" || s.name === "em") italic = true;
        const prop = s.properties?.color;
        if (typeof prop === "string" && prop.length > 0) color = prop;
      }
    }
    runs.push({ key: a, text: chars.slice(a, b).join(""), bold, italic, color });
  }
  return runs.length > 0 ? runs : [{ key: 0, text, bold: false, italic: false, color: "" }];
}

interface Entry {
  id: string;
  speaker: string;
  text: string;
  runs: TextRun[];
  past: boolean;
}

function renderEntries(entries: () => Entry[]) {
  return (
    <View class={`items-center gap-2 max-w-240 py-4 px-8 rounded-panel compact:px-4 compact:py-3 ${packClass("Plate", "frame")}`}>
      <For each={entries()} key={(l) => l.id}>
        {(l) => (
          <View class="group/line items-center gap-0.5" data-past={l().past}>
            <Show when={l().speaker}>
              <Text class="text-sm font-bold uppercase tracking-wide text-gold text-shadow-xs text-shadow-bg">{l().speaker}</Text>
            </Show>
            <Show
              when={l().runs.length > 1 || l().runs[0]?.bold || l().runs[0]?.italic || Boolean(l().runs[0]?.color)}
              fallback={
                <Text class="text-2xl font-semibold leading-snug text-center text-fg text-shadow-sm text-shadow-bg group-data-past/line:text-sm compact:text-xl">{l().text}</Text>
              }
            >
              <Text class="text-2xl font-semibold leading-snug text-center text-fg text-shadow-sm text-shadow-bg group-data-past/line:text-sm compact:text-xl">
                <For each={l().runs} key={(r) => r.key}>
                  {(r) => (
                    <Text
                      class={`${r().bold ? "font-bold" : ""} ${r().italic ? "italic" : ""}`}
                      style={r().color ? { color: r().color } : undefined}
                    >
                      {r().text}
                    </Text>
                  )}
                </For>
              </Text>
            </Show>
          </View>
        )}
      </For>
    </View>
  );
}

/**
 * The latest lines at the foot (or top, for `position: "top"` WebVTT cues) of
 * the screen, centred in HUD type over a soft dark band: the speaker in gold
 * small capitals over the words in white, with rich inline spans. Only older
 * history entries in `lines` step back smaller; simultaneous caption tracks
 * (such as bilingual subtitles) draw at full legibility together.
 */
export function Subtitles(props: SubtitlesProps) {
  const bottom = createMemo((): Entry[] => {
    const lines = props.lines ?? [];
    const caps = (props.captions ?? []).filter((c) => c.position !== "top");
    return [
      ...lines.map((l, i) => ({
        id: `line:${l.id}`,
        speaker: l.speaker,
        text: l.text,
        runs: buildRuns(l.text, l.spans),
        past: i < lines.length - 1,
      })),
      ...caps.map((c) => ({
        id: `caption:${c.id}`,
        speaker: c.speaker ?? "",
        text: c.text,
        runs: buildRuns(c.text, c.spans),
        past: false,
      })),
    ];
  });
  const top = createMemo((): Entry[] =>
    (props.captions ?? [])
      .filter((c) => c.position === "top")
      .map((c) => ({
        id: `caption:${c.id}`,
        speaker: c.speaker ?? "",
        text: c.text,
        runs: buildRuns(c.text, c.spans),
        past: false,
      })),
  );
  return (
    <Show when={bottom().length > 0 || top().length > 0}>
      <View class="absolute inset-0 pointer-events-none">
        <Show when={top().length > 0}>
          <View class={`absolute inset-x-0 top-0 items-center pointer-events-none pt-[calc(env(hud-inset-top)+2rem)] pl-[calc(env(hud-inset-left)+1.5rem)] pr-[calc(env(hud-inset-right)+1.5rem)] ${packClass("Screen", "frame")}`} role="log" live="polite">
            {renderEntries(top)}
          </View>
        </Show>
        <Show when={bottom().length > 0}>
          <View
            class={`absolute inset-x-0 bottom-0 justify-end items-center pointer-events-none pl-[calc(env(hud-inset-left)+1.5rem)] pr-[calc(env(hud-inset-right)+1.5rem)] ${
              props.raised
                ? "pb-[calc(env(hud-inset-bottom)+11rem)] compact:pb-[calc(env(hud-inset-bottom)+14rem)]"
                : "pb-[calc(env(hud-inset-bottom)+3rem)] compact:pb-[calc(env(hud-inset-bottom)+1.5rem)]"
            } ${packClass("Screen", "frame")}`}
            role="log"
            live="polite"
          >
            {renderEntries(bottom)}
          </View>
        </Show>
      </View>
    </Show>
  );
}

export const fixtures: Record<string, Partial<SubtitlesProps>> = {
  default: { lines: [{ id: "1", speaker: "Keeper", text: "The lamps went dark three nights ago." }], captions: [] },
  captions: { lines: [], captions: [{ id: "7:0:2", speaker: "林然", text: "你在等谁？" }, { id: "7:0:3", text: "Who are you waiting for?" }] },
};
