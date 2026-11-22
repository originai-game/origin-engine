import type { Texture } from "@origingame/engine";
import { For, Icon, Image, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmSongPlateProps {
  /** The song's title. */
  title: string;
  /** The artist as the game words it; empty hides it. */
  artist: string;
  /** The tempo as the game words it (174, 90–180); empty hides it. */
  bpm: string;
  /** The cover art texture (og.load.texture); 0 shows the music glyph in its place. */
  jacket: Texture | 0;
  /** The chart's family (easy, normal, hard, expert, master, special): the colour of its chip, on data-tier, never alone. */
  tier: string;
  /** The chart's name as the game words it (Expert); empty hides the chip. */
  chart: string;
  /** The chart's level, beside its name; 0 hides it. */
  rating: number;
  /** The modifiers in play, keyed rows of { id, label } in the game's order, each on a tag (×1.0, Mirror). */
  modifiers: ReadonlyArray<{ id: string; label: string }>;
}

/**
 * The song in play, at the screen's top corner: the jacket with a lit
 * edge, the title, the artist and tempo, the chart's chip in its tier's
 * colour with its level, and the modifiers in play. Read only.
 */
export function RhythmSongPlate(props: RhythmSongPlateProps) {
  return (
    <View
      class={`flex-row items-center gap-4 max-w-lg text-fg`}
      role="group"
      label={t("rhythm.plate.name", { title: props.title })}
    >
      <View class={`shrink size-21 compact:size-14 rounded-panel bg-linear-to-b from-rim-strong via-rim to-rim-soft p-0.5`}>
        <Show when={props.jacket} fallback={
          <View class={`size-full rounded-panel bg-plate-hi flex-row items-center justify-center`}>
            <Icon class={`size-11/25 text-hud-accent-ink`} name="music" />
          </View>
        }>
          <Image class={`size-full rounded-panel object-cover`} src={props.jacket} />
        </Show>
      </View>
      <View class={`flex-col gap-0.5 min-w-0 grow`}>
        <Text class={`text-title compact:text-base leading-tight font-bold whitespace-nowrap text-ellipsis overflow-hidden text-fg text-stroke-3 text-stroke-bg`}>
          {props.title}
        </Text>
        <Show when={props.artist !== "" || props.bpm !== ""}>
          <Text class={`text-base compact:text-caption font-semibold whitespace-nowrap text-ellipsis overflow-hidden text-fg-muted text-stroke-3 text-stroke-bg`}>
            {props.artist !== "" && props.bpm !== "" ? t("rhythm.plate.byline", { artist: props.artist, bpm: props.bpm }) : (props.artist !== "" ? props.artist : t("rhythm.songs.bpm", { bpm: props.bpm }))}
          </Text>
        </Show>
        <Show when={props.chart !== "" || (props.modifiers ?? []).length}>
          <View class={`flex flex-row flex-wrap gap-1 mt-0.5`}>
            <Show when={props.chart !== ""}>
              <Text
                class={`max-w-full py-0 px-1 text-xs font-bold tracking-label uppercase whitespace-nowrap text-ellipsis overflow-hidden text-fg bg-chart-normal nine-chip-cut data-tier-easy:bg-chart-easy data-tier-hard:bg-chart-hard data-tier-expert:bg-fever data-tier-master:bg-chart-master data-tier-special:bg-chart-master ${packClass("Chip", "frame")}`}
                data-tier-easy={props.tier === "easy"}
                data-tier-hard={props.tier === "hard"}
                data-tier-expert={props.tier === "expert"}
                data-tier-master={props.tier === "master"}
                data-tier-special={props.tier === "special"}
              >
                {props.rating > 0 ? t("rhythm.plate.chart", { chart: props.chart, rating: props.rating }) : props.chart}
              </Text>
            </Show>
            <For each={props.modifiers ?? []} key={(m) => m.id}>
              {(m) => (
                <Text class={`max-w-full py-0 px-1 text-xs font-bold tracking-label uppercase whitespace-nowrap text-ellipsis overflow-hidden text-fg bg-well nine-chip-cut ${packClass("Chip", "frame")}`}>
                  {m().label}
                </Text>
              )}
            </For>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RhythmSongPlateProps>> = {
  default: {"title":"Neon Tidal","artist":"Halcyon Array","bpm":"174","jacket":0,"tier":"expert","chart":"Expert","rating":27,"modifiers":[{"id":"speed","label":"×1.0"},{"id":"mirror","label":"Mirror"}]},
  plain: {"title":"Skyline Drive","artist":"","bpm":"","jacket":0,"tier":"normal","chart":"","rating":0,"modifiers":[]},
  master: {"title":"Glasswing","artist":"Teleport Choir","bpm":"200","jacket":0,"tier":"master","chart":"Master","rating":31,"modifiers":[{"id":"hidden","label":"Hidden"}]},
  special: {"title":"Final Ascent","artist":"Orchid Unit","bpm":"90–240","jacket":0,"tier":"special","chart":"Special","rating":99,"modifiers":[]},
  overflow: {"title":"The Unbearably Long Extended Twelve-Minute Midnight Orchestral Remix of a Song","artist":"The Philharmonic Synthesizer Ensemble of the Northern Coast featuring Everyone","bpm":"60–240","jacket":0,"tier":"special","chart":"Remaster Ultimate Challenge","rating":16.5,"modifiers":[{"id":"speed","label":"×10.5"},{"id":"mirror","label":"Mirror"},{"id":"hidden","label":"Hidden"},{"id":"sudden","label":"Sudden death"}]},
};
